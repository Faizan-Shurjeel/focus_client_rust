# Physical "Focus Mode" Trigger - Rust Client

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-in%20development-yellow.svg)](https://github.com/Faizan-Shurjeel/focus_client_rust)

This repository contains the Rust client for a physical "Do Not Disturb" totem. This client runs in the background on your computer, detects the presence of a specific ESP32 device on your local network, and prepares to trigger a digital "focus mode" workflow.

## The Concept 🧘

The idea is simple yet powerful: turn the act of focusing into a physical ritual.

1.  **Place the Totem:** You place a custom ESP32 device on your desk and power it on.
2.  **Enter the Zone:** The Rust client on your laptop detects the device and automatically triggers a "focus mode"—muting notifications, launching work apps, changing your wallpaper, etc.
3.  **Return to Normal:** When you power the ESP32 off, the client detects its absence and reverses all the changes, bringing your digital environment back to normal.

This repository is one piece of the puzzle: the highly reliable, cross-platform client written in Rust.

## How It Works

*   **ESP32 Totem:** An ESP32 microcontroller is programmed to connect to Wi-Fi and announce itself on the local network using the **mDNS** protocol with the hostname `focus-totem`. It also runs a tiny web server.
*   **Rust Client (This Repo):** This application runs continuously in the background.
    1.  **Discovery:** It uses mDNS to automatically discover the IP address of the `focus-totem` without any static configuration.
    2.  **Polling:** Once found, it periodically sends an HTTP request to the ESP32's `/status` endpoint.
    3.  **State Change:**
        *   If it receives a `FOCUS_ON` response, it knows to activate the focus mode.
        *   If the connection fails (i.e., the ESP32 is off), it knows to deactivate the focus mode and returns to the discovery phase.

## Current Status

The core communication and discovery logic is complete and functional. The client can reliably track the presence of the ESP32 totem on the network. The next phase is to implement the actual focus mode automations for Windows 11.

## Getting Started

### Prerequisites

*   **Rust Toolchain:** Install Rust via [rustup](https://www.rust-lang.org/tools/install).
*   **An ESP32 "Totem":** You need an ESP32 device flashed with the corresponding server code.

### Running the Client

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Faizan-Shurjeel/focus_client_rust.git
    cd focus_client_rust
    ```

2.  **Run the application:**
    ```bash
    cargo run
    ```
    The client will start and immediately begin searching for the `focus-totem` on your network.

## Next Steps

The foundational communication layer is built. The next steps involve implementing the `// --- ACTION: Trigger Focus ON/OFF actions here ---` sections in `src/main.rs`:

-   [ ] **Windows 11 Integration:**
    -   [ ] Modify the registry to enable/disable system-wide "Do Not Disturb".
    -   [ ] Programmatically change the desktop wallpaper.
-   [ ] **Application Control:**
    -   [ ] Launch specific applications (VS Code, Obsidian, etc.).
    -   [ ] Mute notifications in apps like Slack and Discord via APIs or other methods.
-   [ ] **Cross-Device Control:**
    -   [ ] Send commands to an Android phone (via Tasker) to enable its "Do Not Disturb" mode.

---
_This project is being developed in parallel with [Python](<https://github.com/Faizan-Shurjeel/focus_client_python>) and [Go](<link-to-go-repo-if-you-create-one>) versions to compare language ergonomics and performance for this task._
