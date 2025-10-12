use mdns_sd::{ServiceDaemon, ServiceEvent};
use reqwest::blocking::Client;
use std::thread;
use std::time::Duration;

// The service name we are looking for. `_http._tcp.local.` is the standard
// name for web servers on a local network.
const SERVICE_NAME: &str = "_http._tcp.local.";
const DEVICE_HOSTNAME: &str = "focus-totem";

/// Searches the network for the ESP32 device for a given duration.
/// This is the Rust equivalent of the `find_focus_device` function.
fn discover_device(search_duration: Duration) -> Option<String> {
    // Create a new mDNS daemon
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");

    // Browse for the service we want
    let receiver = mdns.browse(SERVICE_NAME).expect("Failed to browse for service");

    let start_time = std::time::Instant::now();

    // Listen for discovery events
    while start_time.elapsed() < search_duration {
        // Check if we received an event
        if let Ok(event) = receiver.recv_timeout(Duration::from_secs(1)) {
            if let ServiceEvent::ServiceResolved(info) = event {
                println!("Found a device: {}", info.get_fullname());
                // We check if the device name contains our unique hostname
                if info.get_fullname().contains(DEVICE_HOSTNAME) {
                    let ip = info.get_addresses().iter().next()?; // Get the first IP address
                    let port = info.get_port();
                    let url = format!("http://{}:{}/status", ip, port);
                    println!("Resolved Focus Totem address: {}", url);
                    return Some(url); // Return the full URL
                }
            }
        }
    }
    
    // If we loop for the whole duration and find nothing, return None
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

    // This is the main application loop, equivalent to `while True:`
    loop {
        // We use a `match` statement to handle the DISCOVERING vs POLLING states.
        match &esp32_address {
            None => {
                // --- STATE: DISCOVERING ---
                println!("Searching for Focus Totem on the network...");
                if let Some(found_address) = discover_device(Duration::from_secs(5)) {
                    esp32_address = Some(found_address);
                } else {
                    println!("Device not found. Will retry in 10 seconds.");
                    thread::sleep(Duration::from_secs(10));
                }
            }
            Some(address) => {
                // --- STATE: POLLING ---
                match http_client.get(address).send() {
                    Ok(response) => {
                        // We successfully contacted the server
                        if response.status().is_success() {
                            if let Ok(text) = response.text() {
                                if text == "FOCUS_ON" {
                                    if !is_focused {
                                        is_focused = true;
                                        println!("FOCUS MODE ACTIVATED. Time to get to work!");
                                        // --- ACTION: Trigger Focus ON actions here ---
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // This block runs if the HTTP request fails (timeout, connection refused, etc.)
                        if is_focused {
                            is_focused = false;
                            println!("FOCUS MODE DEACTIVATED. Welcome back!");
                            // --- ACTION: Trigger Focus OFF actions here ---
                        }
                        println!("Lost connection to device. Returning to search mode.");
                        esp32_address = None; // Go back to discovery mode
                    }
                }
            }
        }
        // Wait for a few seconds before the next poll/discovery attempt
        thread::sleep(Duration::from_secs(3));
    }
}