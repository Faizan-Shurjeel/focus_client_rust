use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::time::{Duration, Instant};

const SERVICE_NAME: &str = "_http._tcp.local.";
const DEVICE_HOSTNAME: &str = "focus-totem";

pub fn discover_device(search_duration: Duration) -> Option<String> {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let receiver = mdns
        .browse(SERVICE_NAME)
        .expect("Failed to browse for service");
    let start_time = Instant::now();

    while start_time.elapsed() < search_duration {
        if let Ok(ServiceEvent::ServiceResolved(info)) =
            receiver.recv_timeout(Duration::from_secs(1))
        {
            if info.get_fullname().contains(DEVICE_HOSTNAME) {
                let Some(ip) = info.get_addresses().iter().next() else {
                    continue;
                };
                let host = if ip.is_ipv6() {
                    format!("[{ip}]")
                } else {
                    ip.to_string()
                };
                let port = info.get_port();
                let url = format!("http://{host}:{port}/status");
                println!("Resolved Focus Totem address: {}", url);
                return Some(url);
            }
        }
    }

    None
}
