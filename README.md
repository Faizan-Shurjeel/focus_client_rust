# Physical "Focus Mode" Trigger - Rust Client & ESP32 Firmware

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![C++](https://img.shields.io/badge/platform-ESP32%20(Arduino)-red.svg)](https://www.arduino.cc/)
[![Status](https://img.shields.io/badge/status-automation%20active-brightgreen.svg)](https://github.com/Faizan-Shurjeel/focus_client_rust)

This repository contains the complete ecosystem for a physical "Do Not Disturb" totem: a high-performance **Rust client** for desktop automation and the advanced, dual-core **ESP32 firmware** that powers the physical device, complete with a web monitoring dashboard.

## The Concept 🧘

The idea is to bridge the physical and digital worlds to create a powerful ritual for deep work.

1.  **Place the Totem:** A custom ESP32 device is placed on your desk and powered on.
2.  **Enter the Zone:** The Rust client on your laptop detects the totem and automatically triggers a "focus mode"—**changing your desktop wallpaper and launching your designated work applications.**
3.  **Return to Normal:** When the ESP32 is powered off, the client detects its absence and instantly **reverts your wallpaper and closes all the applications it opened.**

## Project Architecture

The system is composed of two main components that work in harmony:

### ESP32 Totem (`Multithreaded_Dashboard.ino`)

The brain of the physical device. This is a robust, multi-threaded application built on the **FreeRTOS** real-time operating system.
*   **Dual-Core Operation:** The ESP32's two cores are used for maximum stability (Core 0 for Wi-Fi monitoring, Core 1 for the web server).
*   **Web Dashboard:** The totem hosts a beautiful **Material Design 3** web dashboard for real-time status monitoring at `http://focus-totem.local`.
*   **JSON API:** A `/api/status` endpoint serves up-to-date system stats for the dashboard.
*   **mDNS Discovery:** Announces itself on the network so no static IP is needed.

### Rust Client (`src/main.rs`)

A lightweight, highly reliable async background application that runs on your desktop.
*   **Automatic Discovery:** Uses mDNS to find the `focus-totem` on the network.
*   **Async Network Polling:** Powered by `tokio`, gracefully monitoring the totem's `/status` endpoint with a built-in "strike" system for network jitter.
*   **Cross-Platform Automation Engine (Windows + Linux):** Based on the totem's state, it executes powerful workflows:
    *   **Application Control:** Just-in-Time (JIT) loading of app commands/paths from `apps.toml` on activation, then reuses that active-session list when focus mode is deactivated.
    *   **Wallpaper Management:** Changes and restores the desktop wallpaper.
    *   **Session Logging:** Records every completed focus session to a local JSON file for future analytics/reporting.
    *   **Graceful Shutdown:** Intercepts `Ctrl+C` to ensure your wallpaper is restored and session is saved correctly before exiting.

## Key Features Implemented

| ESP32 Firmware | Rust Desktop Client |
| :--- | :--- |
| ✅ Dual-Core Operation (FreeRTOS) | ✅ Automatic mDNS Device Discovery |
| ✅ Material Design 3 Web Dashboard | ✅ Real-time State Tracking |
| ✅ Real-time JSON API for status | ✅ **Dynamic Wallpaper Changing** |
| ✅ Backward-compatible `/status` endpoint | ✅ **Application Launching & Closing** |
| | ✅ Cross-platform JIT app config via `apps.toml` |
| | ✅ Local JSON session logging |
| | ✅ Pure Rust analytics via `--analytics` |
| | ✅ Styled HTML Focus Health Report via `--report` |
| | ✅ Fully async with `tokio` |
| | ✅ Graceful shutdown and state restoration |

## Setup and Usage

### 1. Program the ESP32 Totem

*   **Prerequisites:**
    *   [Arduino IDE](https://www.arduino.cc/en/software) with ESP32 board support.
    *   **`ArduinoJson` Library:** In the Arduino IDE, go to `Tools > Manage Libraries...` and install the library by Benoit Blanchon.
*   **Instructions:**
    1.  Open `Multithreaded_Dashboard.ino` in the Arduino IDE.
    2.  Change the `ssid` and `password` variables to your Wi-Fi credentials.
    3.  Connect your ESP32, select the correct Board and COM Port, and click **Upload**.

### 2. Prepare and Run the Rust Client

The Rust client supports two compile-time runtime modes controlled by the `DEV_MODE` constant in `src/main.rs`:

- **Development / Mock Mode:** Enabled automatically in debug builds (`cargo run`). The client skips mDNS and polls the local Rust mock server at `http://localhost:8080/status`.
- **Production / Real ESP32 Mode:** Enabled automatically in release builds (`cargo run --release` or `cargo build --release`). The client uses mDNS discovery to find the physical `focus-totem` device on your network.

On Windows, if you need stronger process-control behavior for protected apps, run the release executable as Administrator.

*   **Prerequisites:**
    *   Rust Toolchain (via [rustup](https://www.rust-lang.org/tools/install)).
    *   A focus wallpaper image named `focus_wallpaper.jpg` placed in the project's root folder.

*   **Configuration (`apps.toml`):**
    1.  Open `apps.toml` in a text editor.
    2.  Define app lists under the `[apps]` table using OS keys such as `windows` and `linux`.
    3.  Put full executable paths for Windows apps, and command names (or full paths) for Linux apps.
    4.  Example:
        ```focus_client_rust/apps.toml#L1-10
[apps]
windows = [
  "C:\\Windows\\System32\\notepad.exe",
  "C:\\Users\\Faizy\\AppData\\Local\\BraveSoftware\\Brave-Browser\\Application\\brave.exe"
]

linux = [
  "brave-browser"
]
        ```

*   **Development / Mock Environment (no ESP32 required):**
    1.  Start the Rust mock ESP32 server in terminal 1:
        ```focus_client_rust/README.md#L1-1
cargo run -p mock_server
        ```
    2.  Start the desktop client in terminal 2:
        ```focus_client_rust/README.md#L1-1
cargo run
        ```
    3.  In debug mode, the client prints a development banner and polls:
        - `http://localhost:8080/status`
        - Expected response: `FOCUS_ON` or `FOCUS_OFF`
    4.  Toggle focus on/off without stopping the mock server:
        ```focus_client_rust/README.md#L1-1
curl http://localhost:8080/toggle
        ```
    5.  Stop the mock server (`Ctrl+C`) to simulate device disconnect and verify disconnect behavior.
    6.  On GNOME-based Linux distros (like Zorin), wallpaper switching uses `gsettings` first, then falls back to the `wallpaper` crate for better compatibility.

*   **Production / Real ESP32 Mode:**
    1.  Flash and power on the ESP32 firmware first so it advertises `focus-totem` on your network.
    2.  Run the client in release mode:
        ```focus_client_rust/README.md#L1-1
cargo run --release
        ```
    3.  Or build the release binary and run it directly on Linux:
        ```focus_client_rust/README.md#L1-2
cargo build --release
./target/release/focus_client_rust
        ```
    4.  On Windows, build release and run the generated executable, preferably as Administrator if app termination needs elevated permissions:
        ```focus_client_rust/README.md#L1-2
cargo build --release
target\release\focus_client_rust.exe
        ```
    5.  In release mode, the client does **not** use the mock server. It uses normal mDNS discovery (`focus-totem`) and polls the real ESP32 `/status` endpoint.

*   **Session Logs:**
    - A completed focus session is recorded when focus mode deactivates.
    - The client prints the session log file path on first write.
    - Linux path: `~/.local/share/focus_totem/sessions.json`
    - Windows path: `%APPDATA%\FocusTotem\sessions.json`
    - The file is a JSON array, ready for analytics/report generation later.

*   **Analytics & Report Generation:**
    - Run terminal analytics without launching apps, changing wallpaper, or polling the ESP32/mock server:
        ```focus_client_rust/README.md#L1-1
cargo run -- --analytics
        ```
    - Generate and open the styled HTML Focus Health Report:
        ```focus_client_rust/README.md#L1-1
cargo run -- --report
        ```
    - Show available CLI commands:
        ```focus_client_rust/README.md#L1-1
cargo run -- h
        ```
    - When using dash-prefixed app flags through Cargo, put `--` before the app flag, e.g. `cargo run --release -- --a`. Without that separator, Cargo consumes the flag before the client sees it.
    - Layer 1 statistical aggregation runs with 1+ session.
    - Trend detection activates after 7+ calendar days.
    - Decision-tree quality prediction activates after 30+ sessions.

## Project Files

*   `Multithreaded_Dashboard.ino`: **The main, recommended firmware for the ESP32.**
*   `src/main.rs`: Async entry point, CLI flag dispatch, polling state machine, and graceful shutdown coordination.
*   `src/config.rs`: Just-in-Time loading of cross-platform focus app configuration from `apps.toml`.
*   `src/automation.rs`: Wallpaper, application launch, and application termination automation.
*   `src/discovery.rs`: mDNS discovery for the physical Focus Totem device.
*   `src/totem.rs`: HTTP state polling and `FOCUS_ON`/`FOCUS_OFF` response parsing.
*   `src/session.rs`: Focus session model and atomic JSON session logging.
*   `src/analytics.rs`: Pure Rust analytics pipeline for session stats, trend detection, and quality prediction.
*   `src/report.rs`: Monolithic Precision HTML Focus Health Report generator and browser launcher.
*   `apps.toml`: Cross-platform app configuration loaded at runtime (`[apps].windows`, `[apps].linux`, etc.).
*   `mock_server/`: Rust mock ESP32 server crate with `GET /status` and `GET /toggle` endpoints.
*   `build.rs` & `manifest.xml`: Windows-specific build integration (manifest embedding only applies to Windows targets).
*   `totem.cpp`: The original, single-threaded ESP32 code (for historical reference).

## Roadmap & Next Steps

With the core automation in place, the next major goal is full system integration.

-   [x] ~~Programmatically change the desktop wallpaper.~~ (Done!)
-   [x] ~~Automatically launch and close specific applications.~~ (Done!)
-   [ ] **System-wide "Do Not Disturb":** Integrate with Windows 11's Focus Assist by modifying the registry (now possible with admin rights).
-   [ ] **Package Client as a Background Service:** Create a true background process that starts automatically with Windows.
-   [x] **Create a Configuration File:** App paths/commands are now loaded from `apps.toml` for easier cross-platform editing.
-   [x] **Session Logging Foundation:** Completed focus sessions are written to `sessions.json` for future analytics.
-   [x] **AI Analytics Foundation:** `--analytics` runs statistical aggregation now, with ML layers gated by data volume.
-   [x] **Focus Health Report:** `--report` generates and opens the styled HTML report.

---
_This project is now Rust-first: the desktop client and mock ESP32 server both live in this workspace._
