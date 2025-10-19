use mdns_sd::{ServiceDaemon, ServiceEvent};
use reqwest::blocking::Client;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::fs; // Moved to the top with other imports

// --- Configuration ---
const SERVICE_NAME: &str = "_http._tcp.local.";
const DEVICE_HOSTNAME: &str = "focus-totem";
const FOCUS_WALLPAPER_NAME: &str = "focus_wallpaper.jpg";

// --- Safe, global, mutable state ---
lazy_static! {
    static ref ORIGINAL_WALLPAPER_PATH: Mutex<Option<String>> = Mutex::new(None);
}

// --- Automation Functions ---

fn activate_focus_mode() {
    println!("Activating focus mode automations...");
    
    // 1. Save the original wallpaper
    if let Ok(path) = wallpaper::get() {
        println!("Saved original wallpaper: {}", &path);
        *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(path);
    } else {
        eprintln!("Error getting original wallpaper.");
    }

    // 2. Set the new focus wallpaper
    match fs::canonicalize(FOCUS_WALLPAPER_NAME) {
        Ok(absolute_path) => {
            println!("Found focus wallpaper at absolute path: {}", absolute_path.display());
            if let Err(e) = wallpaper::set_from_path(absolute_path.to_str().unwrap()) {
                eprintln!("Error setting focus wallpaper: {:?}", e);
            } else {
                println!("Focus wallpaper has been set.");
            }
        }
        Err(e) => {
            eprintln!("ERROR: Could not find or resolve path for '{}': {}", FOCUS_WALLPAPER_NAME, e);
        }
    }
}

fn deactivate_focus_mode() {
    println!("Deactivating focus mode automations...");
    
    let mut original_path = ORIGINAL_WALLPAPER_PATH.lock().unwrap();

    if let Some(path) = original_path.as_deref() {
        if let Err(e) = wallpaper::set_from_path(path) {
            eprintln!("Error restoring wallpaper: {:?}", e);
        } else {
            println!("Restored original wallpaper: {}", path);
        }
    } else {
        println!("No original wallpaper path saved, cannot restore.");
    }
    
    *original_path = None;
}

// --- Core Logic ---

fn discover_device(search_duration: Duration) -> Option<String> {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let receiver = mdns.browse(SERVICE_NAME).expect("Failed to browse for service");
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < search_duration {
        if let Ok(event) = receiver.recv_timeout(Duration::from_secs(1)) {
            if let ServiceEvent::ServiceResolved(info) = event {
                if info.get_fullname().contains(DEVICE_HOSTNAME) {
                    let ip = info.get_addresses().iter().next()?;
                    let port = info.get_port();
                    let url = format!("http://{}:{}/status", ip, port);
                    println!("Resolved Focus Totem address: {}", url);
                    return Some(url);
                }
            }
        }
    }
    
    None
}

fn main() {
    let http_client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("Failed to build HTTP client");
    let mut is_focused = false;
    let mut esp32_address: Option<String> = None;

    println!("Starting Focus Mode client (Rust version)...");
    loop {
        if esp32_address.is_none() {
            println!("Searching for Focus Totem on the network...");
            if let Some(found_address) = discover_device(Duration::from_secs(5)) {
                esp32_address = Some(found_address);
            } else {
                println!("Device not found. Will retry in 4 seconds.");
                thread::sleep(Duration::from_secs(4));
            }
        }

        if let Some(address) = &esp32_address {
            match http_client.get(address).send() {
                Ok(response) => {
                    if response.status().is_success() && response.text().unwrap_or_default() == "FOCUS_ON" {
                        if !is_focused {
                            is_focused = true;
                            println!("--- FOCUS MODE ACTIVATED ---");
                            activate_focus_mode();
                        }
                    }
                }
                Err(_) => {
                    if is_focused {
                        is_focused = false;
                        println!("--- FOCUS MODE DEACTIVATED ---");
                        deactivate_focus_mode();
                    }
                    println!("Lost connection to device. Returning to search mode.");
                    esp32_address = None;
                }
            }
        }
        thread::sleep(Duration::from_secs(3));
    }
}