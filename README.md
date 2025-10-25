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

A lightweight, highly reliable background application that runs on your desktop and requires administrator privileges to control other applications.
*   **Automatic Discovery:** Uses mDNS to find the `focus-totem` on the network.
*   **State Tracking:** Polls the totem's `/status` endpoint to track its online status.
*   **Automation Engine:** Based on the totem's state, it executes powerful workflows:
    *   **Application Control:** Launches a configurable list of applications and reliably terminates the entire process tree for each one on deactivation.
    *   **Wallpaper Management:** Changes and restores the desktop wallpaper.

## Key Features Implemented

| ESP32 Firmware | Rust Desktop Client |
| :--- | :--- |
| ✅ Dual-Core Operation (FreeRTOS) | ✅ Automatic mDNS Device Discovery |
| ✅ Material Design 3 Web Dashboard | ✅ Real-time State Tracking |
| ✅ Real-time JSON API for status | ✅ **Dynamic Wallpaper Changing** |
| ✅ Backward-compatible `/status` endpoint | ✅ **Application Launching & Closing** |
| | ✅ **Requires Admin Privileges** (for robust control) |

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

The Rust client now requires Administrator privileges to reliably terminate complex applications like web browsers. The project is configured to automatically request these permissions.

*   **Prerequisites:**
    *   Rust Toolchain (via [rustup](https://www.rust-lang.org/tools/install)).
    *   A focus wallpaper image named `focus_wallpaper.jpg` placed in the project's root folder.

*   **Configuration:**
    1.  Open `src/main.rs` in a text editor.
    2.  Find the `FOCUS_APPS` constant near the top of the file.
    3.  Edit the list of paths to point to the `.exe` files of the applications you want to manage.

*   **Build and Run:**
    1.  Build the release executable. This will embed the administrator manifest.
        ```bash
        cargo build --release
        ```
    2.  Navigate to the `target/release` folder in your file explorer.
    3.  **Right-click on `focus_client_rust.exe` and select "Run as administrator"**, or simply double-click it and approve the UAC (User Account Control) prompt.

## Project Files

*   `Multithreaded_Dashboard.ino`: **The main, recommended firmware for the ESP32.**
*   `src/main.rs`: The source code for the Rust desktop client.
*   `build.rs` & `manifest.xml`: Build scripts that embed a manifest into the `.exe`, ensuring it requests administrator privileges from Windows.
*   `totem.cpp`: The original, single-threaded ESP32 code (for historical reference).

## Roadmap & Next Steps

With the core automation in place, the next major goal is full system integration.

-   [x] ~~Programmatically change the desktop wallpaper.~~ (Done!)
-   [x] ~~Automatically launch and close specific applications.~~ (Done!)
-   [ ] **System-wide "Do Not Disturb":** Integrate with Windows 11's Focus Assist by modifying the registry (now possible with admin rights).
-   [ ] **Package Client as a Background Service:** Create a true background process that starts automatically with Windows.
-   [ ] **Create a Configuration File:** Move app paths out of the source code and into a `config.toml` file for easier editing.

---
_This project is being developed in parallel with a [Python version](https://github.com/Faizan-Shurjeel/focus_client_python) to compare language ergonomics and performance._
