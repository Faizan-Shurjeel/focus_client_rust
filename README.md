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

A lightweight, highly reliable background application that runs on your desktop.
*   **Automatic Discovery:** Uses mDNS to find the `focus-totem` on the network.
*   **State Tracking:** Polls the totem's `/status` endpoint to track its online status.
*   **Cross-Platform Automation Engine (Windows + Linux):** Based on the totem's state, it executes powerful workflows:
    *   **Application Control:** Loads app commands/paths from `apps.toml`, launches the configured list, and closes them when focus mode is deactivated.
    *   **Wallpaper Management:** Changes and restores the desktop wallpaper.

## Key Features Implemented

| ESP32 Firmware | Rust Desktop Client |
| :--- | :--- |
| ✅ Dual-Core Operation (FreeRTOS) | ✅ Automatic mDNS Device Discovery |
| ✅ Material Design 3 Web Dashboard | ✅ Real-time State Tracking |
| ✅ Real-time JSON API for status | ✅ **Dynamic Wallpaper Changing** |
| ✅ Backward-compatible `/status` endpoint | ✅ **Application Launching & Closing** |
| | ✅ Cross-platform app config via `apps.toml` |

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

The Rust client supports two runtime modes:

- **Mock Mode (default in debug builds):** Uses a local mock endpoint at `http://localhost:8080/status` so you can test wallpaper/app automation without ESP32 hardware.
- **Real Mode (release builds):** Uses mDNS discovery to find the physical `focus-totem` device on your network.

On Windows, if you need stronger process-control behavior for protected apps, run the executable as Administrator.

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
  "gedit",
  "firefox"
]
        ```

*   **Build and Run (Mock Mode for ESP32-free development):**
    1.  Start the local mock server:
        ```focus_client_rust/README.md#L1-1
python3 mock_esp32.py
        ```
    2.  In another terminal, run the Rust client in debug mode:
        ```focus_client_rust/README.md#L1-1
cargo run
        ```
    3.  In debug mode, the client prints a development banner and polls:
        - `http://localhost:8080/status`
        - Expected response: `FOCUS_ON`
    4.  Stop the mock server (`Ctrl+C`) to simulate device disconnect and verify deactivation behavior.
    5.  On GNOME-based Linux distros (like Zorin), wallpaper switching uses the `wallpaper` crate first, then falls back to `gsettings` (`org.gnome.desktop.background`) for better compatibility.

*   **Build and Run (Real ESP32 mode):**
    1.  Build/run in release mode:
        ```focus_client_rust/README.md#L1-2
cargo build --release
cargo run --release
        ```
    2.  In this mode, the client uses normal mDNS discovery (`focus-totem`) and polls the real `/status` endpoint.

## Project Files

*   `Multithreaded_Dashboard.ino`: **The main, recommended firmware for the ESP32.**
*   `src/main.rs`: The source code for the Rust desktop client (includes debug mock-mode logic and release real-device logic).
*   `apps.toml`: Cross-platform app configuration loaded at runtime (`[apps].windows`, `[apps].linux`, etc.).
*   `mock_esp32.py`: Local mock server that simulates `GET /status -> FOCUS_ON`.
*   `build.rs` & `manifest.xml`: Windows-specific build integration (manifest embedding only applies to Windows targets).
*   `totem.cpp`: The original, single-threaded ESP32 code (for historical reference).

## Roadmap & Next Steps

With the core automation in place, the next major goal is full system integration.

-   [x] ~~Programmatically change the desktop wallpaper.~~ (Done!)
-   [x] ~~Automatically launch and close specific applications.~~ (Done!)
-   [ ] **System-wide "Do Not Disturb":** Integrate with Windows 11's Focus Assist by modifying the registry (now possible with admin rights).
-   [ ] **Package Client as a Background Service:** Create a true background process that starts automatically with Windows.
-   [x] **Create a Configuration File:** App paths/commands are now loaded from `apps.toml` for easier cross-platform editing.

---
_This project is being developed in parallel with a [Python version](https://github.com/Faizan-Shurjeel/focus_client_python) to compare language ergonomics and performance._
