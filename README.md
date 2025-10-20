# Physical "Focus Mode" Trigger - Rust Client & ESP32 Firmware

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![C++](https://img.shields.io/badge/platform-ESP32%20(Arduino)-red.svg)](https://www.arduino.cc/)
[![Status](https://img.shields.io/badge/status-milestone%20reached-brightgreen.svg)](https://github.com/Faizan-Shurjeel/focus_client_rust)

This repository contains the complete ecosystem for a physical "Do Not Disturb" totem: a high-performance **Rust client** for desktop automation and the advanced, dual-core **ESP32 firmware** that powers the physical device, complete with a web monitoring dashboard.

## The Concept 🧘

The idea is to bridge the physical and digital worlds to create a powerful ritual for deep work.

1.  **Place the Totem:** A custom ESP32 device is placed on your desk and powered on.
2.  **Enter the Zone:** The Rust client on your laptop detects the totem and automatically triggers a "focus mode"—in this version, it changes your desktop wallpaper to a minimal design.
3.  **Return to Normal:** When the ESP32 is powered off, the client detects its absence and instantly reverts your wallpaper.

## Project Architecture

The system is composed of two main components that work in harmony:

### ESP32 Totem (`Multithreaded_Dashboard.ino`)

The brain of the physical device. This is no simple script; it's a robust, multi-threaded application built on the **FreeRTOS** real-time operating system.
*   **Dual-Core Operation:** The ESP32's two cores are used for maximum stability.
    *   **Core 0:** Runs a background task to reliably monitor Wi-Fi status.
    *   **Core 1:** Runs the main web server, ensuring the user interface is always responsive.
*   **Web Dashboard:** The totem hosts a beautiful and modern **Material Design 3** web dashboard for real-time status monitoring.
*   **JSON API:** A `/api/status` endpoint serves up-to-date system stats as a JSON object, used by the web dashboard.
*   **mDNS Discovery:** Announces itself on the network as `focus-totem.local` so no static IP is needed.

### Rust Client (`src/main.rs`)

A lightweight, highly reliable background application that runs on your desktop.
*   **Automatic Discovery:** Uses mDNS to find the `focus-totem` on the network automatically.
*   **State Tracking:** Polls the totem's simple `/status` endpoint to track if it is online or offline.
*   **Automation Trigger:** Based on the totem's state, it executes workflows on the host computer. Currently, it manages changing and restoring the Windows desktop wallpaper.

## Key Features Implemented

| ESP32 Firmware | Rust Desktop Client |
| :--- | :--- |
| ✅ Dual-Core Operation (FreeRTOS) | ✅ Automatic mDNS Device Discovery |
| ✅ Material Design 3 Web Dashboard | ✅ Real-time State Tracking |
| ✅ Real-time JSON API for status | ✅ **Dynamic Wallpaper Changing** |
| ✅ Backward-compatible `/status` endpoint | |

## The Web Dashboard

This is a major feature that turns the ESP32 into a professional IoT device. Once the totem is running, you can access the dashboard from any device on the same network.

*   **URL:** `http://focus-totem.local`
*   **Features:** Provides a real-time view of Wi-Fi status, IP address, signal strength (RSSI), uptime, free memory, and which core is running which task.



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

*   **Prerequisites:**
    *   Rust Toolchain (via [rustup](https://www.rust-lang.org/tools/install)).
    *   A focus wallpaper image named `focus_wallpaper.jpg` placed in the root of the project folder.
*   **Instructions:**
    1.  Clone this repository and `cd` into it.
    2.  Run the application with `cargo run`. For best performance, use release mode:
        ```bash
        cargo run --release
        ```

## Project Files

*   `Multithreaded_Dashboard.ino`: **The main, recommended firmware for the ESP32.**
*   `src/main.rs`: The source code for the Rust desktop client.
*   `totem.cpp`: The original, single-threaded ESP32 code. Kept for historical reference.
*   `focus_wallpaper.jpg`: An example focus wallpaper. Replace with your own.

## Roadmap & Next Steps

With the core system stable and the dashboard complete, the next steps focus on expanding the automation workflow.

-   [x] ~~Programmatically change the desktop wallpaper.~~ (Done!)
-   [ ] **Application Control:** Automatically launch and close specific applications (e.g., VS Code, Obsidian) when focus mode starts/stops.
-   [ ] **System-wide "Do Not Disturb":** Integrate with Windows 11's Focus Assist.
-   [ ] **Package Client as a Background Service:** Create a true background process that starts automatically with Windows.
-   [ ] **Enhance Totem with Visual Feedback:** Add an RGB LED to the ESP32 for at-a-glance status indication.

---
_This project is being developed in parallel with a [Python version](https'://github.com/Faizan-Shurjeel/focus_client_python) to compare language ergonomics and performance._