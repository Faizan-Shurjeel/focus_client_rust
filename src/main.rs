// use mdns_sd::{ServiceDaemon, ServiceEvent};
// use reqwest::blocking::Client;
// use lazy_static::lazy_static;
// use std::sync::Mutex;
// use std::thread;
// use std::time::Duration;
// use std::fs;
// use std::process::Command; // <-- Added for launching apps

// // --- 1. CONFIGURE YOUR APPLICATIONS HERE ---
// // Find the full path to the .exe for each application you want to launch.
// // Example: "C:\\Users\\YourUser\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe"
// const FOCUS_APPS: &[&str] = &[
//     "C:\\Windows\\System32\\notepad.exe",
//     "C:\\Users\\Faizy\\AppData\\Local\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
// ];

// // --- Configuration ---
// const SERVICE_NAME: &str = "_http._tcp.local.";
// const DEVICE_HOSTNAME: &str = "focus-totem";
// const FOCUS_WALLPAPER_NAME: &str = "focus_wallpaper.jpg";

// // --- Safe, global, mutable state ---
// lazy_static! {
//     static ref ORIGINAL_WALLPAPER_PATH: Mutex<Option<String>> = Mutex::new(None);
//     // This will hold the Process IDs (PIDs) of the apps we launch
//     static ref LAUNCHED_PIDS: Mutex<Vec<u32>> = Mutex::new(Vec::new());
// }

// // --- Automation Functions ---

// fn activate_focus_mode() {
//     println!("Activating focus mode automations...");
    
//     // --- 1. Wallpaper Logic ---
//     if let Ok(path) = wallpaper::get() {
//         println!("Saved original wallpaper: {}", &path);
//         *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(path);
//     }
//     match fs::canonicalize(FOCUS_WALLPAPER_NAME) {
//         Ok(absolute_path) => {
//             if wallpaper::set_from_path(absolute_path.to_str().unwrap()).is_ok() {
//                 println!("Focus wallpaper has been set.");
//             } else {
//                 eprintln!("Error setting focus wallpaper.");
//             }
//         }
//         Err(e) => eprintln!("ERROR: Could not find wallpaper '{}': {}", FOCUS_WALLPAPER_NAME, e),
//     }

//     // --- 2. Launch Applications ---
//     println!("Launching focus applications...");
//     let mut pids = LAUNCHED_PIDS.lock().unwrap();
//     pids.clear(); // Clear any old PIDs just in case

//     for app_path in FOCUS_APPS {
//         match Command::new(app_path).spawn() {
//             Ok(child) => {
//                 let pid = child.id();
//                 println!("Successfully launched '{}' with PID: {}", app_path, pid);
//                 pids.push(pid);
//             }
//             Err(e) => {
//                 eprintln!("ERROR: Failed to launch '{}': {}", app_path, e);
//             }
//         }
//     }
// }

// // Replace ONLY this function in src/main.rs

// fn deactivate_focus_mode() {
//     println!("Deactivating focus mode automations...");
    
//     // --- 1. Wallpaper Logic (Unchanged) ---
//     let mut original_path = ORIGINAL_WALLPAPER_PATH.lock().unwrap();
//     if let Some(path) = original_path.as_deref() {
//         if wallpaper::set_from_path(path).is_ok() {
//             println!("Restored original wallpaper: {}", path);
//         } else {
//             eprintln!("Error restoring wallpaper.");
//         }
//     }
//     *original_path = None;

//     // --- 2. Close Applications (WITH BETTER ERROR LOGGING) ---
//     println!("Closing focus applications...");
//     let mut pids = LAUNCHED_PIDS.lock().unwrap();
    
//     for &pid in pids.iter() {
//         match Command::new("taskkill").args(["/F", "/T", "/PID", &pid.to_string()]).output() {
//             Ok(output) => {
//                 if output.status.success() {
//                     println!("Successfully terminated process tree for PID: {}", pid);
//                 } else {
//                     // If taskkill failed, print its error message
//                     let stderr = String::from_utf8_lossy(&output.stderr);
//                     eprintln!("Failed to terminate PID {}. Reason: {}", pid, stderr.trim());
//                 }
//             }
//             Err(e) => {
//                 eprintln!("Error executing taskkill for PID {}: {}", pid, e);
//             }
//         }
//     }

//     // Clear the list of PIDs now that they are closed
//     pids.clear();
// }
// // --- Core Logic ---

// fn discover_device(search_duration: Duration) -> Option<String> {
//     let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
//     let receiver = mdns.browse(SERVICE_NAME).expect("Failed to browse for service");
//     let start_time = std::time::Instant::now();

//     while start_time.elapsed() < search_duration {
//         if let Ok(event) = receiver.recv_timeout(Duration::from_secs(1)) {
//             if let ServiceEvent::ServiceResolved(info) = event {
//                 if info.get_fullname().contains(DEVICE_HOSTNAME) {
//                     let ip = info.get_addresses().iter().next()?;
//                     let port = info.get_port();
//                     let url = format!("http://{}:{}/status", ip, port);
//                     println!("Resolved Focus Totem address: {}", url);
//                     return Some(url);
//                 }
//             }
//         }
//     }
    
//     None
// }

// fn main() {
//     let http_client = Client::builder()
//         .timeout(Duration::from_secs(2))
//         .build()
//         .expect("Failed to build HTTP client");
//     let mut is_focused = false;
//     let mut esp32_address: Option<String> = None;

//     println!("Starting Focus Mode client (Rust version)...");
//     loop {
//         if esp32_address.is_none() {
//             println!("Searching for Focus Totem on the network...");
//             if let Some(found_address) = discover_device(Duration::from_secs(5)) {
//                 esp32_address = Some(found_address);
//             } else {
//                 println!("Device not found. Will retry in 10 seconds.");
//                 thread::sleep(Duration::from_secs(10));
//             }
//         }

//         if let Some(address) = &esp32_address {
//             match http_client.get(address).send() {
//                 Ok(response) => {
//                     if response.status().is_success() && response.text().unwrap_or_default() == "FOCUS_ON" {
//                         if !is_focused {
//                             is_focused = true;
//                             println!("--- FOCUS MODE ACTIVATED ---");
//                             activate_focus_mode();
//                         }
//                     }
//                 }
//                 Err(_) => {
//                     if is_focused {
//                         is_focused = false;
//                         println!("--- FOCUS MODE DEACTIVATED ---");
//                         deactivate_focus_mode();
//                     }
//                     println!("Lost connection to device. Returning to search mode.");
//                     esp32_address = None;
//                 }
//             }
//         }
//         thread::sleep(Duration::from_secs(3));
//     }
// }


// Process ID Commented

use mdns_sd::{ServiceDaemon, ServiceEvent};
use reqwest::blocking::Client;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::fs;
use std::process::Command;
use std::path::Path; // <-- We now need this for getting filenames

// --- 1. CONFIGURE YOUR APPLICATIONS HERE ---
const FOCUS_APPS: &[&str] = &[
    "C:\\Windows\\System32\\notepad.exe",
    "C:\\Users\\Faizy\\AppData\\Local\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
];

// --- Configuration ---
const SERVICE_NAME: &str = "_http._tcp.local.";
const DEVICE_HOSTNAME: &str = "focus-totem";
const FOCUS_WALLPAPER_NAME: &str = "focus_wallpaper.jpg";

// --- Safe, global, mutable state (PID list is now gone) ---
lazy_static! {
    static ref ORIGINAL_WALLPAPER_PATH: Mutex<Option<String>> = Mutex::new(None);
}

// --- Automation Functions ---

fn activate_focus_mode() {
    println!("Activating focus mode automations...");
    
    // --- 1. Wallpaper Logic (Unchanged) ---
    if let Ok(path) = wallpaper::get() {
        println!("Saved original wallpaper: {}", &path);
        *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(path);
    }
    match fs::canonicalize(FOCUS_WALLPAPER_NAME) {
        Ok(absolute_path) => {
            if wallpaper::set_from_path(absolute_path.to_str().unwrap()).is_ok() {
                println!("Focus wallpaper has been set.");
            }
        }
        Err(e) => eprintln!("ERROR: Could not find wallpaper: {}", e),
    }

    // --- 2. Launch Applications (Simpler) ---
    println!("Launching focus applications...");
    for app_path in FOCUS_APPS {
        if let Err(e) = Command::new(app_path).spawn() {
            eprintln!("ERROR: Failed to launch '{}': {}", app_path, e);
        } else {
            println!("Successfully launched '{}'", app_path);
        }
    }
}

fn deactivate_focus_mode() {
    println!("Deactivating focus mode automations...");
    
    // --- 1. Wallpaper Logic (Unchanged) ---
    let mut original_path = ORIGINAL_WALLPAPER_PATH.lock().unwrap();
    if let Some(path) = original_path.as_deref() {
        if wallpaper::set_from_path(path).is_ok() {
            println!("Restored original wallpaper: {}", path);
        }
    }
    *original_path = None;

    // --- 2. Close Applications by Executable Name ---
    println!("Closing focus applications...");
    for app_path in FOCUS_APPS {
        // Extract just the filename (e.g., "brave.exe") from the full path
        if let Some(file_name) = Path::new(app_path).file_name().and_then(|s| s.to_str()) {
            // Use taskkill with /IM (Image Name) to target all processes with this name
            match Command::new("taskkill").args(["/F", "/IM", file_name]).output() {
                Ok(output) if output.status.success() => {
                    println!("Successfully terminated all '{}' processes.", file_name);
                }
                _ => {
                    // This will likely show for apps that were already closed, which is fine.
                    // eprintln!("Could not find or terminate '{}'. It may have already been closed.", file_name);
                }
            }
        }
    }
}

// --- The rest of your code (discover_device, main) remains exactly the same ---
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