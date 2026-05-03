use axum::{extract::State, routing::get, Router};
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Clone)]
struct AppState {
    focus_on: Arc<AtomicBool>,
}

async fn status(State(state): State<AppState>) -> &'static str {
    if state.focus_on.load(Ordering::Relaxed) {
        "FOCUS_ON"
    } else {
        "FOCUS_OFF"
    }
}

async fn toggle(State(state): State<AppState>) -> &'static str {
    let previous = state.focus_on.fetch_xor(true, Ordering::Relaxed);
    if previous {
        "FOCUS_OFF"
    } else {
        "FOCUS_ON"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        focus_on: Arc::new(AtomicBool::new(true)),
    };

    let app = Router::new()
        .route("/status", get(status))
        .route("/toggle", get(toggle))
        .with_state(state);

    let address = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(address).await?;

    println!("[Mock ESP32] Running on http://{}", address);
    println!("[Mock ESP32] GET /status -> FOCUS_ON or FOCUS_OFF");
    println!("[Mock ESP32] GET /toggle -> flips focus state and returns the new value");
    println!("[Mock ESP32] Initial state: FOCUS_ON");
    println!("[Mock ESP32] Press Ctrl+C to stop.");

    axum::serve(listener, app).await?;

    Ok(())
}
