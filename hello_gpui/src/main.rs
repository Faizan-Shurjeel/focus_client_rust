// src/main.rs

use gpui::{
    prelude::*, App, Application, Bounds, Context, Entity, FontWeight, Render, Window,
    WindowBounds, WindowOptions, div, px, rgb, size,
};
use lazy_static::lazy_static;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use reqwest::blocking::Client; // Now correctly imported
use smol::Timer;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

// --- CONFIGURATION ---
const FOCUS_APPS: &[&str] = &[
    "C:\\Windows\\System32\\notepad.exe",
    "C:\\Users\\Faizy\\AppData\\Local\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
];
const SERVICE_NAME: &str = "_http._tcp.local.";
const DEVICE_HOSTNAME: &str = "focus-totem";
const FOCUS_WALLPAPER_NAME: &str = "focus_wallpaper.jpg";

// --- GLOBAL STATE ---
lazy_static! {
    static ref ORIGINAL_WALLPAPER_PATH: Mutex<Option<String>> = Mutex::new(None);
}

// --- STATE DEFINITION ---
#[derive(Clone, PartialEq)]
enum AppStatus {
    Searching,
    Connected(String),
    FocusActive(String),
    ConnectionLost,
}

// --- THE GPUI VIEW ---
struct FocusClientUI {
    status: AppStatus,
}

impl FocusClientUI {
    fn new() -> Self {
        Self { status: AppStatus::Searching }
    }
}

impl Render for FocusClientUI {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let (bg_color, text, status_text) = match &self.status {
            AppStatus::Searching => (rgb(0x3B82F6), "SEARCHING", "Looking for Focus Totem..."),
            AppStatus::Connected(_) => (rgb(0x22C55E), "CONNECTED", "Totem found. Ready to focus."),
            AppStatus::FocusActive(_) => (rgb(0x2e7d32), "FOCUS ACTIVE", "Deep work in session."),
            AppStatus::ConnectionLost => (rgb(0xEF4444), "DISCONNECTED", "Lost connection to Totem."),
        };

        div()
            .flex().flex_col().bg(bg_color).size_full()
            .justify_center().items_center().gap_4()
            .text_color(rgb(0xffffff))
            .child(
                div().text_2xl().font_weight(FontWeight::BOLD).child(text)
            )
            .child(div().text_lg().child(status_text))
    }
}

// --- APPLICATION ENTRY POINT ---
fn main() {
    Application::new().run(|cx: &mut App| {
        let window_options = WindowOptions {
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None, size(px(500.), px(250.)), cx,
            ))),
            ..Default::default()
        };

        cx.open_window(window_options, |_, cx| {
            let view = cx.new(|_| FocusClientUI::new());
            let view_handle = view.clone();

            cx.spawn(|cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    run_network_logic(view_handle, &mut cx).await;
                }
            })
            .detach();

            view
        })
        .unwrap();
    });
}

// --- BACKGROUND LOGIC ---
async fn run_network_logic(view: Entity<FocusClientUI>, cx: &mut gpui::AsyncApp) {
    let http_client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("Failed to build HTTP client");
    let mut esp32_address: Option<String> = None;

    loop {
        if esp32_address.is_none() {
            update_status(&view, AppStatus::Searching, cx);
            
            // THIS IS THE FIX: Wrap the blocking call in an `async move` block to create a Future.
            let found_address = cx
                .background_spawn(async move { discover_device(Duration::from_secs(5)) })
                .await;

            if let Some(address) = found_address {
                esp32_address = Some(address.clone());
                update_status(&view, AppStatus::Connected(address), cx);
            } else {
                Timer::after(Duration::from_secs(4)).await;
            }
        }

        if let Some(address) = &esp32_address {
            let client = http_client.clone();
            let address_clone = address.clone();

            // THIS IS THE FIX: Wrap the blocking call in an `async move` block to create a Future.
            let result = cx
                .background_spawn(async move { client.get(address_clone).send() })
                .await;

            match result {
                Ok(response) => {
                    if response.status().is_success() {
                        // The .text() call can also fail, so handle that Result too.
                        if let Ok(text) = response.text() {
                             if text == "FOCUS_ON" {
                                let is_active = cx
                                    .read_entity(&view, |ui, _| ui.status == AppStatus::FocusActive(address.clone()))
                                    .unwrap_or(false);

                                if !is_active {
                                    update_status(&view, AppStatus::FocusActive(address.clone()), cx);
                                    activate_focus_mode();
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    let was_active = cx
                        .read_entity(&view, |ui, _| matches!(ui.status, AppStatus::FocusActive(_)))
                        .unwrap_or(false);

                    if was_active {
                        deactivate_focus_mode();
                    }
                    update_status(&view, AppStatus::ConnectionLost, cx);
                    esp32_address = None;
                }
            }
        }
        Timer::after(Duration::from_secs(3)).await;
    }
}

// --- UI UPDATE HELPER ---
fn update_status(view: &Entity<FocusClientUI>, new_status: AppStatus, cx: &mut gpui::AsyncApp) {
    let _ = cx.update_entity(view, |ui, cx| {
        ui.status = new_status;
        cx.notify();
    });
}

// --- ORIGINAL FUNCTIONS (UNCHANGED) ---
fn discover_device(search_duration: Duration) -> Option<String> {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let receiver = mdns.browse(SERVICE_NAME).expect("Failed to browse for service");
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < search_duration {
        if let Ok(event) = receiver.recv_timeout(Duration::from_secs(1)) {
            if let ServiceEvent::ServiceResolved(info) = event {
                if info.get_fullname().contains(DEVICE_HOSTNAME) {
                    if let Some(ip) = info.get_addresses().iter().next() {
                        let port = info.get_port();
                        let url = format!("http://{}:{}/status", ip, port);
                        return Some(url);
                    }
                }
            }
        }
    }
    None
}

fn activate_focus_mode() {
    println!("Activating focus mode automations...");
    if let Ok(path) = wallpaper::get() {
        *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(path);
    }
    if let Ok(absolute_path) = std::fs::canonicalize(FOCUS_WALLPAPER_NAME) {
        let _ = wallpaper::set_from_path(absolute_path.to_str().unwrap());
    }
    for app_path in FOCUS_APPS {
        let _ = Command::new(app_path).spawn();
    }
}

fn deactivate_focus_mode() {
    println!("Deactivating focus mode automations...");
    let mut original_path = ORIGINAL_WALLPAPER_PATH.lock().unwrap();
    if let Some(path) = original_path.as_deref() {
        let _ = wallpaper::set_from_path(path);
    }
    *original_path = None;
    for app_path in FOCUS_APPS {
        if let Some(file_name) = Path::new(app_path).file_name().and_then(|s| s.to_str()) {
            let _ = Command::new("taskkill").args(["/F", "/IM", file_name]).output();
        }
    }
}