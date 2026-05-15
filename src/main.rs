mod analytics;
mod automation;
mod config;
mod discovery;
mod report;
mod session;
mod totem;

use analytics::{format_analytics, run_analytics};
use automation::{activate_focus_mode, deactivate_focus_mode};
use chrono::{DateTime, Local};
use config::load_focus_apps;
use discovery::discover_device;
use reqwest::Client;
use session::{append_session, load_sessions, sessions_file_path, SessionRecord};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::Notify;
use totem::{fetch_state as poll_totem_state, TotemState};

#[cfg(debug_assertions)]
const DEV_MODE: bool = false; // Enable for local development
#[cfg(not(debug_assertions))]
const DEV_MODE: bool = false; // Disable for release/production

const MOCK_STATUS_ENDPOINT: &str = "http://localhost:8080/status";
const HTTP_TIMEOUT: Duration = Duration::from_secs(2);
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(5);
const DISCOVERY_RETRY_DELAY: Duration = Duration::from_secs(4);
const POLL_INTERVAL: Duration = Duration::from_secs(3);
const MAX_FAILED_PINGS: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CliCommand {
    Run,
    Analytics,
    Report,
    Help,
}

struct FocusClient {
    http_client: Client,
    is_focused: bool,
    esp32_address: Option<String>,
    session_start: Option<DateTime<Local>>,
    failed_pings: u8,
    active_focus_apps: Vec<String>,
}

impl FocusClient {
    fn new(http_client: Client) -> Self {
        Self {
            http_client,
            is_focused: false,
            esp32_address: None,
            session_start: None,
            failed_pings: 0,
            active_focus_apps: Vec::new(),
        }
    }

    async fn tick(&mut self, shutdown_notify: &Notify) {
        let Some(state) = self.resolve_and_fetch_state(shutdown_notify).await else {
            return;
        };

        self.react_to_state(state).await;
    }

    async fn resolve_and_fetch_state(&mut self, shutdown_notify: &Notify) -> Option<TotemState> {
        if self.esp32_address.is_none() {
            if DEV_MODE {
                self.esp32_address = Some(MOCK_STATUS_ENDPOINT.to_string());
                println!("[MOCK] Simulated ESP32 address: {}", MOCK_STATUS_ENDPOINT);
            } else {
                println!("Searching for Focus Totem on the network...");
                let discovery_task =
                    tokio::task::spawn_blocking(|| discover_device(DISCOVERY_TIMEOUT));
                match wait_for_discovery(discovery_task, shutdown_notify).await {
                    Some(found_address) => self.esp32_address = Some(found_address),
                    None => return None,
                }
            }
        }

        self.fetch_state(shutdown_notify).await
    }

    async fn fetch_state(&mut self, shutdown_notify: &Notify) -> Option<TotemState> {
        let address = self.esp32_address.as_ref()?;

        let poll_result = tokio::select! {
            result = poll_totem_state(&self.http_client, address) => result,
            _ = shutdown_notify.notified() => return None,
        };

        match poll_result {
            Ok(state) => {
                self.failed_pings = 0;
                Some(state)
            }
            Err(e) => self.register_failed_ping(e.to_string()),
        }
    }

    fn register_failed_ping(&mut self, reason: String) -> Option<TotemState> {
        self.failed_pings = self.failed_pings.saturating_add(1);

        if self.failed_pings >= MAX_FAILED_PINGS {
            if DEV_MODE {
                eprintln!(
                    "[MOCK] Error polling mock ESP32 after {} strike(s): {}",
                    self.failed_pings, reason
                );
            } else {
                println!(
                    "Lost connection to device after {} strike(s): {}. Returning to search mode.",
                    self.failed_pings, reason
                );
            }

            self.failed_pings = 0;
            self.esp32_address = None;
            Some(TotemState::Error)
        } else {
            println!(
                "Polling strike {}/{}: {}. Holding current focus state.",
                self.failed_pings, MAX_FAILED_PINGS, reason
            );
            Some(if self.is_focused {
                TotemState::FocusOn
            } else {
                TotemState::FocusOff
            })
        }
    }

    async fn react_to_state(&mut self, state: TotemState) {
        if state == TotemState::FocusOn && !self.is_focused {
            self.is_focused = true;
            print_focus_transition(true, state);
            begin_focus_session(&mut self.session_start);
            self.active_focus_apps = activate_with_jit_config().await;
        } else if state != TotemState::FocusOn && self.is_focused {
            self.is_focused = false;
            print_focus_transition(false, state);
            let focus_apps = self.apps_to_deactivate().await;
            deactivate_with_focus_apps(focus_apps).await;
            end_focus_session(&mut self.session_start);
        }
    }

    async fn shutdown_gracefully(&mut self) {
        println!("Shutting down gracefully...");
        if self.is_focused {
            self.is_focused = false;
            let focus_apps = self.apps_to_deactivate().await;
            deactivate_with_focus_apps(focus_apps).await;
            end_focus_session(&mut self.session_start);
        }
    }

    async fn apps_to_deactivate(&mut self) -> Vec<String> {
        if self.active_focus_apps.is_empty() {
            load_focus_apps_async().await
        } else {
            std::mem::take(&mut self.active_focus_apps)
        }
    }
}

async fn wait_for_discovery(
    discovery_task: tokio::task::JoinHandle<Option<String>>,
    shutdown_notify: &Notify,
) -> Option<String> {
    tokio::select! {
        result = discovery_task => match result {
            Ok(Some(found_address)) => Some(found_address),
            Ok(None) => {
                println!("Device not found. Will retry in {} seconds.", DISCOVERY_RETRY_DELAY.as_secs());
                None
            }
            Err(e) => {
                eprintln!("ERROR: Device discovery task failed: {}", e);
                None
            }
        },
        _ = shutdown_notify.notified() => None,
    }
}

fn print_focus_transition(activated: bool, state: TotemState) {
    let action = if activated {
        "ACTIVATED"
    } else {
        "DEACTIVATED"
    };
    let suffix = if state == TotemState::Error {
        " (ERROR)"
    } else {
        ""
    };

    if DEV_MODE {
        println!("[MOCK] --- FOCUS MODE {}{} ---", action, suffix);
    } else {
        println!("--- FOCUS MODE {}{} ---", action, suffix);
    }
}

async fn activate_with_jit_config() -> Vec<String> {
    let focus_apps = load_focus_apps_async().await;
    let apps_for_activation = focus_apps.clone();
    if let Err(e) =
        tokio::task::spawn_blocking(move || activate_focus_mode(&apps_for_activation)).await
    {
        eprintln!("ERROR: Focus activation task failed: {}", e);
    }
    focus_apps
}

async fn deactivate_with_focus_apps(focus_apps: Vec<String>) {
    if let Err(e) = tokio::task::spawn_blocking(move || deactivate_focus_mode(&focus_apps)).await {
        eprintln!("ERROR: Focus deactivation task failed: {}", e);
    }
}

async fn load_focus_apps_async() -> Vec<String> {
    match tokio::task::spawn_blocking(load_focus_apps).await {
        Ok(apps) => apps,
        Err(e) => {
            eprintln!("ERROR: Focus app config loading task failed: {}", e);
            Vec::new()
        }
    }
}

fn begin_focus_session(session_start: &mut Option<DateTime<Local>>) {
    let start_time = Local::now();
    println!(
        "Focus session started at {}",
        start_time.format("%Y-%m-%d %H:%M:%S")
    );
    *session_start = Some(start_time);
}

fn end_focus_session(session_start: &mut Option<DateTime<Local>>) {
    let Some(start_time) = session_start.take() else {
        eprintln!("Session logging skipped: no active session start time was recorded.");
        return;
    };

    let end_time = Local::now();
    let record = SessionRecord::new(start_time, end_time);
    let duration_minutes = record.duration_minutes;

    match append_session(record) {
        Ok(path) => println!(
            "Focus session logged: {:.2} minute(s) -> {}",
            duration_minutes,
            path.display()
        ),
        Err(e) => eprintln!("ERROR: Failed to write focus session log: {}", e),
    }
}

fn run_analytics_cli() {
    let path = sessions_file_path();
    match load_sessions() {
        Ok(sessions) => {
            println!(
                "Loaded {} session(s) from {}",
                sessions.len(),
                path.display()
            );
            let result = run_analytics(&sessions);
            println!("{}", format_analytics(&result));
        }
        Err(e) => eprintln!(
            "ERROR: Failed to load session log '{}': {}",
            path.display(),
            e
        ),
    }
}

fn print_usage() {
    println!("Focus Totem client");
    println!();
    println!("Usage:");
    println!("  focus_client_rust                  Run the async Focus Totem client");
    println!("  focus_client_rust analytics        Print terminal analytics from sessions.json");
    println!("  focus_client_rust report           Generate and open the HTML Focus Health Report");
    println!("  focus_client_rust help             Show this help text");
    println!();
    println!("Aliases:");
    println!("  analytics: a, --analytics, -a, --a");
    println!("  report:    r, --report, -r, --r");
    println!("  help:      h, --help, -h, --h");
    println!();
    println!("Cargo examples:");
    println!("  cargo run -- analytics");
    println!("  cargo run --release -- report");
    println!("  cargo run -- h");
}

fn parse_cli_command(args: &[String]) -> Result<CliCommand, String> {
    let command_args = &args[1..];

    match command_args {
        [] => Ok(CliCommand::Run),
        [arg] => match arg.as_str() {
            "analytics" | "a" | "--analytics" | "-a" | "--a" => Ok(CliCommand::Analytics),
            "report" | "r" | "--report" | "-r" | "--r" => Ok(CliCommand::Report),
            "help" | "h" | "--help" | "-h" | "--h" => Ok(CliCommand::Help),
            other => Err(format!("Unknown command or flag '{other}'.")),
        },
        _ => Err(format!(
            "Expected at most one command argument, got {}.",
            command_args.len()
        )),
    }
}

fn run_report_cli() {
    let path = sessions_file_path();
    match load_sessions() {
        Ok(sessions) => {
            println!(
                "Loaded {} session(s) from {}",
                sessions.len(),
                path.display()
            );
            let result = run_analytics(&sessions);
            let html = report::generate_report(&result);
            match report::open_in_browser(&html) {
                Ok(report_path) => println!("Focus report generated: {}", report_path.display()),
                Err(e) => eprintln!("ERROR: Failed to generate focus report: {}", e),
            }
        }
        Err(e) => eprintln!(
            "ERROR: Failed to load session log '{}': {}",
            path.display(),
            e
        ),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cli_command = match parse_cli_command(&args) {
        Ok(command) => command,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            print_usage();
            return;
        }
    };

    match cli_command {
        CliCommand::Help => {
            print_usage();
            return;
        }
        CliCommand::Analytics => {
            run_analytics_cli();
            return;
        }
        CliCommand::Report => {
            run_report_cli();
            return;
        }
        CliCommand::Run => {}
    }

    let http_client = Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    let mut client = FocusClient::new(http_client);
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let shutdown_notify = Arc::new(Notify::new());

    install_shutdown_listener(shutdown_requested.clone(), shutdown_notify.clone());

    println!("Starting Focus Mode client (Rust version)...");
    println!("Detected OS: {}", std::env::consts::OS);

    if DEV_MODE {
        println!("==========================================");
        println!("=== DEVELOPMENT MODE ACTIVE (MOCK ESP32) ===");
        println!("Mock ESP32 endpoint: {}", MOCK_STATUS_ENDPOINT);
        println!("To exit mock mode, build release or set DEV_MODE to false.");
        println!("==========================================");
    }

    while !shutdown_requested.load(Ordering::SeqCst) {
        client.tick(&shutdown_notify).await;

        if shutdown_requested.load(Ordering::SeqCst) {
            break;
        }

        tokio::select! {
            _ = tokio::time::sleep(POLL_INTERVAL) => {},
            _ = shutdown_notify.notified() => break,
        }
    }

    client.shutdown_gracefully().await;
}

fn install_shutdown_listener(shutdown_requested: Arc<AtomicBool>, shutdown_notify: Arc<Notify>) {
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                shutdown_requested.store(true, Ordering::SeqCst);
                shutdown_notify.notify_one();
            }
            Err(e) => eprintln!("ERROR: Failed to listen for Ctrl+C: {}", e),
        }
    });
}
