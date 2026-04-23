use lazy_static::lazy_static;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
#[cfg(target_os = "windows")]
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

// --- Configuration ---
const SERVICE_NAME: &str = "_http._tcp.local.";
const DEVICE_HOSTNAME: &str = "focus-totem";
const FOCUS_WALLPAPER_NAME: &str = "focus_wallpaper.jpg";
const APPS_CONFIG_FILE: &str = "apps.toml";

lazy_static! {
    static ref ORIGINAL_WALLPAPER_PATH: Mutex<Option<String>> = Mutex::new(None);
}

#[derive(Debug, Deserialize)]
struct AppsConfig {
    apps: std::collections::HashMap<String, Vec<String>>,
}

fn default_focus_apps() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        vec![
            r"C:\Windows\System32\notepad.exe".to_string(),
            r"C:\Users\Faizy\AppData\Local\BraveSoftware\Brave-Browser\Application\brave.exe"
                .to_string(),
        ]
    }

    #[cfg(target_os = "linux")]
    {
        vec!["gedit".to_string(), "firefox".to_string()]
    }

    #[cfg(target_os = "macos")]
    {
        vec!["TextEdit".to_string(), "Google Chrome".to_string()]
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        vec![]
    }
}

fn parse_apps_config(contents: &str) -> Result<AppsConfig, toml::de::Error> {
    toml::from_str(contents)
}

fn select_apps_for_current_os(config: &AppsConfig) -> Option<Vec<String>> {
    config.apps.get(std::env::consts::OS).cloned()
}

fn sanitize_apps(apps: Vec<String>) -> Vec<String> {
    apps.into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn load_focus_apps() -> Vec<String> {
    match fs::read_to_string(APPS_CONFIG_FILE) {
        Ok(contents) => match parse_apps_config(&contents) {
            Ok(config) => {
                let selected = select_apps_for_current_os(&config).unwrap_or_default();
                let cleaned = sanitize_apps(selected);
                if cleaned.is_empty() {
                    let defaults = default_focus_apps();
                    println!(
                        "No apps configured for OS '{}' in '{}'. Using {} default app(s).",
                        std::env::consts::OS,
                        APPS_CONFIG_FILE,
                        defaults.len()
                    );
                    defaults
                } else {
                    println!(
                        "Loaded {} app(s) for OS '{}' from '{}'.",
                        cleaned.len(),
                        std::env::consts::OS,
                        APPS_CONFIG_FILE
                    );
                    cleaned
                }
            }
            Err(e) => {
                eprintln!(
                    "ERROR: Could not parse '{}': {}. Falling back to defaults.",
                    APPS_CONFIG_FILE, e
                );
                default_focus_apps()
            }
        },
        Err(e) => {
            eprintln!(
                "ERROR: Could not read '{}': {}. Falling back to defaults.",
                APPS_CONFIG_FILE, e
            );
            default_focus_apps()
        }
    }
}

fn launch_app(app: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        return Command::new("open").args(["-a", app]).spawn().map(|_| ());
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        return Command::new(app).spawn().map(|_| ());
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unsupported OS: {}", std::env::consts::OS),
        ))
    }
}

fn terminate_app(app: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let image_name = Path::new(app)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(app);

        let output = Command::new("taskkill")
            .args(["/F", "/IM", image_name])
            .output()?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("taskkill failed for '{}': {}", image_name, stderr.trim()),
        ));
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let output = Command::new("pkill").args(["-f", app]).output()?;

        // pkill exit code: 0 => matched/killed, 1 => no matching process (safe for us)
        if output.status.success() || output.status.code() == Some(1) {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("pkill failed for '{}': {}", app, stderr.trim()),
        ));
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unsupported OS: {}", std::env::consts::OS),
        ))
    }
}

fn activate_focus_mode(focus_apps: &[String]) {
    println!("Activating focus mode automations...");

    // Save current wallpaper so we can restore it later
    if let Ok(path) = wallpaper::get() {
        println!("Saved original wallpaper: {}", path);
        *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(path);
    }

    // Set focus wallpaper
    match fs::canonicalize(FOCUS_WALLPAPER_NAME) {
        Ok(absolute_path) => {
            if wallpaper::set_from_path(absolute_path.to_str().unwrap()).is_ok() {
                println!("Focus wallpaper has been set.");
            } else {
                eprintln!("ERROR: Failed to set focus wallpaper.");
            }
        }
        Err(e) => eprintln!(
            "ERROR: Could not find wallpaper '{}': {}",
            FOCUS_WALLPAPER_NAME, e
        ),
    }

    // Launch apps
    println!(
        "Launching focus applications for {}...",
        std::env::consts::OS
    );
    for app in focus_apps {
        match launch_app(app) {
            Ok(_) => println!("Successfully launched '{}'", app),
            Err(e) => eprintln!("ERROR: Failed to launch '{}': {}", app, e),
        }
    }
}

fn deactivate_focus_mode(focus_apps: &[String]) {
    println!("Deactivating focus mode automations...");

    // Restore wallpaper
    let mut original_path = ORIGINAL_WALLPAPER_PATH.lock().unwrap();
    if let Some(path) = original_path.as_deref() {
        if wallpaper::set_from_path(path).is_ok() {
            println!("Restored original wallpaper: {}", path);
        } else {
            eprintln!("ERROR: Failed to restore wallpaper.");
        }
    }
    *original_path = None;

    // Close apps
    println!("Closing focus applications for {}...", std::env::consts::OS);
    for app in focus_apps {
        if let Err(e) = terminate_app(app) {
            eprintln!("ERROR: Failed to close '{}': {}", app, e);
        }
    }
}

fn discover_device(search_duration: Duration) -> Option<String> {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let receiver = mdns
        .browse(SERVICE_NAME)
        .expect("Failed to browse for service");
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

    let focus_apps = load_focus_apps();

    let mut is_focused = false;
    let mut esp32_address: Option<String> = None;

    println!("Starting Focus Mode client (Rust version)...");
    println!("Detected OS: {}", std::env::consts::OS);
    println!("Configured apps: {:?}", focus_apps);

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
                    if response.status().is_success()
                        && response.text().unwrap_or_default() == "FOCUS_ON"
                    {
                        if !is_focused {
                            is_focused = true;
                            println!("--- FOCUS MODE ACTIVATED ---");
                            activate_focus_mode(&focus_apps);
                        }
                    }
                }
                Err(_) => {
                    if is_focused {
                        is_focused = false;
                        println!("--- FOCUS MODE DEACTIVATED ---");
                        deactivate_focus_mode(&focus_apps);
                    }
                    println!("Lost connection to device. Returning to search mode.");
                    esp32_address = None;
                }
            }
        }

        thread::sleep(Duration::from_secs(3));
    }
}
