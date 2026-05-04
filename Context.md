Project Path: focus_client_rust

Source Tree:

```txt
focus_client_rust
├── Cargo.toml
├── Multithreaded_Dashboard
│   ├── M3-Redesign.ino
│   ├── M3Design.md
│   ├── Multithreaded_Dashboard.ino
│   └── test.html
├── README.md
├── ROADMAP-new.md
├── ROADMAP.md
├── Updated.md
├── apps.toml
├── build.rs
├── focus_wallpaper.jpg
├── hello_gpui
│   ├── Cargo.toml
│   ├── src
│   │   └── main.rs
│   └── steps.txt
├── manifest.xml
├── mock_server
│   ├── Cargo.toml
│   └── src
│       └── main.rs
├── src
│   ├── analytics.rs
│   ├── main.rs
│   └── session.rs
└── totem.cpp

```

`Cargo.toml`:

```toml
[package]
name = "focus_client_rust"
version = "0.1.0"
edition = "2021"

[workspace]
members = [".", "mock_server"]
resolver = "2"

[dependencies]
lazy_static = "1.5.0"
mdns-sd = "0.15.1" # For mDNS service discovery
reqwest = { version = "0.12.23", features = ["blocking"] } # For making HTTP requests
wallpaper = "3.2.0"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1"
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"
linfa = "0.7"
linfa-linear = "0.7"
linfa-trees = "0.7"
ndarray = "0.15"

# --- ADD THIS ENTIRE SECTION ---
[build-dependencies]
embed-manifest = "1.4.0"

```

`Multithreaded_Dashboard/M3-Redesign.ino`:

```ino
#include <WiFi.h>
#include <ESPmDNS.h>
#include <WebServer.h>
#include <ArduinoJson.h>

// --- IMPORTANT: CHANGE THESE TO YOUR WI-FI CREDENTIALS ---
const char *ssid = "Faizy's A54";
const char *password = "12345678";

WebServer server(80);

// --- Shared Status Variables ---
struct Status
{
  bool wifiConnecting;
  bool wifiConnected;
};
Status sharedStatus;

// --- FreeRTOS Handles ---
TaskHandle_t statusMonitorTaskHandle = NULL;
SemaphoreHandle_t statusMutex;

// --- THE (REDESIGNED) HTML PAGE ---
const char *dashboardHTML = R"rawliteral(
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Focus Totem Dashboard</title>
  
  <!-- The Editorial Voice: Typography Base -->
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Epilogue:wght@700&family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
  
  <style>
    :root {
      /* Surface Hierarchy (Dark Mode Default) */
      --bg-void: #0b0e12;
      --surface-low: #1D2024;
      --surface-highest: #2B2D31;
      --tertiary-muted: #d1f3dc;
      --primary-accent: #bbdaff;
      
      /* Typography Colors */
      --text-primary: #ffffff;
      --text-secondary: #9ca3af;
    }

    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }

    body {
      font-family: 'Inter', sans-serif;
      background-color: var(--bg-void);
      color: var(--text-primary);
      -webkit-font-smoothing: antialiased;
      min-height: 100vh;
      display: flex;
      justify-content: center;
      padding: 64px 24px;
    }

    .monolith-wrapper {
      width: 100%;
      max-width: 800px;
      display: flex;
      flex-direction: column;
      gap: 56px; /* Massive breathing room per Do's and Don'ts */
    }

    /* Typography: Statement Styles */
    .display-lg {
      font-family: 'Epilogue', sans-serif;
      font-size: 3.5rem;
      font-weight: 700;
      letter-spacing: -0.02em;
      line-height: 1.1;
      color: var(--text-primary);
    }

    .headline-md {
      font-family: 'Epilogue', sans-serif;
      font-size: 1.5rem;
      font-weight: 700;
      letter-spacing: -0.02em;
      color: var(--text-primary);
    }

    /* Typography: High-Density Data Workhorse */
    .body-md {
      font-family: 'Inter', sans-serif;
      font-size: 1rem;
      color: var(--text-secondary);
      line-height: 1.5;
      margin-top: 16px;
    }

    .label {
      font-family: 'Inter', sans-serif;
      font-size: 1rem;
      font-weight: 500;
      color: var(--text-primary);
    }

    .value {
      font-family: 'Inter', sans-serif;
      font-size: 1rem;
      color: var(--text-secondary);
      display: flex;
      align-items: center;
    }

    .badge {
      background-color: var(--surface-highest);
      color: var(--primary-accent);
      padding: 6px 16px;
      border-radius: 9999px;
      font-size: 0.875rem;
      font-weight: 600;
      margin-left: 16px;
      vertical-align: middle;
    }

    /* Tonal Stepping: Level 0 -> Level 1 */
    .dashboard-surface {
      background-color: var(--surface-low);
      border-radius: 24px;
      padding: 40px;
      display: flex;
      flex-direction: column;
      gap: 32px;
    }

    /* The Divider Rule: No 1px lines */
    .list-container {
      display: flex;
      flex-direction: column;
      gap: 1.4rem; 
    }

    /* High-Contrast Touch Targets & Cards */
    .list-item {
      background-color: var(--surface-highest); /* Tonal shift milled inside the surface */
      border-radius: 16px;
      min-height: 56px; /* Guaranteed accessible hit area */
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 0 24px;
    }

    /* Buttons & Chips: Primary Actions */
    .btn-primary {
      background-color: var(--primary-accent);
      color: var(--bg-void); 
      border: none;
      border-radius: 9999px; /* Pill Shape */
      min-height: 56px; /* Strict 56px rule */
      padding: 0 40px;
      font-family: 'Inter', sans-serif;
      font-size: 1.125rem;
      font-weight: 600;
      cursor: pointer;
      transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
      display: inline-flex;
      align-items: center;
      justify-content: center;
    }

    .btn-primary:active {
      transform: scale(0.97);
    }

    /* Status Indicators */
    .status-indicator {
      display: inline-block;
      width: 12px;
      height: 12px;
      border-radius: 50%;
      margin-right: 12px;
    }

    .status-connected {
      background-color: var(--tertiary-muted);
    }

    .status-disconnected {
      background-color: #ff5e5e; /* Stark contrast error state */
    }

    .status-connecting {
      background-color: var(--primary-accent);
      animation: pulse 1.5s ease-in-out infinite;
    }

    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.2; }
    }

    .actions-wrapper {
      display: flex;
      justify-content: flex-start;
    }

    /* Form Factor Adaptations */
    @media (max-width: 600px) {
      body { padding: 32px 16px; }
      .display-lg { font-size: 2.75rem; }
      .dashboard-surface { padding: 24px; }
      .list-item {
        flex-direction: column;
        align-items: flex-start;
        justify-content: center;
        gap: 8px;
        padding: 16px 24px;
      }
    }
  </style>
</head>
<body>
  <div class="monolith-wrapper">
    
    <!-- The Hierarchy Rule Applied -->
    <header class="header">
      <h1 class="display-lg">Focus Totem</h1>
      <p class="body-md">Real-time status monitoring <span class="badge">Dual-Core</span></p>
    </header>

    <main class="dashboard-surface">
      <h2 class="headline-md">System Status</h2>
      
      <div class="list-container">
        <div class="list-item">
          <span class="label">WiFi Connection</span>
          <span class="value">
            <span class="status-indicator" id="wifi-indicator"></span>
            <span id="wifi-status">Loading...</span>
          </span>
        </div>

        <div class="list-item">
          <span class="label">Network Name (SSID)</span>
          <span class="value" id="ssid">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">IP Address</span>
          <span class="value" id="ip-address">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">mDNS Hostname</span>
          <span class="value" id="mdns">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">Signal Strength</span>
          <span class="value" id="rssi">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">Uptime</span>
          <span class="value" id="uptime">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">Free Heap Memory</span>
          <span class="value" id="heap">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">Web Server Core</span>
          <span class="value" id="web-core">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">Monitor Core</span>
          <span class="value" id="monitor-core">Loading...</span>
        </div>
      </div>
    </main>

    <div class="actions-wrapper">
      <button class="btn-primary" id="refresh-btn">Refresh Status</button>
    </div>

  </div>

  <script>
    async function updateStatus() {
      try {
        const response = await fetch('/api/status');
        if (!response.ok) {
            console.error("Failed to fetch status, server responded with:", response.status);
            return;
        }
        const data = await response.json();

        const wifiIndicator = document.getElementById('wifi-indicator');
        const wifiStatus = document.getElementById('wifi-status');

        if (data.wifiConnected) {
          wifiIndicator.className = 'status-indicator status-connected';
          wifiStatus.textContent = 'Connected';
        } else if (data.wifiConnecting) {
          wifiIndicator.className = 'status-indicator status-connecting';
          wifiStatus.textContent = 'Connecting...';
        } else {
          wifiIndicator.className = 'status-indicator status-disconnected';
          wifiStatus.textContent = 'Disconnected';
        }

        document.getElementById('ssid').textContent = data.ssid || 'N/A';
        document.getElementById('ip-address').textContent = data.ipAddress || 'N/A';
        document.getElementById('mdns').textContent = data.mdns || 'N/A';
        document.getElementById('rssi').textContent = data.rssi || 'N/A';
        document.getElementById('uptime').textContent = data.uptime || 'N/A';
        document.getElementById('heap').textContent = data.freeHeap || 'N/A';
        document.getElementById('web-core').textContent = data.webCore || 'N/A';
        document.getElementById('monitor-core').textContent = data.monitorCore || 'N/A';
      } catch (error) {
        console.error('Error fetching status:', error);
      }
    }

    document.getElementById('refresh-btn').addEventListener('click', updateStatus);

    // Initial load
    updateStatus();

    // Auto-refresh every 5 seconds
    setInterval(updateStatus, 5000);
  </script>
</body>
</html>
)rawliteral";

String formatUptime(unsigned long milliseconds)
{
  unsigned long seconds = milliseconds / 1000;
  unsigned long minutes = seconds / 60;
  unsigned long hours = minutes / 60;
  unsigned long days = hours / 24;
  seconds %= 60;
  minutes %= 60;
  hours %= 24;
  String uptime = "";
  if (days > 0)
    uptime += String(days) + "d ";
  if (hours > 0)
    uptime += String(hours) + "h ";
  if (minutes > 0)
    uptime += String(minutes) + "m ";
  uptime += String(seconds) + "s";
  return uptime;
}

void handleRoot()
{
  server.send(200, "text/html", dashboardHTML);
}

void handleApiStatus()
{
  StaticJsonDocument<256> doc;
  xSemaphoreTake(statusMutex, portMAX_DELAY);
  bool isConnecting = sharedStatus.wifiConnecting;
  bool isConnected = sharedStatus.wifiConnected;
  xSemaphoreGive(statusMutex);

  doc["wifiConnected"] = isConnected;
  doc["wifiConnecting"] = isConnecting;
  doc["ssid"] = ssid;
  doc["mdns"] = "focus-totem.local";
  doc["uptime"] = formatUptime(millis());
  doc["freeHeap"] = String(ESP.getFreeHeap()) + " bytes";

  if (isConnected)
  {
    doc["ipAddress"] = WiFi.localIP().toString();
    doc["rssi"] = String(WiFi.RSSI()) + " dBm";
  }
  else
  {
    doc["ipAddress"] = "N/A";
    doc["rssi"] = "N/A";
  }

  doc["webCore"] = "Core " + String(xPortGetCoreID());
  doc["monitorCore"] = "Core 0";

  String jsonResponse;
  serializeJson(doc, jsonResponse);

  server.sendHeader("Access-Control-Allow-Origin", "*");
  server.send(200, "application/json", jsonResponse);
}

void handleStatus()
{
  server.send(200, "text/plain", "FOCUS_ON");
}

void statusMonitorTask(void *parameter)
{
  for (;;)
  {
    bool isConnectedNow = (WiFi.status() == WL_CONNECTED);
    xSemaphoreTake(statusMutex, portMAX_DELAY);

    if (isConnectedNow && !sharedStatus.wifiConnected)
    {
      Serial.println("WiFi has connected!");
      sharedStatus.wifiConnected = true;
      sharedStatus.wifiConnecting = false;
    }
    else if (!isConnectedNow && sharedStatus.wifiConnected)
    {
      Serial.println("WiFi has disconnected!");
      sharedStatus.wifiConnected = false;
      sharedStatus.wifiConnecting = true;
      WiFi.reconnect();
    }

    xSemaphoreGive(statusMutex);

    vTaskDelay(5000 / portTICK_PERIOD_MS);
  }
}

void setup()
{
  Serial.begin(115200);
  Serial.println("\n=== ESP32 Dual-Core Focus Totem (v3 - Stable) ===");

  statusMutex = xSemaphoreCreateMutex();

  sharedStatus.wifiConnected = false;
  sharedStatus.wifiConnecting = true;
  WiFi.begin(ssid, password);
  Serial.print("Initial WiFi connection attempt...");
  while (WiFi.status() != WL_CONNECTED)
  {
    delay(500);
    Serial.print(".");
  }
  Serial.println("\nWiFi connected!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());

  sharedStatus.wifiConnected = true;
  sharedStatus.wifiConnecting = false;

  if (!MDNS.begin("focus-totem"))
  {
    Serial.println("Error setting up mDNS responder!");
    while (1)
      ;
  }
  MDNS.addService("http", "tcp", 80);

  server.on("/", HTTP_GET, handleRoot);
  server.on("/api/status", HTTP_GET, handleApiStatus);
  server.on("/status", HTTP_GET, handleStatus);
  server.begin();

  Serial.println("Web server running on Core 1 (main loop)");
  Serial.println("Dashboard: http://focus-totem.local/");

  // Create Status Monitor Task on Core 0
  xTaskCreatePinnedToCore(
      statusMonitorTask,
      "StatusMonitorTask",
      10000,
      NULL,
      1,
      &statusMonitorTaskHandle, // <-- THIS IS THE CORRECTED LINE
      0);

  Serial.println("Status monitor running on Core 0");
}

void loop()
{
  // The main loop() is now our dedicated Web Server task for Core 1
  server.handleClient();
  // Add a small delay to prevent the watchdog timer from triggering
  // and to allow other lower-priority tasks on the same core to run.
  vTaskDelay(2 / portTICK_PERIOD_MS);
}
```

`Multithreaded_Dashboard/M3Design.md`:

```md
# Design System Specification: The Monolithic Precision System

## 1. Overview & Creative North Star
**Creative North Star: "The Architectural Monolith"**

This design system rejects the ephemeral fluff of modern web trends in favor of grounded, architectural permanence. The aesthetic is defined by **Tonal Brutalism**: a high-utility, high-sophistication approach that uses solid masses of color to define space. 

By stripping away blurs, glassmorphism, and traditional drop shadows, we rely on the purity of the Material 3 "Expressive" logic. We create depth through "Carved Surfaces"—where the UI feels like a single block of obsidian with functional areas precisely milled into the surface. The result is an interface that feels authoritative, secure, and hyper-legible.

---

## 2. Colors & Surface Logic
The palette is rooted in deep minerals and high-contrast accents. We prioritize functional clarity over decorative gradients.

### The "No-Line" Rule
**Explicit Instruction:** 1px solid borders are strictly prohibited for sectioning or containment. 
Structure must be achieved through **Tonal Stepping**. To separate a sidebar from a main feed, or a header from a body, transition between `surface-container-low` (#1D2024) and `surface-container-highest` (#2B2D31). This creates a "milled" look where components appear to be physically inset or embossed within the interface.

### Surface Hierarchy (Dark Mode Default)
| Token | Hex | Role |
| :--- | :--- | :--- |
| **background** | #0b0e12 | The foundational "base" layer. |
| **surface-container-low** | #1D2024 | Primary background for main content areas and secondary sections. |
| **surface-container-highest**| #2B2D31 | Elevated surfaces: Cards, active modals, and high-priority containers. |
| **tertiary-container** | #d1f3dc | **Visited States:** A muted, sophisticated dark green to denote historical navigation. |
| **primary** | #bbdaff | Actionable elements and brand highlights. |

---

## 3. Typography: The Editorial Voice
We utilize a high-contrast scale to ensure the "Expressive" nature of the system is felt immediately. 

*   **Display & Headlines (Epilogue):** These are your "Statement" styles. Use `display-lg` (3.5rem) and `headline-lg` (2rem) with tight letter-spacing (-0.02em) to create a bold, editorial feel. These should feel like headlines in a premium architectural magazine.
*   **Body & Labels (Inter):** Reserved for high-density data. While the headers are expressive, the body remains a workhorse—clean, legible, and utilitarian.

**The Hierarchy Rule:** Never pair two "Display" sizes together. Use a bold `headline-md` for titles and immediately drop to `body-md` for descriptions to maximize the dynamic range of the layout.

---

## 4. Elevation & Depth: Tonal Stacking
Since shadows and blurs are forbidden, we use **The Stacking Principle** to communicate importance.

1.  **Level 0 (The Void):** `surface-container-low` (#1D2024). Use this for the largest background areas.
2.  **Level 1 (The Object):** `surface-container-highest` (#2B2D31). Use this for cards and list items. 
3.  **Level 2 (The Focus):** `primary` (#bbdaff). Used for the most critical interactive state.

**Ghost Borders (The Exception):** If high-density data requires a container but a background shift is too heavy, use `outline-variant` (#424850) at **15% opacity**. This creates a "perceived" edge that assists eye-tracking without introducing visual noise.

---

## 5. Components

### High-Contrast Touch Targets
Every interactive list element or tile must maintain a **minimum height of 56px**. This ensures the "Expressive" system remains accessible and feels premium under-thumb.

### Buttons & Chips
*   **Shape:** `rounded-full` (Pill shape).
*   **Primary:** Solid `primary` background with `on-primary` text. No shadows.
*   **Secondary:** `surface-container-highest` background.
*   **Interaction:** On press, shift the tonal value one step higher (e.g., from `surface-container-low` to `surface-container-highest`).

### Cards & Lists
*   **Rounding:** `rounded-[16px]`.
*   **The Divider Rule:** Forbid 1px dividers. Use a `1.4rem` (Spacing 4) vertical gap to separate list items. If separation is visually required, use a 1-step tonal shift between the list item and the background.
*   **Visited State:** Items that have been viewed or "planned" should transition their container or a secondary indicator to `tertiary-container` (Muted Green).

### The Bottom Sheet (Signature Component)
*   **Rounding:** `rounded-t-[32px]`.
*   **Style:** Must use `surface-container-highest` (#2B2D31) to contrast sharply against the lower-level background.
*   **Context:** Used for branch filtering and appointment confirmation.

### Input Fields
*   **Style:** Filled (not outlined).
*   **Background:** `surface-container-highest`.
*   **Active State:** A bottom-heavy `2px` border using the `primary` token. No glow/blur.

---

## 6. Do’s and Don’ts

### Do
*   **Do** use massive "Display" typography for branch names or empty states.
*   **Do** use the full spacing scale (up to `spacing-24`) to create "Breathing Room" around monolithic blocks.
*   **Do** rely on `surface-container` tiers to group related information.
*   **Do** ensure all primary actions use the `primary` (#bbdaff) color to pop against the dark mode.

### Don't
*   **Don't** use `drop-shadow`. If an element needs to stand out, make it a lighter tonal hex.
*   **Don't** use `backdrop-blur`. Backgrounds must remain solid and opaque.
*   **Don't** use 1px lines to separate content. Use whitespace or color shifts.
*   **Don't** cram data. If the touch target is less than 56px, the design is a failure of this system.

```

`Multithreaded_Dashboard/Multithreaded_Dashboard.ino`:

```ino
#include <WiFi.h>
#include <ESPmDNS.h>
#include <WebServer.h>
#include <ArduinoJson.h>

// --- IMPORTANT: CHANGE THESE TO YOUR WI-FI CREDENTIALS ---
const char *ssid = "Faizy's A54";
const char *password = "12345678";

WebServer server(80);

// --- Shared Status Variables ---
struct Status
{
  bool wifiConnecting;
  bool wifiConnected;
};
Status sharedStatus;

// --- FreeRTOS Handles ---
TaskHandle_t statusMonitorTaskHandle = NULL;
SemaphoreHandle_t statusMutex;

// --- THE (UNCHANGED) HTML PAGE ---
const char *dashboardHTML = R"rawliteral(
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Focus Totem Dashboard</title>
  <link href="https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;700&display=swap" rel="stylesheet">
  <script type="importmap">
    {
      "imports": {
        "@material/web/": "https://esm.run/@material/web/"
      }
    }
  </script>
  <script type="module">
    import '@material/web/all.js';
    import {styles as typescaleStyles} from '@material/web/typography/md-typescale-styles.js';
    document.adoptedStyleSheets.push(typescaleStyles.styleSheet);
  </script>
  <style>
    :root {
      --md-sys-color-primary: #6750A4;
      --md-sys-color-on-primary: #FFFFFF;
      --md-sys-color-primary-container: #EADDFF;
      --md-sys-color-on-primary-container: #21005D;
      --md-sys-color-secondary: #625B71;
      --md-sys-color-on-secondary: #FFFFFF;
      --md-sys-color-surface: #FEF7FF;
      --md-sys-color-on-surface: #1D1B20;
      --md-sys-color-surface-variant: #E7E0EC;
      --md-sys-color-on-surface-variant: #49454F;
      --md-sys-color-error: #B3261E;
      --md-sys-color-on-error: #FFFFFF;
    }
    body {
      font-family: 'Roboto', sans-serif;
      margin: 0;
      padding: 0;
      background-color: var(--md-sys-color-surface);
      color: var(--md-sys-color-on-surface);
    }
    .container {
      max-width: 800px;
      margin: 0 auto;
      padding: 24px;
    }
    .header {
      margin-bottom: 32px;
    }
    .status-card {
      background: var(--md-sys-color-surface-variant);
      border-radius: 12px;
      padding: 24px;
      margin-bottom: 16px;
    }
    .status-row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 0;
      border-bottom: 1px solid rgba(0,0,0,0.1);
    }
    .status-row:last-child {
      border-bottom: none;
    }
    .status-indicator {
      display: inline-block;
      width: 12px;
      height: 12px;
      border-radius: 50%;
      margin-right: 8px;
    }
    .status-connected {
      background-color: #4CAF50;
    }
    .status-disconnected {
      background-color: #F44336;
    }
    .status-connecting {
      background-color: #FF9800;
      animation: pulse 1.5s ease-in-out infinite;
    }
    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.5; }
    }
    .refresh-container {
      margin-top: 24px;
      text-align: center;
    }
    .core-badge {
      display: inline-block;
      background: var(--md-sys-color-primary-container);
      color: var(--md-sys-color-on-primary-container);
      padding: 4px 12px;
      border-radius: 16px;
      font-size: 12px;
      margin-left: 8px;
    }
  </style>
</head>
<body>
  <div class="container">
    <div class="header">
      <h1 class="md-typescale-display-small">Focus Totem Dashboard</h1>
      <p class="md-typescale-body-medium">Real-time status monitoring <span class="core-badge">Dual-Core</span></p>
    </div>

    <div class="status-card">
      <h2 class="md-typescale-title-large">System Status</h2>

      <div class="status-row">
        <span class="md-typescale-body-large">WiFi Connection</span>
        <span class="md-typescale-body-medium">
          <span class="status-indicator" id="wifi-indicator"></span>
          <span id="wifi-status">Loading...</span>
        </span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Network Name (SSID)</span>
        <span class="md-typescale-body-medium" id="ssid">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">IP Address</span>
        <span class="md-typescale-body-medium" id="ip-address">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">mDNS Hostname</span>
        <span class="md-typescale-body-medium" id="mdns">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Signal Strength</span>
        <span class="md-typescale-body-medium" id="rssi">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Uptime</span>
        <span class="md-typescale-body-medium" id="uptime">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Free Heap Memory</span>
        <span class="md-typescale-body-medium" id="heap">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Web Server Core</span>
        <span class="md-typescale-body-medium" id="web-core">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Monitor Core</span>
        <span class="md-typescale-body-medium" id="monitor-core">Loading...</span>
      </div>
    </div>

    <div class="refresh-container">
      <md-filled-button id="refresh-btn">Refresh Status</md-filled-button>
    </div>
  </div>

  <script>
    async function updateStatus() {
      try {
        const response = await fetch('/api/status');
        if (!response.ok) {
            console.error("Failed to fetch status, server responded with:", response.status);
            return;
        }
        const data = await response.json();

        const wifiIndicator = document.getElementById('wifi-indicator');
        const wifiStatus = document.getElementById('wifi-status');

        if (data.wifiConnected) {
          wifiIndicator.className = 'status-indicator status-connected';
          wifiStatus.textContent = 'Connected';
        } else if (data.wifiConnecting) {
          wifiIndicator.className = 'status-indicator status-connecting';
          wifiStatus.textContent = 'Connecting...';
        } else {
          wifiIndicator.className = 'status-indicator status-disconnected';
          wifiStatus.textContent = 'Disconnected';
        }

        document.getElementById('ssid').textContent = data.ssid || 'N/A';
        document.getElementById('ip-address').textContent = data.ipAddress || 'N/A';
        document.getElementById('mdns').textContent = data.mdns || 'N/A';
        document.getElementById('rssi').textContent = data.rssi || 'N/A';
        document.getElementById('uptime').textContent = data.uptime || 'N/A';
        document.getElementById('heap').textContent = data.freeHeap || 'N/A';
        document.getElementById('web-core').textContent = data.webCore || 'N/A';
        document.getElementById('monitor-core').textContent = data.monitorCore || 'N/A';
      } catch (error) {
        console.error('Error fetching status:', error);
      }
    }

    document.getElementById('refresh-btn').addEventListener('click', updateStatus);

    // Initial load
    updateStatus();

    // Auto-refresh every 5 seconds
    setInterval(updateStatus, 5000);
  </script>
</body>
</html>
)rawliteral";

String formatUptime(unsigned long milliseconds)
{
  unsigned long seconds = milliseconds / 1000;
  unsigned long minutes = seconds / 60;
  unsigned long hours = minutes / 60;
  unsigned long days = hours / 24;
  seconds %= 60;
  minutes %= 60;
  hours %= 24;
  String uptime = "";
  if (days > 0)
    uptime += String(days) + "d ";
  if (hours > 0)
    uptime += String(hours) + "h ";
  if (minutes > 0)
    uptime += String(minutes) + "m ";
  uptime += String(seconds) + "s";
  return uptime;
}

void handleRoot()
{
  server.send(200, "text/html", dashboardHTML);
}

void handleApiStatus()
{
  StaticJsonDocument<256> doc;
  xSemaphoreTake(statusMutex, portMAX_DELAY);
  bool isConnecting = sharedStatus.wifiConnecting;
  bool isConnected = sharedStatus.wifiConnected;
  xSemaphoreGive(statusMutex);

  doc["wifiConnected"] = isConnected;
  doc["wifiConnecting"] = isConnecting;
  doc["ssid"] = ssid;
  doc["mdns"] = "focus-totem.local";
  doc["uptime"] = formatUptime(millis());
  doc["freeHeap"] = String(ESP.getFreeHeap()) + " bytes";

  if (isConnected)
  {
    doc["ipAddress"] = WiFi.localIP().toString();
    doc["rssi"] = String(WiFi.RSSI()) + " dBm";
  }
  else
  {
    doc["ipAddress"] = "N/A";
    doc["rssi"] = "N/A";
  }

  doc["webCore"] = "Core " + String(xPortGetCoreID());
  doc["monitorCore"] = "Core 0";

  String jsonResponse;
  serializeJson(doc, jsonResponse);

  server.sendHeader("Access-Control-Allow-Origin", "*");
  server.send(200, "application/json", jsonResponse);
}

void handleStatus()
{
  server.send(200, "text/plain", "FOCUS_ON");
}

void statusMonitorTask(void *parameter)
{
  for (;;)
  {
    bool isConnectedNow = (WiFi.status() == WL_CONNECTED);
    xSemaphoreTake(statusMutex, portMAX_DELAY);

    if (isConnectedNow && !sharedStatus.wifiConnected)
    {
      Serial.println("WiFi has connected!");
      sharedStatus.wifiConnected = true;
      sharedStatus.wifiConnecting = false;
    }
    else if (!isConnectedNow && sharedStatus.wifiConnected)
    {
      Serial.println("WiFi has disconnected!");
      sharedStatus.wifiConnected = false;
      sharedStatus.wifiConnecting = true;
      WiFi.reconnect();
    }

    xSemaphoreGive(statusMutex);

    vTaskDelay(5000 / portTICK_PERIOD_MS);
  }
}

void setup()
{
  Serial.begin(115200);
  Serial.println("\n=== ESP32 Dual-Core Focus Totem (v3 - Stable) ===");

  statusMutex = xSemaphoreCreateMutex();

  sharedStatus.wifiConnected = false;
  sharedStatus.wifiConnecting = true;
  WiFi.begin(ssid, password);
  Serial.print("Initial WiFi connection attempt...");
  while (WiFi.status() != WL_CONNECTED)
  {
    delay(500);
    Serial.print(".");
  }
  Serial.println("\nWiFi connected!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());

  sharedStatus.wifiConnected = true;
  sharedStatus.wifiConnecting = false;

  if (!MDNS.begin("focus-totem"))
  {
    Serial.println("Error setting up mDNS responder!");
    while (1)
      ;
  }
  MDNS.addService("http", "tcp", 80);

  server.on("/", HTTP_GET, handleRoot);
  server.on("/api/status", HTTP_GET, handleApiStatus);
  server.on("/status", HTTP_GET, handleStatus);
  server.begin();

  Serial.println("Web server running on Core 1 (main loop)");
  Serial.println("Dashboard: http://focus-totem.local/");

  // Create Status Monitor Task on Core 0
  xTaskCreatePinnedToCore(
      statusMonitorTask,
      "StatusMonitorTask",
      10000,
      NULL,
      1,
      &statusMonitorTaskHandle, // <-- THIS IS THE CORRECTED LINE
      0);

  Serial.println("Status monitor running on Core 0");
}

void loop()
{
  // The main loop() is now our dedicated Web Server task for Core 1
  server.handleClient();
  // Add a small delay to prevent the watchdog timer from triggering
  // and to allow other lower-priority tasks on the same core to run.
  vTaskDelay(2 / portTICK_PERIOD_MS);
}

```

`Multithreaded_Dashboard/test.html`:

```html
<!doctype html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Focus Totem Dashboard</title>

        <!-- The Editorial Voice: Typography Base -->
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
        <link
            href="https://fonts.googleapis.com/css2?family=Epilogue:wght@700&family=Inter:wght@400;500;600&display=swap"
            rel="stylesheet"
        />

        <style>
            :root {
                /* Surface Hierarchy (Dark Mode Default) */
                --bg-void: #0b0e12;
                --surface-low: #1d2024;
                --surface-highest: #2b2d31;
                --tertiary-muted: #d1f3dc;
                --primary-accent: #bbdaff;

                /* Typography Colors */
                --text-primary: #ffffff;
                --text-secondary: #9ca3af;
            }

            * {
                box-sizing: border-box;
                margin: 0;
                padding: 0;
            }

            body {
                font-family: "Inter", sans-serif;
                background-color: var(--bg-void);
                color: var(--text-primary);
                -webkit-font-smoothing: antialiased;
                min-height: 100vh;
                display: flex;
                justify-content: center;
                padding: 64px 24px;
            }

            .monolith-wrapper {
                width: 100%;
                max-width: 800px;
                display: flex;
                flex-direction: column;
                gap: 56px; /* Massive breathing room per Do's and Don'ts */
            }

            /* Typography: Statement Styles */
            .display-lg {
                font-family: "Epilogue", sans-serif;
                font-size: 3.5rem;
                font-weight: 700;
                letter-spacing: -0.02em;
                line-height: 1.1;
                color: var(--text-primary);
            }

            .headline-md {
                font-family: "Epilogue", sans-serif;
                font-size: 1.5rem;
                font-weight: 700;
                letter-spacing: -0.02em;
                color: var(--text-primary);
            }

            /* Typography: High-Density Data Workhorse */
            .body-md {
                font-family: "Inter", sans-serif;
                font-size: 1rem;
                color: var(--text-secondary);
                line-height: 1.5;
                margin-top: 16px;
            }

            .label {
                font-family: "Inter", sans-serif;
                font-size: 1rem;
                font-weight: 500;
                color: var(--text-primary);
            }

            .value {
                font-family: "Inter", sans-serif;
                font-size: 1rem;
                color: var(--text-secondary);
                display: flex;
                align-items: center;
            }

            .badge {
                background-color: var(--surface-highest);
                color: var(--primary-accent);
                padding: 6px 16px;
                border-radius: 9999px;
                font-size: 0.875rem;
                font-weight: 600;
                margin-left: 16px;
                vertical-align: middle;
            }

            /* Tonal Stepping: Level 0 -> Level 1 */
            .dashboard-surface {
                background-color: var(--surface-low);
                border-radius: 24px;
                padding: 40px;
                display: flex;
                flex-direction: column;
                gap: 32px;
            }

            /* The Divider Rule: No 1px lines */
            .list-container {
                display: flex;
                flex-direction: column;
                gap: 1.4rem;
            }

            /* High-Contrast Touch Targets & Cards */
            .list-item {
                background-color: var(
                    --surface-highest
                ); /* Tonal shift milled inside the surface */
                border-radius: 16px;
                min-height: 56px; /* Guaranteed accessible hit area */
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 0 24px;
            }

            /* Buttons & Chips: Primary Actions */
            .btn-primary {
                background-color: var(--primary-accent);
                color: var(--bg-void);
                border: none;
                border-radius: 9999px; /* Pill Shape */
                min-height: 56px; /* Strict 56px rule */
                padding: 0 40px;
                font-family: "Inter", sans-serif;
                font-size: 1.125rem;
                font-weight: 600;
                cursor: pointer;
                transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
                display: inline-flex;
                align-items: center;
                justify-content: center;
            }

            .btn-primary:active {
                transform: scale(0.97);
            }

            /* Status Indicators */
            .status-indicator {
                display: inline-block;
                width: 12px;
                height: 12px;
                border-radius: 50%;
                margin-right: 12px;
            }

            .status-connected {
                background-color: var(--tertiary-muted);
            }

            .status-disconnected {
                background-color: #ff5e5e; /* Stark contrast error state */
            }

            .status-connecting {
                background-color: var(--primary-accent);
                animation: pulse 1.5s ease-in-out infinite;
            }

            @keyframes pulse {
                0%,
                100% {
                    opacity: 1;
                }
                50% {
                    opacity: 0.2;
                }
            }

            .actions-wrapper {
                display: flex;
                justify-content: flex-start;
            }

            /* Form Factor Adaptations */
            @media (max-width: 600px) {
                body {
                    padding: 32px 16px;
                }
                .display-lg {
                    font-size: 2.75rem;
                }
                .dashboard-surface {
                    padding: 24px;
                }
                .list-item {
                    flex-direction: column;
                    align-items: flex-start;
                    justify-content: center;
                    gap: 8px;
                    padding: 16px 24px;
                }
            }
        </style>
    </head>
    <body>
        <div class="monolith-wrapper">
            <!-- The Hierarchy Rule Applied -->
            <header class="header">
                <h1 class="display-lg">Focus Totem</h1>
                <p class="body-md">
                    Real-time status monitoring
                    <span class="badge">Dual-Core</span>
                </p>
            </header>

            <main class="dashboard-surface">
                <h2 class="headline-md">System Status</h2>

                <div class="list-container">
                    <div class="list-item">
                        <span class="label">WiFi Connection</span>
                        <span class="value">
                            <span
                                class="status-indicator"
                                id="wifi-indicator"
                            ></span>
                            <span id="wifi-status">Loading...</span>
                        </span>
                    </div>

                    <div class="list-item">
                        <span class="label">Network Name (SSID)</span>
                        <span class="value" id="ssid">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">IP Address</span>
                        <span class="value" id="ip-address">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">mDNS Hostname</span>
                        <span class="value" id="mdns">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">Signal Strength</span>
                        <span class="value" id="rssi">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">Uptime</span>
                        <span class="value" id="uptime">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">Free Heap Memory</span>
                        <span class="value" id="heap">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">Web Server Core</span>
                        <span class="value" id="web-core">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">Monitor Core</span>
                        <span class="value" id="monitor-core">Loading...</span>
                    </div>
                </div>
            </main>

            <div class="actions-wrapper">
                <button class="btn-primary" id="refresh-btn">
                    Refresh Status
                </button>
            </div>
        </div>

        <script>
            async function updateStatus() {
                try {
                    const response = await fetch("/api/status");
                    if (!response.ok) {
                        console.error(
                            "Failed to fetch status, server responded with:",
                            response.status,
                        );
                        return;
                    }
                    const data = await response.json();

                    const wifiIndicator =
                        document.getElementById("wifi-indicator");
                    const wifiStatus = document.getElementById("wifi-status");

                    if (data.wifiConnected) {
                        wifiIndicator.className =
                            "status-indicator status-connected";
                        wifiStatus.textContent = "Connected";
                    } else if (data.wifiConnecting) {
                        wifiIndicator.className =
                            "status-indicator status-connecting";
                        wifiStatus.textContent = "Connecting...";
                    } else {
                        wifiIndicator.className =
                            "status-indicator status-disconnected";
                        wifiStatus.textContent = "Disconnected";
                    }

                    document.getElementById("ssid").textContent =
                        data.ssid || "N/A";
                    document.getElementById("ip-address").textContent =
                        data.ipAddress || "N/A";
                    document.getElementById("mdns").textContent =
                        data.mdns || "N/A";
                    document.getElementById("rssi").textContent =
                        data.rssi || "N/A";
                    document.getElementById("uptime").textContent =
                        data.uptime || "N/A";
                    document.getElementById("heap").textContent =
                        data.freeHeap || "N/A";
                    document.getElementById("web-core").textContent =
                        data.webCore || "N/A";
                    document.getElementById("monitor-core").textContent =
                        data.monitorCore || "N/A";
                } catch (error) {
                    console.error("Error fetching status:", error);
                }
            }

            document
                .getElementById("refresh-btn")
                .addEventListener("click", updateStatus);

            // Initial load
            updateStatus();

            // Auto-refresh every 5 seconds
            setInterval(updateStatus, 5000);
        </script>
    </body>
</html>

```

`README.md`:

```md
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
    *   **Session Logging:** Records every completed focus session to a local JSON file for future analytics/reporting.

## Key Features Implemented

| ESP32 Firmware | Rust Desktop Client |
| :--- | :--- |
| ✅ Dual-Core Operation (FreeRTOS) | ✅ Automatic mDNS Device Discovery |
| ✅ Material Design 3 Web Dashboard | ✅ Real-time State Tracking |
| ✅ Real-time JSON API for status | ✅ **Dynamic Wallpaper Changing** |
| ✅ Backward-compatible `/status` endpoint | ✅ **Application Launching & Closing** |
| | ✅ Cross-platform app config via `apps.toml` |
| | ✅ Local JSON session logging |

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
  "brave-browser"
]
        ```

*   **Build and Run (Mock Mode for ESP32-free development):**
    1.  Start the Rust mock server:
        ```focus_client_rust/README.md#L1-1
cargo run -p mock_server
        ```
    2.  In another terminal, run the Rust client in debug mode:
        ```focus_client_rust/README.md#L1-1
cargo run
        ```
    3.  In debug mode, the client prints a development banner and polls:
        - `http://localhost:8080/status`
        - Expected response: `FOCUS_ON` or `FOCUS_OFF`
    4.  Toggle focus state without stopping the mock server:
        ```focus_client_rust/README.md#L1-1
curl http://localhost:8080/toggle
        ```
    5.  Stop the mock server (`Ctrl+C`) to simulate device disconnect and verify disconnect behavior.
    6.  On GNOME-based Linux distros (like Zorin), wallpaper switching uses `gsettings` first, then falls back to the `wallpaper` crate for better compatibility.

*   **Build and Run (Real ESP32 mode):**
    1.  Build/run in release mode:
        ```focus_client_rust/README.md#L1-2
cargo build --release
cargo run --release
        ```
    2.  In this mode, the client uses normal mDNS discovery (`focus-totem`) and polls the real `/status` endpoint.

*   **Session Logs:**
    - A completed focus session is recorded when focus mode deactivates.
    - The client prints the session log file path on first write.
    - Linux path: `~/.local/share/focus_totem/sessions.json`
    - Windows path: `%APPDATA%\FocusTotem\sessions.json`
    - The file is a JSON array, ready for analytics/report generation later.

*   **Analytics:**
    - Run analytics without launching apps/wallpaper automation:
        ```focus_client_rust/README.md#L1-1
cargo run -- --analytics
        ```
    - Layer 1 statistical aggregation runs with 1+ session.
    - Trend detection activates after 7+ calendar days.
    - Decision-tree quality prediction activates after 30+ sessions.

## Project Files

*   `Multithreaded_Dashboard.ino`: **The main, recommended firmware for the ESP32.**
*   `src/main.rs`: The source code for the Rust desktop client (includes debug mock-mode logic and release real-device logic).
*   `src/session.rs`: Focus session model and atomic JSON session logging.
*   `src/analytics.rs`: Pure Rust analytics pipeline for session stats, trend detection, and quality prediction.
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

---
_This project is now Rust-first: the desktop client and mock ESP32 server both live in this workspace._

```

`ROADMAP-new.md`:

```md
# Focus Totem — Development Roadmap

> **Constraints in play:** No ESP32 hardware available. Linux (GNOME) primary dev environment. Max Rust — Python goes in the bin.

---

## Current State Snapshot

- ✅ `src/main.rs` — cross-platform client (Windows + Linux GNOME), `apps.toml` config, `DEV_MODE` compile flag, gsettings wallpaper fallback
- ✅ `M3-Redesign.ino` — dual-core FreeRTOS firmware with Material Design 3 dashboard
- ✅ `mock_server/` — Rust Axum mock server with `/status` and `/toggle` endpoints
- ✅ `hello_gpui/` — GPUI UI experiment (parked, not integrated)
- ✅ Session logging — implemented in `src/session.rs` with local JSON storage
- ✅ AI analytics — implemented in `src/analytics.rs` with threshold-gated ML layers
- ❌ Report generation — not implemented

---

## Phase 1 — Kill `mock_esp32.py`: Rust Mock Server Binary

**Status: ✅ Complete**

Replaced the Python mock server with an Axum binary in the workspace. Two endpoints:
- `GET /status` → `"FOCUS_ON"` or `"FOCUS_OFF"` based on shared `Arc<AtomicBool>` state
- `GET /toggle` → flips state, returns new value — lets you test deactivation without killing the process

Run with: `cargo run -p mock_server` in one terminal, `cargo run` in another.

---

## Phase 2 — Session Logging Foundation

**Status: ✅ Complete**

Every focus session is silently written to a platform-aware JSON file:
- Linux: `~/.local/share/focus_totem/sessions.json`
- Windows: `%APPDATA%\FocusTotem\sessions.json`

Writes are atomic (`write to .tmp` → `rename`). `session.rs` owns the full lifecycle: `SessionRecord` struct, `append_session()`, `sessions_file_path()`.

---

## Phase 3 — AI Analytics: 3-Layer Pure Rust Stack

**Goal:** Extract meaningful insights from session data. Zero Python, zero ML black boxes.

**Status:** ✅ Implemented

> **Why not K-Means?** K-means finds natural *groupings* where none are predefined. Your data has *temporal patterns*, not natural clusters — forcing k=3 on `(duration, hour)` pairs produces arbitrary blobs with no semantic meaning, especially on small datasets. The k was also completely arbitrary.
>
> **Why not plain Linear Regression on raw features?** `hour_of_day → duration_minutes` is not a linear relationship. Productivity peaks mid-morning, dips post-lunch — a straight line fit would be actively misleading.
>
> **Why not Random Forest?** Ensemble methods reduce variance across many trees. That only helps on large datasets. With tens-to-low-hundreds of sessions, a single Decision Tree generalises better and, crucially, its rules are human-readable — far more valuable for a demo.

### The Correct Stack: 3 Layers, Each Gated by Data Threshold

| Layer | Method | Crate | Minimum data | Activates when |
|---|---|---|---|---|
| **1** | Statistical aggregation | `std` only | 1 session | Always |
| **2** | Linear regression (1D time-series) | `linfa-linear` | 7+ calendar days | Auto |
| **3** | Decision tree (3-class quality) | `linfa-trees` | 30+ sessions | Auto |

Each layer checks its threshold at runtime and skips gracefully with a message like:
`"Predictive model needs 28 more sessions — collecting data"`.

---

### Add to `Cargo.toml`

```toml
linfa          = "0.7"
linfa-linear   = "0.7"
linfa-trees    = "0.7"
ndarray        = "0.15"
```

> `linfa-clustering` is **not** needed. Drop it from the original plan.

---

### Create `src/analytics.rs`

#### Layer 1 — Statistical Aggregation (always runs, no ML)

No model, no training. Pure grouping and arithmetic on `Vec<SessionRecord>`.

**Peak-hour detection**
- Group sessions by `hour_of_day` into 24 buckets
- For each bucket compute: `score = mean_duration_minutes × session_count`
  - Rewards hours that are both long *and* consistent, not just lucky outliers
- Sort descending → top 3 = recommended focus windows
- Format output: `"10:00–11:00 (score: 47.3)"`

**Best day detection**
- Group by `day_of_week` (0=Mon…6=Sun)
- Compute mean duration per day → sort → report top 2

**Distraction rate**
- `rate = (interrupted_count as f32 / total as f32) * 100.0`
- Threshold: `rate > 30.0` → flag for a specific report callout

**Weekly summary**
- Filter sessions by `start_time` within current week (Mon 00:00 → now)
- Sum `duration_minutes` → total deep work hours this week
- Compare against prior week total → compute delta: `+2h 15m vs last week`

---

#### Layer 2 — Linear Regression on Daily Totals (trend detection)

**What it answers:** Is my focus improving or declining over time?

**Training data:** One data point per calendar day — `(day_index: f32, total_focus_minutes: f32)`. Day index is just 0, 1, 2, 3… from the first recorded session.

**Training (linfa-linear):**
```rust
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use ndarray::{Array1, Array2};

// x: day indices [0.0, 1.0, 2.0, ...]
// y: total focus minutes per day [45.0, 60.0, 30.0, ...]
let x: Array2<f32> = Array2::from_shape_vec((n_days, 1), day_indices)?;
let y: Array1<f32> = Array1::from_vec(daily_totals);

let dataset = linfa::Dataset::new(x.clone(), y);
let model = LinearRegression::default().fit(&dataset)?;

// The slope is what matters
let slope = model.params()[0]; // positive = improving, negative = declining
```

**No separate predict step needed** — you only care about the slope coefficient, not future predictions. `slope > 0.5` → `"📈 Focus trending up"`. `slope < -0.5` → `"📉 Focus declining — protect your blocks"`. `|slope| <= 0.5` → `"Focus is steady"`.

**This is genuine supervised ML** — the model fits a line to labelled `(x, y)` pairs and learns a weight (the slope). Training happens in-process each time the report runs, takes microseconds on this data size, and the model is not persisted to disk (no need — retraining from the JSON file is instant).

---

#### Layer 3 — Decision Tree: Quality Prediction (predictive model)

**What it answers:** Given an hour and day, will this likely be a quality session?

**Label generation (automatic, no manual work):**
Derive a `SessionQuality` label from fields already in `SessionRecord`:

```rust
#[derive(Clone, Copy)]
enum SessionQuality {
    Quality    = 0,  // duration >= 20 min AND not interrupted
    Shallow    = 1,  // duration >= 10 min (but < 20, or interrupted)
    Distracted = 2,  // duration < 10 min (already tracked as interrupted=true)
}

fn label(s: &SessionRecord) -> SessionQuality {
    if s.duration_minutes >= 20.0 && !s.interrupted {
        SessionQuality::Quality
    } else if s.duration_minutes >= 10.0 {
        SessionQuality::Shallow
    } else {
        SessionQuality::Distracted
    }
}
```

**Features (inputs to the tree):**
- `hour_of_day` as `f32` (0–23)
- `day_of_week` as `f32` (0–6)

**Training (linfa-trees):**
```rust
use linfa::prelude::*;
use linfa_trees::{DecisionTree, SplitQuality};
use ndarray::{Array1, Array2};

// Shape: (n_sessions, 2) — [hour_of_day, day_of_week]
let features: Array2<f32> = /* built from sessions */;
// Shape: (n_sessions,) — [0, 1, 2, ...] = [Quality, Shallow, Distracted]
let labels: Array1<usize> = /* derived via label() */;

let dataset = linfa::Dataset::new(features, labels);
let model = DecisionTree::params()
    .split_quality(SplitQuality::Gini)
    .max_depth(Some(4))       // keep it shallow = interpretable
    .min_samples_split(5)     // don't split on fewer than 5 sessions
    .fit(&dataset)?;
```

**`max_depth(4)` is deliberate** — deeper trees memorise training data (overfit). With small datasets a depth of 3–4 generalises well and produces readable rules.

**Prediction (inference):**
```rust
// Query: "How likely is quality focus at 10am on a Monday?"
let query = Array2::from_shape_vec((1, 2), vec![10.0_f32, 0.0])?;
let prediction = model.predict(&query); // returns [0], [1], or [2]
```

**Model persistence:** The tree is **retrained from `sessions.json` every time** the report runs. At this data scale (sub-millisecond training), there is no benefit to serialising and loading a model file. If you later accumulate thousands of sessions and training noticeably slows, you can serialise with `serde` + `bincode` — but you're not there yet.

**Human-readable output:** `linfa-trees` lets you walk the tree's split nodes. Implement a simple recursive printer in `analytics.rs` that produces output like:
```
If hour <= 11.5:
  If day_of_week <= 4.5 (Mon–Fri):
    → Quality (confidence: 78%)
  Else (weekend):
    → Shallow (confidence: 61%)
Else (afternoon/evening):
  → Distracted (confidence: 83%)
```
This goes verbatim into the report. Interpretable, academically credible, actually useful.

---

### `AnalyticsResult` struct (output of the whole module)

```rust
pub struct AnalyticsResult {
    // Layer 1 — always present
    pub total_sessions: usize,
    pub distraction_rate: f32,
    pub top_focus_hours: Vec<(u8, f32)>,   // (hour, score), top 3
    pub best_days: Vec<(u8, f32)>,          // (day, mean_duration), top 2
    pub weekly_total_minutes: f32,
    pub weekly_delta_minutes: f32,          // vs prior week, can be negative

    // Layer 2 — None if < 7 calendar days of data
    pub trend_slope: Option<f32>,
    pub trend_label: Option<String>,        // "📈 Trending up" etc.

    // Layer 3 — None if < 30 sessions
    pub tree_rules: Option<String>,         // rendered decision tree text
    pub quality_rate: Option<f32>,          // % of sessions labelled Quality
}
```

### Public API of `analytics.rs`

```rust
pub fn run_analytics(sessions: &[SessionRecord]) -> AnalyticsResult
```

Called from `main.rs` (on `--report` flag) and from `report.rs`. Takes a slice — no file I/O inside `analytics.rs`. Callers load sessions; analytics crunches them.

---

## Phase 4 — Focus Health Report Generator

**Goal:** Generate a styled HTML report and open it in the default browser.

### Steps

- Write `src/report.rs` with `generate_report(result: &AnalyticsResult) -> String`
- Template uses the **Monolithic Precision design system** from `M3Design.md`:
  - `#0b0e12` void, `#1D2024` surface, `#2B2D31` card, `#bbdaff` accent, Inter font
  - Same list-item / card pattern as the ESP32 dashboard
- Write to `std::env::temp_dir().join("focus_report.html")` then open:
  ```rust
  #[cfg(target_os = "linux")]
  Command::new("xdg-open").arg(&path).spawn().ok();
  #[cfg(target_os = "windows")]
  Command::new("explorer").arg(&path).spawn().ok();
  ```

**Report sections:**
1. **Weekly Summary** — total hours, session count, delta vs last week
2. **Top Focus Windows** — top 3 hours with scores, styled as ranked cards
3. **Trend** — slope label + a CSS `<div>` width-trick bar chart of 7-day daily totals (no JS, no chart lib — pure HTML)
4. **Decision Tree Rules** — rendered verbatim in a `<pre>` block if Layer 3 is active, else a "collecting data" notice
5. **AI Recommendation** — one plain-English paragraph assembled from the `AnalyticsResult` fields

**CLI flag** — parse `std::env::args()` manually:
```rust
if std::env::args().any(|a| a == "--report") {
    let sessions = session::load_all_sessions();
    let result = analytics::run_analytics(&sessions);
    let html = report::generate_report(&result);
    report::open_in_browser(&html);
    return;
}
```

---

## Phase 5 — Modern Rust Housekeeping

**Goal:** Remove legacy patterns before the codebase grows further.

### 5.1 Replace `lazy_static` with `std::sync::LazyLock`

Stable since Rust 1.80. Zero external dependency.

```rust
// Before
lazy_static! {
    static ref ORIGINAL_WALLPAPER_PATH: Mutex<Option<String>> = Mutex::new(None);
}

// After
static ORIGINAL_WALLPAPER_PATH: LazyLock<Mutex<Option<String>>> =
    LazyLock::new(|| Mutex::new(None));
```

Remove `lazy_static` from `Cargo.toml` entirely.

### 5.2 Module structure

Split `src/main.rs` into focused modules:

```
src/
├── main.rs          # entry point + main loop only
├── config.rs        # load_focus_apps(), AppsConfig, constants
├── automation.rs    # launch_app(), terminate_app(), wallpaper functions
├── discovery.rs     # discover_device(), mDNS logic
├── session.rs       # SessionRecord, append_session(), file I/O  ← exists
├── analytics.rs     # 3-layer analytics, AnalyticsResult         ← Phase 3
└── report.rs        # HTML generation, browser open              ← Phase 4
```

### 5.3 Error handling

- Add `anyhow = "1"` to deps
- `main()` returns `anyhow::Result<()>`
- Replace `unwrap()` in non-fatal paths with `?` or `if let Err(e)`

### 5.4 Remove or graduate `hello_gpui/`

Currently dead-end with stale GPUI API calls. Recommendation: delete, open a GitHub issue titled "GUI: GPUI integration" to track it separately.

---

## Phase 6 — Async Refactor (Main Client)

**Goal:** Replace `blocking reqwest + thread::sleep` with proper async.

- Add `tokio = { version = "1", features = ["full"] }` to root `Cargo.toml`
- Switch `reqwest` feature from `blocking` → `json` (async by default)
- Annotate `main` with `#[tokio::main]`
- Replace `thread::sleep(...)` with `tokio::time::sleep(...).await`
- Wrap `discover_device()` in `tokio::task::spawn_blocking` (mdns-sd uses its own thread internally)
- `activate_focus_mode` / `deactivate_focus_mode` stay sync — call from `spawn_blocking`

tokio is already a workspace dependency via `mock_server`, so compile overhead is negligible.

---

## Phase 7 — System Integration: Autostart & Background

### Linux — systemd user service

```ini
[Unit]
Description=Focus Totem Client
After=graphical-session.target

[Service]
Type=simple
ExecStart=/path/to/focus_client_rust
Restart=on-failure
RestartSec=5s
Environment=DISPLAY=:0

[Install]
WantedBy=graphical-session.target
```

Install: `cp focus-totem.service ~/.config/systemd/user/ && systemctl --user enable --now focus-totem`

Add `--install-service` CLI flag that writes and enables this file automatically.

### Windows — registry autostart

```rust
#[cfg(target_os = "windows")]
fn install_autostart() {
    use winreg::enums::*;
    let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_WRITE
    ).unwrap();
    run.set_value("FocusTotem", &std::env::current_exe().unwrap().to_str().unwrap()).unwrap();
}
```

### Tray icon (optional)

`tray-icon = "0.19"` — three states: searching (grey), connected (blue), focus active (green). Right-click: "View Report" / "Force Deactivate" / "Quit".

---

## Phase 8 — ESP32 Enhancements (hardware-dependent)

- **Manual override from dashboard:** `POST /toggle` flips `isFocusOverride` bool. `/status` returns `FOCUS_OFF` when set. Test deactivation without unplugging.
- **LED indicator:** GPIO output → green = `FOCUS_ON`, off = idle.
- **Button ISR:** Physical button → `attachInterrupt()` + `portENTER_CRITICAL` for FreeRTOS-safe hardware toggle.
- **OTA updates:** `ArduinoOTA` library — push firmware over Wi-Fi from Arduino IDE, no USB required.

---

## Dependency Summary (end state)

```toml
[dependencies]
# Existing
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
toml         = "0.8"
reqwest      = { version = "0.12", features = ["json"] }  # async after Phase 6
wallpaper    = "3.2"
mdns-sd      = "0.15"
chrono       = { version = "0.4", features = ["serde"] }
dirs         = "5"

# Phase 3 — Analytics
linfa        = "0.7"
linfa-linear = "0.7"   # Layer 2: trend regression
linfa-trees  = "0.7"   # Layer 3: decision tree quality prediction
ndarray      = "0.15"   # Matches linfa 0.7

# Phase 5 — Housekeeping
anyhow       = "1"

# Phase 6 — Async
tokio        = { version = "1", features = ["full"] }

# Phase 7 — Autostart (Windows only)
[target.'cfg(windows)'.dependencies]
winreg = "0.52"

# Removed vs original plan
# lazy_static     — replaced by std::sync::LazyLock (Rust 1.80+)
# linfa-clustering — K-means dropped, wrong algorithm for this data
```

---

## Execution Order

| Priority | Phase | Why first |
|---|---|---|
| ✅ Done | **Phase 1** — Rust mock server | Python dep eliminated |
| ✅ Done | **Phase 2** — Session logging | Data pipeline live |
| 🔴 Next | **Phase 3** — Analytics | Core AI deliverable for CEP |
| 🟠 | **Phase 4** — Report generator | Closes the AI loop, demo-able artifact |
| 🟠 | **Phase 5** — Housekeeping | Do before codebase grows further |
| 🟡 | **Phase 6** — Async refactor | Needed before any GUI work |
| 🟡 | **Phase 7** — Autostart | Usability polish |
| 🟢 | **Phase 8** — ESP32 hardening | Hardware-dependent |

```

`ROADMAP.md`:

```md
# Focus Totem — Development Roadmap

> **Constraints in play:** No ESP32 hardware available. Linux (GNOME) primary dev environment. Max Rust — Python goes in the bin.

---

## Current State Snapshot

- ✅ `src/main.rs` — cross-platform client (Windows + Linux GNOME), `apps.toml` config, `DEV_MODE` compile flag, gsettings wallpaper fallback
- ✅ `M3-Redesign.ino` — dual-core FreeRTOS firmware with Material Design 3 dashboard
- ✅ `mock_server/` — Rust Axum mock server with `/status` and `/toggle` endpoints
- ✅ `hello_gpui/` — GPUI UI experiment (parked, not integrated)
- ✅ Session logging — implemented in `src/session.rs` with local JSON storage
- ✅ AI analytics — implemented in `src/analytics.rs`
- ❌ Report generation — not implemented
- ✅ Rust mock server — implemented in `mock_server/`

---

## Phase 1 — Kill `mock_esp32.py`: Rust Mock Server Binary

**Goal:** Purge the Python dependency entirely. Replace with a proper Rust binary in the same workspace.

**Status:** ✅ Implemented

### Steps

- Convert the project root into a **Cargo workspace** by updating `Cargo.toml`:
  ```toml
  [workspace]
  members = [".", "mock_server"]
  ```
- Create `mock_server/` as a new crate: `cargo new mock_server --bin`
- Add dependencies to `mock_server/Cargo.toml`:
  ```toml
  axum = "0.8"
  tokio = { version = "1", features = ["full"] }
  ```
- Implement the server with **two endpoints**:
  - `GET /status` → returns `"FOCUS_ON"` or `"FOCUS_OFF"` based on shared atomic state
  - `GET /toggle` → flips the state and returns the new value (so you can simulate disconnect without killing the process)
- The state should be an `Arc<AtomicBool>` shared between the toggle handler and the status handler
- **No CLI args needed** — hardcode `127.0.0.1:8080` matching the existing `DEV_MODE` constant
- Run with: `cargo run -p mock_server` in one terminal, `cargo run` in another

### Why axum
- Tokio-native, zero-cost, production-grade — not overkill since you'll reuse tokio when refactoring the main client later
- A single route takes ~10 lines. Compile time is the only cost

---

## Phase 2 — Session Logging Foundation

**Goal:** The client silently records every focus session to a local JSON file. This is the data source for all AI work.

**Status:** ✅ Implemented

### Steps

- Add to `Cargo.toml`:
  ```toml
  chrono = { version = "0.4", features = ["serde"] }
  serde_json = "1"
  dirs = "5"           # XDG / platform-aware data dir resolution
  ```
- Define the session record struct in a new `src/session.rs` module:
  ```rust
  #[derive(Serialize, Deserialize)]
  pub struct SessionRecord {
      pub start_time: DateTime<Local>,
      pub end_time: DateTime<Local>,
      pub duration_minutes: f32,
      pub hour_of_day: u8,          // 0–23, from start_time
      pub day_of_week: u8,          // 0=Mon … 6=Sun
      pub interrupted: bool,         // duration < 10 min
  }
  ```
- Resolve the sessions file path with `dirs::data_dir()`:
  - Linux: `~/.local/share/focus_totem/sessions.json`
  - Windows: `%APPDATA%\FocusTotem\sessions.json`
  - Fallback: `./sessions.json` next to the binary
- Store sessions as a **JSON array** — append on each session end (read → push → write)
- Track session start: set a `Option<DateTime<Local>>` when `activate_focus_mode` is called
- Write the record when `deactivate_focus_mode` is called, computing duration from start
- Log the file path to stdout on first write so it's easy to find

### Data integrity note
- Read the full file, deserialize, push the new record, serialize back, write atomically (write to `.tmp` then `rename`)
- `rename` is atomic on Linux. Use `std::fs::rename`

---

## Phase 3 — AI Analytics: Pure Rust with `linfa`

**Goal:** K-means clustering on session data + peak-hour detection. Zero Python.

**Status:** Superseded by `ROADMAP-new.md`; implemented as a 3-layer analytics stack in `src/analytics.rs`.

### Add to `Cargo.toml`
```toml
linfa = "0.7"
linfa-clustering = "0.7"
ndarray = "0.16"
```

### Create `src/analytics.rs`

#### 3.1 Load & filter sessions
- Read `sessions.json`, deserialize into `Vec<SessionRecord>`
- Require at least **10 sessions** before running clustering (return early with a message otherwise — cold start problem)

#### 3.2 K-means clustering (k=3)
- Feature matrix: `[duration_minutes, hour_of_day as f32]` — shape `(n_sessions, 2)`
- Normalize both features to `[0, 1]` before fitting (duration range ≈ 0–120 min, hour range 0–23)
- Use `linfa_clustering::KMeans::params(3).fit(&dataset)` — Lloyd's algorithm, pure Rust
- Label clusters by mean duration: highest = **"Peak"**, middle = **"Moderate"**, lowest = **"Distracted"**

```rust
use linfa::prelude::*;
use linfa_clustering::KMeans;
use ndarray::Array2;

let data: Array2<f32> = /* build from sessions */;
let dataset = linfa::Dataset::from(data);
let model = KMeans::params(3)
    .max_n_iterations(200)
    .tolerance(1e-5)
    .fit(&dataset)
    .expect("KMeans failed");
let assignments = model.predict(&dataset);
```

#### 3.3 Peak-hour detection
- Group sessions by `hour_of_day` (0–23 buckets)
- For each hour, compute: `score = mean_duration * session_count` (rewards both quality and consistency)
- Sort descending → top 3 hours = recommended focus windows
- Format as "10:00 – 11:00" etc.

#### 3.4 Moving average on daily totals (7-day window)
- Group sessions by calendar date, sum durations per day
- Apply simple 7-day centered moving average: `avg[i] = sum(days[i-3..=i+3]) / count`
- Detect trend: if last 3 days average > overall average → "📈 Focus trending up", else "📉 Consider protecting your focus blocks"

#### 3.5 Distraction rate
- `distraction_rate = interrupted_sessions / total_sessions * 100`
- Threshold: > 30% distracted → trigger a specific recommendation in the report

---

## Phase 4 — Focus Health Report Generator

**Goal:** After each session (or via a `--report` CLI flag), generate a readable HTML report and open it in the browser.

### Steps

- Add to `Cargo.toml`:
  ```toml
  # Nothing new needed — use std::fs + format! strings for HTML
  ```
- Write `src/report.rs` with a `generate_report(stats: &AnalyticsResult) -> String` function
- Inline HTML/CSS using the existing **Monolithic Precision design system** from `M3Design.md`:
  - Background: `#0b0e12`, surface: `#1D2024`, accent: `#bbdaff`
  - Font: Inter (load from Google Fonts CDN inline in `<head>`)
  - Use the same card/list-item pattern from your firmware dashboard
- Write the HTML to a temp file: `std::env::temp_dir().join("focus_report.html")`
- Open it with the platform default browser:
  ```rust
  #[cfg(target_os = "linux")]
  Command::new("xdg-open").arg(&report_path).spawn();

  #[cfg(target_os = "windows")]
  Command::new("explorer").arg(&report_path).spawn();
  ```
- Report sections to include:
  1. **Weekly Summary** — total deep work hours, total sessions, this week vs last week delta
  2. **Top Focus Windows** — top 3 recommended hours with their score
  3. **Cluster Breakdown** — count/avg duration for Peak, Moderate, Distracted clusters
  4. **Trend Line** — 7-day moving average as a simple ASCII bar chart (or HTML `<div>` width trick)
  5. **AI Recommendation** — one plain-English paragraph generated from the stats

### CLI flag
- Parse `std::env::args()` manually (no need for `clap` yet):
  ```rust
  if args.contains(&"--report".to_string()) {
      run_report_only();
      return;
  }
  ```

---

## Phase 5 — Modern Rust Housekeeping

**Goal:** Remove legacy patterns. Improve code structure before it gets harder.

### 5.1 Replace `lazy_static` with `std::sync::LazyLock`
- `LazyLock` is stable since **Rust 1.80** — no external dependency needed
- Before: `lazy_static! { static ref ORIGINAL_WALLPAPER_PATH: Mutex<Option<String>> = Mutex::new(None); }`
- After:
  ```rust
  static ORIGINAL_WALLPAPER_PATH: LazyLock<Mutex<Option<String>>> =
      LazyLock::new(|| Mutex::new(None));
  ```
- Remove `lazy_static` from `Cargo.toml` entirely

### 5.2 Module structure
Split `src/main.rs` (currently ~500 lines and growing) into:
```
src/
├── main.rs          # entry point, main loop only
├── config.rs        # load_focus_apps(), AppsConfig, constants
├── automation.rs    # launch_app(), terminate_app(), wallpaper functions
├── discovery.rs     # discover_device(), mDNS logic
├── session.rs       # SessionRecord, logging, file I/O
├── analytics.rs     # k-means, peak-hour detection, moving average
└── report.rs        # HTML report generation, browser open
```

### 5.3 Error handling
- Replace `unwrap()` calls in non-fatal paths with `?` propagation or `if let Err(e)`
- `main()` should return `anyhow::Result<()>` — add `anyhow = "1"` to deps
- Gives you proper error context on any crash: `"Failed to write sessions.json: Permission denied (os error 13)"`

### 5.4 Remove `hello_gpui/` or graduate it
- It's currently a dead-end experiment with stale GPUI API calls
- Decision: either delete it and track the GUI idea in the roadmap, or wire it to the current `src/` logic
- Recommendation: **delete for now**, open a GitHub issue titled "GUI: GPUI integration" to track it properly

---

## Phase 6 — Async Refactor (Main Client)

**Goal:** Replace the `blocking reqwest + thread::sleep` loop with proper async. Cleaner cancellation, lower resource usage, future-proofs the GUI path.

### Steps

- Add `tokio = { version = "1", features = ["full"] }` to main `Cargo.toml`
- Switch `reqwest` to async: `reqwest = { version = "0.12", features = ["json"] }` (drop `blocking` feature)
- Annotate `main` with `#[tokio::main]`
- Replace `thread::sleep(Duration::from_secs(3))` with `tokio::time::sleep(...).await`
- The mDNS discovery (`mdns-sd`) uses its own thread internally — wrap `discover_device` in `tokio::task::spawn_blocking`
- All automation functions (`activate_focus_mode`, `deactivate_focus_mode`) stay sync — call them from `spawn_blocking` since they shell out to OS commands

### Why now and not earlier
- The mock server (Phase 1) already uses tokio, so the dependency is already in the workspace
- Sessions + analytics (Phases 2–4) are sync file I/O — they don't need async
- Doing this before adding a GUI is the right order

---

## Phase 7 — System Integration: Autostart & Background

**Goal:** The client should start automatically with the desktop session and run silently in the background.

### Linux (systemd user service)
- Create `focus-totem.service`:
  ```ini
  [Unit]
  Description=Focus Totem Client
  After=graphical-session.target

  [Service]
  Type=simple
  ExecStart=/path/to/focus_client_rust
  Restart=on-failure
  RestartSec=5s
  Environment=DISPLAY=:0

  [Install]
  WantedBy=graphical-session.target
  ```
- Install: `cp focus-totem.service ~/.config/systemd/user/ && systemctl --user enable --now focus-totem`
- Generate a `--install-service` CLI flag that writes and enables this automatically

### Windows (startup via registry)
- On `--install-service`, write the exe path to:
  `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
- Use the `winreg` crate: `winreg = "0.52"`
- Conditionally compile: `#[cfg(target_os = "windows")]`

### Tray icon (optional, post-autostart)
- `tray-icon = "0.19"` crate for a minimal system tray presence on both platforms
- States: searching (grey), connected (blue), focus active (green)
- Right-click menu: "Force Deactivate Focus" / "View Report" / "Quit"

---

## Phase 8 — ESP32 Enhancements (for when hardware is back)

**Goal:** Harden the firmware and add interactivity to the dashboard.

- **Manual toggle from dashboard:** Add a `POST /toggle` endpoint on the ESP32 that flips an `isFocusOverride` bool. The `/status` endpoint returns `FOCUS_OFF` when override is set. Lets you test deactivation without physically unplugging the device.
- **LED status indicator:** GPIO output on a pin → green LED for FOCUS_ON, off for idle. Wire from the `handleStatus` function.
- **Session count on dashboard:** Increment a `uint32_t sessionCount` in SRAM each time `/status` is polled and the focus state is active. Show it on the dashboard.
- **Button debounce interrupt:** Wire a physical push button → ISR on GPIO → toggle focus state directly from hardware, no HTTP needed. Use `attachInterrupt()` + `portENTER_CRITICAL` for thread safety with FreeRTOS.
- **OTA firmware updates:** `ArduinoOTA` library. Add to `setup()`, call `ArduinoOTA.handle()` in `loop()`. Lets you push new firmware over Wi-Fi from Arduino IDE without USB.

---

## Dependency Summary (end state)

```toml
[dependencies]
# Existing
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
toml        = "0.8"
reqwest     = { version = "0.12", features = ["json"] }   # async after Phase 6
wallpaper   = "3.2"
mdns-sd     = "0.15"

# New
tokio       = { version = "1", features = ["full"] }      # Phase 1 / Phase 6
axum        = "0.8"                                        # mock_server crate only
chrono      = { version = "0.4", features = ["serde"] }   # Phase 2
dirs        = "5"                                          # Phase 2
linfa       = "0.7"                                        # Phase 3
linfa-clustering = "0.7"                                   # Phase 3
ndarray     = "0.16"                                       # Phase 3
anyhow      = "1"                                          # Phase 5

# Windows-only
[target.'cfg(windows)'.dependencies]
winreg = "0.52"                                            # Phase 7

# Removed
# lazy_static — replaced by std::sync::LazyLock (Rust 1.80+)
```

---

## Execution Order (Recommended)

| Priority | Phase | Why first |
|---|---|---|
| 🔴 1 | **Phase 1** — Rust mock server | Unblocks ESP-free development immediately, kills Python dep |
| 🔴 2 | **Phase 2** — Session logging | Everything downstream (AI, reports) depends on this data |
| 🟠 3 | **Phase 5.1–5.3** — Housekeeping | Do this before the codebase gets bigger, or you'll regret it |
| 🟠 4 | **Phase 3** — Analytics | Core AI module for the CEP proposal |
| 🟠 5 | **Phase 4** — Report generator | Closes the AI loop, gives you something to demo |
| 🟡 6 | **Phase 6** — Async refactor | Quality of life, needed before any GUI work |
| 🟡 7 | **Phase 7** — Autostart | Usability polish |
| 🟢 8 | **Phase 8** — ESP32 hardening | Hardware-dependent, do when device is available |

```

`Updated.md`:

```md
Project Path: focus_client_rust

Source Tree:

```txt
focus_client_rust
├── Cargo.toml
├── Multithreaded_Dashboard
│   ├── M3-Redesign.ino
│   ├── M3Design.md
│   ├── Multithreaded_Dashboard.ino
│   └── test.html
├── README.md
├── ROADMAP.md
├── apps.toml
├── build.rs
├── focus_wallpaper.jpg
├── hello_gpui
│   ├── Cargo.toml
│   ├── src
│   │   └── main.rs
│   └── steps.txt
├── manifest.xml
├── mock_server
│   ├── Cargo.toml
│   └── src
│       └── main.rs
├── src
│   ├── main.rs
│   └── session.rs
└── totem.cpp

```

`Cargo.toml`:

```toml
[package]
name = "focus_client_rust"
version = "0.1.0"
edition = "2021"

[workspace]
members = [".", "mock_server"]
resolver = "2"

[dependencies]
lazy_static = "1.5.0"
mdns-sd = "0.15.1" # For mDNS service discovery
reqwest = { version = "0.12.23", features = ["blocking"] } # For making HTTP requests
wallpaper = "3.2.0"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1"
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"

# --- ADD THIS ENTIRE SECTION ---
[build-dependencies]
embed-manifest = "1.4.0"

```

`Multithreaded_Dashboard/M3-Redesign.ino`:

```ino
#include <WiFi.h>
#include <ESPmDNS.h>
#include <WebServer.h>
#include <ArduinoJson.h>

// --- IMPORTANT: CHANGE THESE TO YOUR WI-FI CREDENTIALS ---
const char *ssid = "Faizy's A54";
const char *password = "12345678";

WebServer server(80);

// --- Shared Status Variables ---
struct Status
{
  bool wifiConnecting;
  bool wifiConnected;
};
Status sharedStatus;

// --- FreeRTOS Handles ---
TaskHandle_t statusMonitorTaskHandle = NULL;
SemaphoreHandle_t statusMutex;

// --- THE (REDESIGNED) HTML PAGE ---
const char *dashboardHTML = R"rawliteral(
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Focus Totem Dashboard</title>
  
  <!-- The Editorial Voice: Typography Base -->
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Epilogue:wght@700&family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
  
  <style>
    :root {
      /* Surface Hierarchy (Dark Mode Default) */
      --bg-void: #0b0e12;
      --surface-low: #1D2024;
      --surface-highest: #2B2D31;
      --tertiary-muted: #d1f3dc;
      --primary-accent: #bbdaff;
      
      /* Typography Colors */
      --text-primary: #ffffff;
      --text-secondary: #9ca3af;
    }

    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }

    body {
      font-family: 'Inter', sans-serif;
      background-color: var(--bg-void);
      color: var(--text-primary);
      -webkit-font-smoothing: antialiased;
      min-height: 100vh;
      display: flex;
      justify-content: center;
      padding: 64px 24px;
    }

    .monolith-wrapper {
      width: 100%;
      max-width: 800px;
      display: flex;
      flex-direction: column;
      gap: 56px; /* Massive breathing room per Do's and Don'ts */
    }

    /* Typography: Statement Styles */
    .display-lg {
      font-family: 'Epilogue', sans-serif;
      font-size: 3.5rem;
      font-weight: 700;
      letter-spacing: -0.02em;
      line-height: 1.1;
      color: var(--text-primary);
    }

    .headline-md {
      font-family: 'Epilogue', sans-serif;
      font-size: 1.5rem;
      font-weight: 700;
      letter-spacing: -0.02em;
      color: var(--text-primary);
    }

    /* Typography: High-Density Data Workhorse */
    .body-md {
      font-family: 'Inter', sans-serif;
      font-size: 1rem;
      color: var(--text-secondary);
      line-height: 1.5;
      margin-top: 16px;
    }

    .label {
      font-family: 'Inter', sans-serif;
      font-size: 1rem;
      font-weight: 500;
      color: var(--text-primary);
    }

    .value {
      font-family: 'Inter', sans-serif;
      font-size: 1rem;
      color: var(--text-secondary);
      display: flex;
      align-items: center;
    }

    .badge {
      background-color: var(--surface-highest);
      color: var(--primary-accent);
      padding: 6px 16px;
      border-radius: 9999px;
      font-size: 0.875rem;
      font-weight: 600;
      margin-left: 16px;
      vertical-align: middle;
    }

    /* Tonal Stepping: Level 0 -> Level 1 */
    .dashboard-surface {
      background-color: var(--surface-low);
      border-radius: 24px;
      padding: 40px;
      display: flex;
      flex-direction: column;
      gap: 32px;
    }

    /* The Divider Rule: No 1px lines */
    .list-container {
      display: flex;
      flex-direction: column;
      gap: 1.4rem; 
    }

    /* High-Contrast Touch Targets & Cards */
    .list-item {
      background-color: var(--surface-highest); /* Tonal shift milled inside the surface */
      border-radius: 16px;
      min-height: 56px; /* Guaranteed accessible hit area */
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 0 24px;
    }

    /* Buttons & Chips: Primary Actions */
    .btn-primary {
      background-color: var(--primary-accent);
      color: var(--bg-void); 
      border: none;
      border-radius: 9999px; /* Pill Shape */
      min-height: 56px; /* Strict 56px rule */
      padding: 0 40px;
      font-family: 'Inter', sans-serif;
      font-size: 1.125rem;
      font-weight: 600;
      cursor: pointer;
      transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
      display: inline-flex;
      align-items: center;
      justify-content: center;
    }

    .btn-primary:active {
      transform: scale(0.97);
    }

    /* Status Indicators */
    .status-indicator {
      display: inline-block;
      width: 12px;
      height: 12px;
      border-radius: 50%;
      margin-right: 12px;
    }

    .status-connected {
      background-color: var(--tertiary-muted);
    }

    .status-disconnected {
      background-color: #ff5e5e; /* Stark contrast error state */
    }

    .status-connecting {
      background-color: var(--primary-accent);
      animation: pulse 1.5s ease-in-out infinite;
    }

    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.2; }
    }

    .actions-wrapper {
      display: flex;
      justify-content: flex-start;
    }

    /* Form Factor Adaptations */
    @media (max-width: 600px) {
      body { padding: 32px 16px; }
      .display-lg { font-size: 2.75rem; }
      .dashboard-surface { padding: 24px; }
      .list-item {
        flex-direction: column;
        align-items: flex-start;
        justify-content: center;
        gap: 8px;
        padding: 16px 24px;
      }
    }
  </style>
</head>
<body>
  <div class="monolith-wrapper">
    
    <!-- The Hierarchy Rule Applied -->
    <header class="header">
      <h1 class="display-lg">Focus Totem</h1>
      <p class="body-md">Real-time status monitoring <span class="badge">Dual-Core</span></p>
    </header>

    <main class="dashboard-surface">
      <h2 class="headline-md">System Status</h2>
      
      <div class="list-container">
        <div class="list-item">
          <span class="label">WiFi Connection</span>
          <span class="value">
            <span class="status-indicator" id="wifi-indicator"></span>
            <span id="wifi-status">Loading...</span>
          </span>
        </div>

        <div class="list-item">
          <span class="label">Network Name (SSID)</span>
          <span class="value" id="ssid">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">IP Address</span>
          <span class="value" id="ip-address">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">mDNS Hostname</span>
          <span class="value" id="mdns">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">Signal Strength</span>
          <span class="value" id="rssi">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">Uptime</span>
          <span class="value" id="uptime">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">Free Heap Memory</span>
          <span class="value" id="heap">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">Web Server Core</span>
          <span class="value" id="web-core">Loading...</span>
        </div>

        <div class="list-item">
          <span class="label">Monitor Core</span>
          <span class="value" id="monitor-core">Loading...</span>
        </div>
      </div>
    </main>

    <div class="actions-wrapper">
      <button class="btn-primary" id="refresh-btn">Refresh Status</button>
    </div>

  </div>

  <script>
    async function updateStatus() {
      try {
        const response = await fetch('/api/status');
        if (!response.ok) {
            console.error("Failed to fetch status, server responded with:", response.status);
            return;
        }
        const data = await response.json();

        const wifiIndicator = document.getElementById('wifi-indicator');
        const wifiStatus = document.getElementById('wifi-status');

        if (data.wifiConnected) {
          wifiIndicator.className = 'status-indicator status-connected';
          wifiStatus.textContent = 'Connected';
        } else if (data.wifiConnecting) {
          wifiIndicator.className = 'status-indicator status-connecting';
          wifiStatus.textContent = 'Connecting...';
        } else {
          wifiIndicator.className = 'status-indicator status-disconnected';
          wifiStatus.textContent = 'Disconnected';
        }

        document.getElementById('ssid').textContent = data.ssid || 'N/A';
        document.getElementById('ip-address').textContent = data.ipAddress || 'N/A';
        document.getElementById('mdns').textContent = data.mdns || 'N/A';
        document.getElementById('rssi').textContent = data.rssi || 'N/A';
        document.getElementById('uptime').textContent = data.uptime || 'N/A';
        document.getElementById('heap').textContent = data.freeHeap || 'N/A';
        document.getElementById('web-core').textContent = data.webCore || 'N/A';
        document.getElementById('monitor-core').textContent = data.monitorCore || 'N/A';
      } catch (error) {
        console.error('Error fetching status:', error);
      }
    }

    document.getElementById('refresh-btn').addEventListener('click', updateStatus);

    // Initial load
    updateStatus();

    // Auto-refresh every 5 seconds
    setInterval(updateStatus, 5000);
  </script>
</body>
</html>
)rawliteral";

String formatUptime(unsigned long milliseconds)
{
  unsigned long seconds = milliseconds / 1000;
  unsigned long minutes = seconds / 60;
  unsigned long hours = minutes / 60;
  unsigned long days = hours / 24;
  seconds %= 60;
  minutes %= 60;
  hours %= 24;
  String uptime = "";
  if (days > 0)
    uptime += String(days) + "d ";
  if (hours > 0)
    uptime += String(hours) + "h ";
  if (minutes > 0)
    uptime += String(minutes) + "m ";
  uptime += String(seconds) + "s";
  return uptime;
}

void handleRoot()
{
  server.send(200, "text/html", dashboardHTML);
}

void handleApiStatus()
{
  StaticJsonDocument<256> doc;
  xSemaphoreTake(statusMutex, portMAX_DELAY);
  bool isConnecting = sharedStatus.wifiConnecting;
  bool isConnected = sharedStatus.wifiConnected;
  xSemaphoreGive(statusMutex);

  doc["wifiConnected"] = isConnected;
  doc["wifiConnecting"] = isConnecting;
  doc["ssid"] = ssid;
  doc["mdns"] = "focus-totem.local";
  doc["uptime"] = formatUptime(millis());
  doc["freeHeap"] = String(ESP.getFreeHeap()) + " bytes";

  if (isConnected)
  {
    doc["ipAddress"] = WiFi.localIP().toString();
    doc["rssi"] = String(WiFi.RSSI()) + " dBm";
  }
  else
  {
    doc["ipAddress"] = "N/A";
    doc["rssi"] = "N/A";
  }

  doc["webCore"] = "Core " + String(xPortGetCoreID());
  doc["monitorCore"] = "Core 0";

  String jsonResponse;
  serializeJson(doc, jsonResponse);

  server.sendHeader("Access-Control-Allow-Origin", "*");
  server.send(200, "application/json", jsonResponse);
}

void handleStatus()
{
  server.send(200, "text/plain", "FOCUS_ON");
}

void statusMonitorTask(void *parameter)
{
  for (;;)
  {
    bool isConnectedNow = (WiFi.status() == WL_CONNECTED);
    xSemaphoreTake(statusMutex, portMAX_DELAY);

    if (isConnectedNow && !sharedStatus.wifiConnected)
    {
      Serial.println("WiFi has connected!");
      sharedStatus.wifiConnected = true;
      sharedStatus.wifiConnecting = false;
    }
    else if (!isConnectedNow && sharedStatus.wifiConnected)
    {
      Serial.println("WiFi has disconnected!");
      sharedStatus.wifiConnected = false;
      sharedStatus.wifiConnecting = true;
      WiFi.reconnect();
    }

    xSemaphoreGive(statusMutex);

    vTaskDelay(5000 / portTICK_PERIOD_MS);
  }
}

void setup()
{
  Serial.begin(115200);
  Serial.println("\n=== ESP32 Dual-Core Focus Totem (v3 - Stable) ===");

  statusMutex = xSemaphoreCreateMutex();

  sharedStatus.wifiConnected = false;
  sharedStatus.wifiConnecting = true;
  WiFi.begin(ssid, password);
  Serial.print("Initial WiFi connection attempt...");
  while (WiFi.status() != WL_CONNECTED)
  {
    delay(500);
    Serial.print(".");
  }
  Serial.println("\nWiFi connected!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());

  sharedStatus.wifiConnected = true;
  sharedStatus.wifiConnecting = false;

  if (!MDNS.begin("focus-totem"))
  {
    Serial.println("Error setting up mDNS responder!");
    while (1)
      ;
  }
  MDNS.addService("http", "tcp", 80);

  server.on("/", HTTP_GET, handleRoot);
  server.on("/api/status", HTTP_GET, handleApiStatus);
  server.on("/status", HTTP_GET, handleStatus);
  server.begin();

  Serial.println("Web server running on Core 1 (main loop)");
  Serial.println("Dashboard: http://focus-totem.local/");

  // Create Status Monitor Task on Core 0
  xTaskCreatePinnedToCore(
      statusMonitorTask,
      "StatusMonitorTask",
      10000,
      NULL,
      1,
      &statusMonitorTaskHandle, // <-- THIS IS THE CORRECTED LINE
      0);

  Serial.println("Status monitor running on Core 0");
}

void loop()
{
  // The main loop() is now our dedicated Web Server task for Core 1
  server.handleClient();
  // Add a small delay to prevent the watchdog timer from triggering
  // and to allow other lower-priority tasks on the same core to run.
  vTaskDelay(2 / portTICK_PERIOD_MS);
}
```

`Multithreaded_Dashboard/M3Design.md`:

```md
# Design System Specification: The Monolithic Precision System

## 1. Overview & Creative North Star
**Creative North Star: "The Architectural Monolith"**

This design system rejects the ephemeral fluff of modern web trends in favor of grounded, architectural permanence. The aesthetic is defined by **Tonal Brutalism**: a high-utility, high-sophistication approach that uses solid masses of color to define space. 

By stripping away blurs, glassmorphism, and traditional drop shadows, we rely on the purity of the Material 3 "Expressive" logic. We create depth through "Carved Surfaces"—where the UI feels like a single block of obsidian with functional areas precisely milled into the surface. The result is an interface that feels authoritative, secure, and hyper-legible.

---

## 2. Colors & Surface Logic
The palette is rooted in deep minerals and high-contrast accents. We prioritize functional clarity over decorative gradients.

### The "No-Line" Rule
**Explicit Instruction:** 1px solid borders are strictly prohibited for sectioning or containment. 
Structure must be achieved through **Tonal Stepping**. To separate a sidebar from a main feed, or a header from a body, transition between `surface-container-low` (#1D2024) and `surface-container-highest` (#2B2D31). This creates a "milled" look where components appear to be physically inset or embossed within the interface.

### Surface Hierarchy (Dark Mode Default)
| Token | Hex | Role |
| :--- | :--- | :--- |
| **background** | #0b0e12 | The foundational "base" layer. |
| **surface-container-low** | #1D2024 | Primary background for main content areas and secondary sections. |
| **surface-container-highest**| #2B2D31 | Elevated surfaces: Cards, active modals, and high-priority containers. |
| **tertiary-container** | #d1f3dc | **Visited States:** A muted, sophisticated dark green to denote historical navigation. |
| **primary** | #bbdaff | Actionable elements and brand highlights. |

---

## 3. Typography: The Editorial Voice
We utilize a high-contrast scale to ensure the "Expressive" nature of the system is felt immediately. 

*   **Display & Headlines (Epilogue):** These are your "Statement" styles. Use `display-lg` (3.5rem) and `headline-lg` (2rem) with tight letter-spacing (-0.02em) to create a bold, editorial feel. These should feel like headlines in a premium architectural magazine.
*   **Body & Labels (Inter):** Reserved for high-density data. While the headers are expressive, the body remains a workhorse—clean, legible, and utilitarian.

**The Hierarchy Rule:** Never pair two "Display" sizes together. Use a bold `headline-md` for titles and immediately drop to `body-md` for descriptions to maximize the dynamic range of the layout.

---

## 4. Elevation & Depth: Tonal Stacking
Since shadows and blurs are forbidden, we use **The Stacking Principle** to communicate importance.

1.  **Level 0 (The Void):** `surface-container-low` (#1D2024). Use this for the largest background areas.
2.  **Level 1 (The Object):** `surface-container-highest` (#2B2D31). Use this for cards and list items. 
3.  **Level 2 (The Focus):** `primary` (#bbdaff). Used for the most critical interactive state.

**Ghost Borders (The Exception):** If high-density data requires a container but a background shift is too heavy, use `outline-variant` (#424850) at **15% opacity**. This creates a "perceived" edge that assists eye-tracking without introducing visual noise.

---

## 5. Components

### High-Contrast Touch Targets
Every interactive list element or tile must maintain a **minimum height of 56px**. This ensures the "Expressive" system remains accessible and feels premium under-thumb.

### Buttons & Chips
*   **Shape:** `rounded-full` (Pill shape).
*   **Primary:** Solid `primary` background with `on-primary` text. No shadows.
*   **Secondary:** `surface-container-highest` background.
*   **Interaction:** On press, shift the tonal value one step higher (e.g., from `surface-container-low` to `surface-container-highest`).

### Cards & Lists
*   **Rounding:** `rounded-[16px]`.
*   **The Divider Rule:** Forbid 1px dividers. Use a `1.4rem` (Spacing 4) vertical gap to separate list items. If separation is visually required, use a 1-step tonal shift between the list item and the background.
*   **Visited State:** Items that have been viewed or "planned" should transition their container or a secondary indicator to `tertiary-container` (Muted Green).

### The Bottom Sheet (Signature Component)
*   **Rounding:** `rounded-t-[32px]`.
*   **Style:** Must use `surface-container-highest` (#2B2D31) to contrast sharply against the lower-level background.
*   **Context:** Used for branch filtering and appointment confirmation.

### Input Fields
*   **Style:** Filled (not outlined).
*   **Background:** `surface-container-highest`.
*   **Active State:** A bottom-heavy `2px` border using the `primary` token. No glow/blur.

---

## 6. Do’s and Don’ts

### Do
*   **Do** use massive "Display" typography for branch names or empty states.
*   **Do** use the full spacing scale (up to `spacing-24`) to create "Breathing Room" around monolithic blocks.
*   **Do** rely on `surface-container` tiers to group related information.
*   **Do** ensure all primary actions use the `primary` (#bbdaff) color to pop against the dark mode.

### Don't
*   **Don't** use `drop-shadow`. If an element needs to stand out, make it a lighter tonal hex.
*   **Don't** use `backdrop-blur`. Backgrounds must remain solid and opaque.
*   **Don't** use 1px lines to separate content. Use whitespace or color shifts.
*   **Don't** cram data. If the touch target is less than 56px, the design is a failure of this system.

```

`Multithreaded_Dashboard/Multithreaded_Dashboard.ino`:

```ino
#include <WiFi.h>
#include <ESPmDNS.h>
#include <WebServer.h>
#include <ArduinoJson.h>

// --- IMPORTANT: CHANGE THESE TO YOUR WI-FI CREDENTIALS ---
const char *ssid = "Faizy's A54";
const char *password = "12345678";

WebServer server(80);

// --- Shared Status Variables ---
struct Status
{
  bool wifiConnecting;
  bool wifiConnected;
};
Status sharedStatus;

// --- FreeRTOS Handles ---
TaskHandle_t statusMonitorTaskHandle = NULL;
SemaphoreHandle_t statusMutex;

// --- THE (UNCHANGED) HTML PAGE ---
const char *dashboardHTML = R"rawliteral(
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Focus Totem Dashboard</title>
  <link href="https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;700&display=swap" rel="stylesheet">
  <script type="importmap">
    {
      "imports": {
        "@material/web/": "https://esm.run/@material/web/"
      }
    }
  </script>
  <script type="module">
    import '@material/web/all.js';
    import {styles as typescaleStyles} from '@material/web/typography/md-typescale-styles.js';
    document.adoptedStyleSheets.push(typescaleStyles.styleSheet);
  </script>
  <style>
    :root {
      --md-sys-color-primary: #6750A4;
      --md-sys-color-on-primary: #FFFFFF;
      --md-sys-color-primary-container: #EADDFF;
      --md-sys-color-on-primary-container: #21005D;
      --md-sys-color-secondary: #625B71;
      --md-sys-color-on-secondary: #FFFFFF;
      --md-sys-color-surface: #FEF7FF;
      --md-sys-color-on-surface: #1D1B20;
      --md-sys-color-surface-variant: #E7E0EC;
      --md-sys-color-on-surface-variant: #49454F;
      --md-sys-color-error: #B3261E;
      --md-sys-color-on-error: #FFFFFF;
    }
    body {
      font-family: 'Roboto', sans-serif;
      margin: 0;
      padding: 0;
      background-color: var(--md-sys-color-surface);
      color: var(--md-sys-color-on-surface);
    }
    .container {
      max-width: 800px;
      margin: 0 auto;
      padding: 24px;
    }
    .header {
      margin-bottom: 32px;
    }
    .status-card {
      background: var(--md-sys-color-surface-variant);
      border-radius: 12px;
      padding: 24px;
      margin-bottom: 16px;
    }
    .status-row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 0;
      border-bottom: 1px solid rgba(0,0,0,0.1);
    }
    .status-row:last-child {
      border-bottom: none;
    }
    .status-indicator {
      display: inline-block;
      width: 12px;
      height: 12px;
      border-radius: 50%;
      margin-right: 8px;
    }
    .status-connected {
      background-color: #4CAF50;
    }
    .status-disconnected {
      background-color: #F44336;
    }
    .status-connecting {
      background-color: #FF9800;
      animation: pulse 1.5s ease-in-out infinite;
    }
    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.5; }
    }
    .refresh-container {
      margin-top: 24px;
      text-align: center;
    }
    .core-badge {
      display: inline-block;
      background: var(--md-sys-color-primary-container);
      color: var(--md-sys-color-on-primary-container);
      padding: 4px 12px;
      border-radius: 16px;
      font-size: 12px;
      margin-left: 8px;
    }
  </style>
</head>
<body>
  <div class="container">
    <div class="header">
      <h1 class="md-typescale-display-small">Focus Totem Dashboard</h1>
      <p class="md-typescale-body-medium">Real-time status monitoring <span class="core-badge">Dual-Core</span></p>
    </div>

    <div class="status-card">
      <h2 class="md-typescale-title-large">System Status</h2>

      <div class="status-row">
        <span class="md-typescale-body-large">WiFi Connection</span>
        <span class="md-typescale-body-medium">
          <span class="status-indicator" id="wifi-indicator"></span>
          <span id="wifi-status">Loading...</span>
        </span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Network Name (SSID)</span>
        <span class="md-typescale-body-medium" id="ssid">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">IP Address</span>
        <span class="md-typescale-body-medium" id="ip-address">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">mDNS Hostname</span>
        <span class="md-typescale-body-medium" id="mdns">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Signal Strength</span>
        <span class="md-typescale-body-medium" id="rssi">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Uptime</span>
        <span class="md-typescale-body-medium" id="uptime">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Free Heap Memory</span>
        <span class="md-typescale-body-medium" id="heap">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Web Server Core</span>
        <span class="md-typescale-body-medium" id="web-core">Loading...</span>
      </div>

      <div class="status-row">
        <span class="md-typescale-body-large">Monitor Core</span>
        <span class="md-typescale-body-medium" id="monitor-core">Loading...</span>
      </div>
    </div>

    <div class="refresh-container">
      <md-filled-button id="refresh-btn">Refresh Status</md-filled-button>
    </div>
  </div>

  <script>
    async function updateStatus() {
      try {
        const response = await fetch('/api/status');
        if (!response.ok) {
            console.error("Failed to fetch status, server responded with:", response.status);
            return;
        }
        const data = await response.json();

        const wifiIndicator = document.getElementById('wifi-indicator');
        const wifiStatus = document.getElementById('wifi-status');

        if (data.wifiConnected) {
          wifiIndicator.className = 'status-indicator status-connected';
          wifiStatus.textContent = 'Connected';
        } else if (data.wifiConnecting) {
          wifiIndicator.className = 'status-indicator status-connecting';
          wifiStatus.textContent = 'Connecting...';
        } else {
          wifiIndicator.className = 'status-indicator status-disconnected';
          wifiStatus.textContent = 'Disconnected';
        }

        document.getElementById('ssid').textContent = data.ssid || 'N/A';
        document.getElementById('ip-address').textContent = data.ipAddress || 'N/A';
        document.getElementById('mdns').textContent = data.mdns || 'N/A';
        document.getElementById('rssi').textContent = data.rssi || 'N/A';
        document.getElementById('uptime').textContent = data.uptime || 'N/A';
        document.getElementById('heap').textContent = data.freeHeap || 'N/A';
        document.getElementById('web-core').textContent = data.webCore || 'N/A';
        document.getElementById('monitor-core').textContent = data.monitorCore || 'N/A';
      } catch (error) {
        console.error('Error fetching status:', error);
      }
    }

    document.getElementById('refresh-btn').addEventListener('click', updateStatus);

    // Initial load
    updateStatus();

    // Auto-refresh every 5 seconds
    setInterval(updateStatus, 5000);
  </script>
</body>
</html>
)rawliteral";

String formatUptime(unsigned long milliseconds)
{
  unsigned long seconds = milliseconds / 1000;
  unsigned long minutes = seconds / 60;
  unsigned long hours = minutes / 60;
  unsigned long days = hours / 24;
  seconds %= 60;
  minutes %= 60;
  hours %= 24;
  String uptime = "";
  if (days > 0)
    uptime += String(days) + "d ";
  if (hours > 0)
    uptime += String(hours) + "h ";
  if (minutes > 0)
    uptime += String(minutes) + "m ";
  uptime += String(seconds) + "s";
  return uptime;
}

void handleRoot()
{
  server.send(200, "text/html", dashboardHTML);
}

void handleApiStatus()
{
  StaticJsonDocument<256> doc;
  xSemaphoreTake(statusMutex, portMAX_DELAY);
  bool isConnecting = sharedStatus.wifiConnecting;
  bool isConnected = sharedStatus.wifiConnected;
  xSemaphoreGive(statusMutex);

  doc["wifiConnected"] = isConnected;
  doc["wifiConnecting"] = isConnecting;
  doc["ssid"] = ssid;
  doc["mdns"] = "focus-totem.local";
  doc["uptime"] = formatUptime(millis());
  doc["freeHeap"] = String(ESP.getFreeHeap()) + " bytes";

  if (isConnected)
  {
    doc["ipAddress"] = WiFi.localIP().toString();
    doc["rssi"] = String(WiFi.RSSI()) + " dBm";
  }
  else
  {
    doc["ipAddress"] = "N/A";
    doc["rssi"] = "N/A";
  }

  doc["webCore"] = "Core " + String(xPortGetCoreID());
  doc["monitorCore"] = "Core 0";

  String jsonResponse;
  serializeJson(doc, jsonResponse);

  server.sendHeader("Access-Control-Allow-Origin", "*");
  server.send(200, "application/json", jsonResponse);
}

void handleStatus()
{
  server.send(200, "text/plain", "FOCUS_ON");
}

void statusMonitorTask(void *parameter)
{
  for (;;)
  {
    bool isConnectedNow = (WiFi.status() == WL_CONNECTED);
    xSemaphoreTake(statusMutex, portMAX_DELAY);

    if (isConnectedNow && !sharedStatus.wifiConnected)
    {
      Serial.println("WiFi has connected!");
      sharedStatus.wifiConnected = true;
      sharedStatus.wifiConnecting = false;
    }
    else if (!isConnectedNow && sharedStatus.wifiConnected)
    {
      Serial.println("WiFi has disconnected!");
      sharedStatus.wifiConnected = false;
      sharedStatus.wifiConnecting = true;
      WiFi.reconnect();
    }

    xSemaphoreGive(statusMutex);

    vTaskDelay(5000 / portTICK_PERIOD_MS);
  }
}

void setup()
{
  Serial.begin(115200);
  Serial.println("\n=== ESP32 Dual-Core Focus Totem (v3 - Stable) ===");

  statusMutex = xSemaphoreCreateMutex();

  sharedStatus.wifiConnected = false;
  sharedStatus.wifiConnecting = true;
  WiFi.begin(ssid, password);
  Serial.print("Initial WiFi connection attempt...");
  while (WiFi.status() != WL_CONNECTED)
  {
    delay(500);
    Serial.print(".");
  }
  Serial.println("\nWiFi connected!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());

  sharedStatus.wifiConnected = true;
  sharedStatus.wifiConnecting = false;

  if (!MDNS.begin("focus-totem"))
  {
    Serial.println("Error setting up mDNS responder!");
    while (1)
      ;
  }
  MDNS.addService("http", "tcp", 80);

  server.on("/", HTTP_GET, handleRoot);
  server.on("/api/status", HTTP_GET, handleApiStatus);
  server.on("/status", HTTP_GET, handleStatus);
  server.begin();

  Serial.println("Web server running on Core 1 (main loop)");
  Serial.println("Dashboard: http://focus-totem.local/");

  // Create Status Monitor Task on Core 0
  xTaskCreatePinnedToCore(
      statusMonitorTask,
      "StatusMonitorTask",
      10000,
      NULL,
      1,
      &statusMonitorTaskHandle, // <-- THIS IS THE CORRECTED LINE
      0);

  Serial.println("Status monitor running on Core 0");
}

void loop()
{
  // The main loop() is now our dedicated Web Server task for Core 1
  server.handleClient();
  // Add a small delay to prevent the watchdog timer from triggering
  // and to allow other lower-priority tasks on the same core to run.
  vTaskDelay(2 / portTICK_PERIOD_MS);
}

```

`Multithreaded_Dashboard/test.html`:

```html
<!doctype html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Focus Totem Dashboard</title>

        <!-- The Editorial Voice: Typography Base -->
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
        <link
            href="https://fonts.googleapis.com/css2?family=Epilogue:wght@700&family=Inter:wght@400;500;600&display=swap"
            rel="stylesheet"
        />

        <style>
            :root {
                /* Surface Hierarchy (Dark Mode Default) */
                --bg-void: #0b0e12;
                --surface-low: #1d2024;
                --surface-highest: #2b2d31;
                --tertiary-muted: #d1f3dc;
                --primary-accent: #bbdaff;

                /* Typography Colors */
                --text-primary: #ffffff;
                --text-secondary: #9ca3af;
            }

            * {
                box-sizing: border-box;
                margin: 0;
                padding: 0;
            }

            body {
                font-family: "Inter", sans-serif;
                background-color: var(--bg-void);
                color: var(--text-primary);
                -webkit-font-smoothing: antialiased;
                min-height: 100vh;
                display: flex;
                justify-content: center;
                padding: 64px 24px;
            }

            .monolith-wrapper {
                width: 100%;
                max-width: 800px;
                display: flex;
                flex-direction: column;
                gap: 56px; /* Massive breathing room per Do's and Don'ts */
            }

            /* Typography: Statement Styles */
            .display-lg {
                font-family: "Epilogue", sans-serif;
                font-size: 3.5rem;
                font-weight: 700;
                letter-spacing: -0.02em;
                line-height: 1.1;
                color: var(--text-primary);
            }

            .headline-md {
                font-family: "Epilogue", sans-serif;
                font-size: 1.5rem;
                font-weight: 700;
                letter-spacing: -0.02em;
                color: var(--text-primary);
            }

            /* Typography: High-Density Data Workhorse */
            .body-md {
                font-family: "Inter", sans-serif;
                font-size: 1rem;
                color: var(--text-secondary);
                line-height: 1.5;
                margin-top: 16px;
            }

            .label {
                font-family: "Inter", sans-serif;
                font-size: 1rem;
                font-weight: 500;
                color: var(--text-primary);
            }

            .value {
                font-family: "Inter", sans-serif;
                font-size: 1rem;
                color: var(--text-secondary);
                display: flex;
                align-items: center;
            }

            .badge {
                background-color: var(--surface-highest);
                color: var(--primary-accent);
                padding: 6px 16px;
                border-radius: 9999px;
                font-size: 0.875rem;
                font-weight: 600;
                margin-left: 16px;
                vertical-align: middle;
            }

            /* Tonal Stepping: Level 0 -> Level 1 */
            .dashboard-surface {
                background-color: var(--surface-low);
                border-radius: 24px;
                padding: 40px;
                display: flex;
                flex-direction: column;
                gap: 32px;
            }

            /* The Divider Rule: No 1px lines */
            .list-container {
                display: flex;
                flex-direction: column;
                gap: 1.4rem;
            }

            /* High-Contrast Touch Targets & Cards */
            .list-item {
                background-color: var(
                    --surface-highest
                ); /* Tonal shift milled inside the surface */
                border-radius: 16px;
                min-height: 56px; /* Guaranteed accessible hit area */
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 0 24px;
            }

            /* Buttons & Chips: Primary Actions */
            .btn-primary {
                background-color: var(--primary-accent);
                color: var(--bg-void);
                border: none;
                border-radius: 9999px; /* Pill Shape */
                min-height: 56px; /* Strict 56px rule */
                padding: 0 40px;
                font-family: "Inter", sans-serif;
                font-size: 1.125rem;
                font-weight: 600;
                cursor: pointer;
                transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
                display: inline-flex;
                align-items: center;
                justify-content: center;
            }

            .btn-primary:active {
                transform: scale(0.97);
            }

            /* Status Indicators */
            .status-indicator {
                display: inline-block;
                width: 12px;
                height: 12px;
                border-radius: 50%;
                margin-right: 12px;
            }

            .status-connected {
                background-color: var(--tertiary-muted);
            }

            .status-disconnected {
                background-color: #ff5e5e; /* Stark contrast error state */
            }

            .status-connecting {
                background-color: var(--primary-accent);
                animation: pulse 1.5s ease-in-out infinite;
            }

            @keyframes pulse {
                0%,
                100% {
                    opacity: 1;
                }
                50% {
                    opacity: 0.2;
                }
            }

            .actions-wrapper {
                display: flex;
                justify-content: flex-start;
            }

            /* Form Factor Adaptations */
            @media (max-width: 600px) {
                body {
                    padding: 32px 16px;
                }
                .display-lg {
                    font-size: 2.75rem;
                }
                .dashboard-surface {
                    padding: 24px;
                }
                .list-item {
                    flex-direction: column;
                    align-items: flex-start;
                    justify-content: center;
                    gap: 8px;
                    padding: 16px 24px;
                }
            }
        </style>
    </head>
    <body>
        <div class="monolith-wrapper">
            <!-- The Hierarchy Rule Applied -->
            <header class="header">
                <h1 class="display-lg">Focus Totem</h1>
                <p class="body-md">
                    Real-time status monitoring
                    <span class="badge">Dual-Core</span>
                </p>
            </header>

            <main class="dashboard-surface">
                <h2 class="headline-md">System Status</h2>

                <div class="list-container">
                    <div class="list-item">
                        <span class="label">WiFi Connection</span>
                        <span class="value">
                            <span
                                class="status-indicator"
                                id="wifi-indicator"
                            ></span>
                            <span id="wifi-status">Loading...</span>
                        </span>
                    </div>

                    <div class="list-item">
                        <span class="label">Network Name (SSID)</span>
                        <span class="value" id="ssid">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">IP Address</span>
                        <span class="value" id="ip-address">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">mDNS Hostname</span>
                        <span class="value" id="mdns">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">Signal Strength</span>
                        <span class="value" id="rssi">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">Uptime</span>
                        <span class="value" id="uptime">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">Free Heap Memory</span>
                        <span class="value" id="heap">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">Web Server Core</span>
                        <span class="value" id="web-core">Loading...</span>
                    </div>

                    <div class="list-item">
                        <span class="label">Monitor Core</span>
                        <span class="value" id="monitor-core">Loading...</span>
                    </div>
                </div>
            </main>

            <div class="actions-wrapper">
                <button class="btn-primary" id="refresh-btn">
                    Refresh Status
                </button>
            </div>
        </div>

        <script>
            async function updateStatus() {
                try {
                    const response = await fetch("/api/status");
                    if (!response.ok) {
                        console.error(
                            "Failed to fetch status, server responded with:",
                            response.status,
                        );
                        return;
                    }
                    const data = await response.json();

                    const wifiIndicator =
                        document.getElementById("wifi-indicator");
                    const wifiStatus = document.getElementById("wifi-status");

                    if (data.wifiConnected) {
                        wifiIndicator.className =
                            "status-indicator status-connected";
                        wifiStatus.textContent = "Connected";
                    } else if (data.wifiConnecting) {
                        wifiIndicator.className =
                            "status-indicator status-connecting";
                        wifiStatus.textContent = "Connecting...";
                    } else {
                        wifiIndicator.className =
                            "status-indicator status-disconnected";
                        wifiStatus.textContent = "Disconnected";
                    }

                    document.getElementById("ssid").textContent =
                        data.ssid || "N/A";
                    document.getElementById("ip-address").textContent =
                        data.ipAddress || "N/A";
                    document.getElementById("mdns").textContent =
                        data.mdns || "N/A";
                    document.getElementById("rssi").textContent =
                        data.rssi || "N/A";
                    document.getElementById("uptime").textContent =
                        data.uptime || "N/A";
                    document.getElementById("heap").textContent =
                        data.freeHeap || "N/A";
                    document.getElementById("web-core").textContent =
                        data.webCore || "N/A";
                    document.getElementById("monitor-core").textContent =
                        data.monitorCore || "N/A";
                } catch (error) {
                    console.error("Error fetching status:", error);
                }
            }

            document
                .getElementById("refresh-btn")
                .addEventListener("click", updateStatus);

            // Initial load
            updateStatus();

            // Auto-refresh every 5 seconds
            setInterval(updateStatus, 5000);
        </script>
    </body>
</html>

```

`README.md`:

```md
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
    *   **Session Logging:** Records every completed focus session to a local JSON file for future analytics/reporting.

## Key Features Implemented

| ESP32 Firmware | Rust Desktop Client |
| :--- | :--- |
| ✅ Dual-Core Operation (FreeRTOS) | ✅ Automatic mDNS Device Discovery |
| ✅ Material Design 3 Web Dashboard | ✅ Real-time State Tracking |
| ✅ Real-time JSON API for status | ✅ **Dynamic Wallpaper Changing** |
| ✅ Backward-compatible `/status` endpoint | ✅ **Application Launching & Closing** |
| | ✅ Cross-platform app config via `apps.toml` |
| | ✅ Local JSON session logging |

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
  "brave-browser"
]
        ```

*   **Build and Run (Mock Mode for ESP32-free development):**
    1.  Start the Rust mock server:
        ```focus_client_rust/README.md#L1-1
cargo run -p mock_server
        ```
    2.  In another terminal, run the Rust client in debug mode:
        ```focus_client_rust/README.md#L1-1
cargo run
        ```
    3.  In debug mode, the client prints a development banner and polls:
        - `http://localhost:8080/status`
        - Expected response: `FOCUS_ON` or `FOCUS_OFF`
    4.  Toggle focus state without stopping the mock server:
        ```focus_client_rust/README.md#L1-1
curl http://localhost:8080/toggle
        ```
    5.  Stop the mock server (`Ctrl+C`) to simulate device disconnect and verify disconnect behavior.
    6.  On GNOME-based Linux distros (like Zorin), wallpaper switching uses `gsettings` first, then falls back to the `wallpaper` crate for better compatibility.

*   **Build and Run (Real ESP32 mode):**
    1.  Build/run in release mode:
        ```focus_client_rust/README.md#L1-2
cargo build --release
cargo run --release
        ```
    2.  In this mode, the client uses normal mDNS discovery (`focus-totem`) and polls the real `/status` endpoint.

*   **Session Logs:**
    - A completed focus session is recorded when focus mode deactivates.
    - The client prints the session log file path on first write.
    - Linux path: `~/.local/share/focus_totem/sessions.json`
    - Windows path: `%APPDATA%\FocusTotem\sessions.json`
    - The file is a JSON array, ready for analytics/report generation later.

## Project Files

*   `Multithreaded_Dashboard.ino`: **The main, recommended firmware for the ESP32.**
*   `src/main.rs`: The source code for the Rust desktop client (includes debug mock-mode logic and release real-device logic).
*   `src/session.rs`: Focus session model and atomic JSON session logging.
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

---
_This project is now Rust-first: the desktop client and mock ESP32 server both live in this workspace._

```

`ROADMAP.md`:

```md
# Focus Totem — Development Roadmap

> **Constraints in play:** No ESP32 hardware available. Linux (GNOME) primary dev environment. Max Rust — Python goes in the bin.

---

## Current State Snapshot

- ✅ `src/main.rs` — cross-platform client (Windows + Linux GNOME), `apps.toml` config, `DEV_MODE` compile flag, gsettings wallpaper fallback
- ✅ `M3-Redesign.ino` — dual-core FreeRTOS firmware with Material Design 3 dashboard
- ✅ `mock_server/` — Rust Axum mock server with `/status` and `/toggle` endpoints
- ✅ `hello_gpui/` — GPUI UI experiment (parked, not integrated)
- ✅ Session logging — implemented in `src/session.rs` with local JSON storage
- ❌ AI analytics — not implemented
- ❌ Report generation — not implemented
- ✅ Rust mock server — implemented in `mock_server/`

---

## Phase 1 — Kill `mock_esp32.py`: Rust Mock Server Binary

**Goal:** Purge the Python dependency entirely. Replace with a proper Rust binary in the same workspace.

**Status:** ✅ Implemented

### Steps

- Convert the project root into a **Cargo workspace** by updating `Cargo.toml`:
  ```toml
  [workspace]
  members = [".", "mock_server"]
  ```
- Create `mock_server/` as a new crate: `cargo new mock_server --bin`
- Add dependencies to `mock_server/Cargo.toml`:
  ```toml
  axum = "0.8"
  tokio = { version = "1", features = ["full"] }
  ```
- Implement the server with **two endpoints**:
  - `GET /status` → returns `"FOCUS_ON"` or `"FOCUS_OFF"` based on shared atomic state
  - `GET /toggle` → flips the state and returns the new value (so you can simulate disconnect without killing the process)
- The state should be an `Arc<AtomicBool>` shared between the toggle handler and the status handler
- **No CLI args needed** — hardcode `127.0.0.1:8080` matching the existing `DEV_MODE` constant
- Run with: `cargo run -p mock_server` in one terminal, `cargo run` in another

### Why axum
- Tokio-native, zero-cost, production-grade — not overkill since you'll reuse tokio when refactoring the main client later
- A single route takes ~10 lines. Compile time is the only cost

---

## Phase 2 — Session Logging Foundation

**Goal:** The client silently records every focus session to a local JSON file. This is the data source for all AI work.

**Status:** ✅ Implemented

### Steps

- Add to `Cargo.toml`:
  ```toml
  chrono = { version = "0.4", features = ["serde"] }
  serde_json = "1"
  dirs = "5"           # XDG / platform-aware data dir resolution
  ```
- Define the session record struct in a new `src/session.rs` module:
  ```rust
  #[derive(Serialize, Deserialize)]
  pub struct SessionRecord {
      pub start_time: DateTime<Local>,
      pub end_time: DateTime<Local>,
      pub duration_minutes: f32,
      pub hour_of_day: u8,          // 0–23, from start_time
      pub day_of_week: u8,          // 0=Mon … 6=Sun
      pub interrupted: bool,         // duration < 10 min
  }
  ```
- Resolve the sessions file path with `dirs::data_dir()`:
  - Linux: `~/.local/share/focus_totem/sessions.json`
  - Windows: `%APPDATA%\FocusTotem\sessions.json`
  - Fallback: `./sessions.json` next to the binary
- Store sessions as a **JSON array** — append on each session end (read → push → write)
- Track session start: set a `Option<DateTime<Local>>` when `activate_focus_mode` is called
- Write the record when `deactivate_focus_mode` is called, computing duration from start
- Log the file path to stdout on first write so it's easy to find

### Data integrity note
- Read the full file, deserialize, push the new record, serialize back, write atomically (write to `.tmp` then `rename`)
- `rename` is atomic on Linux. Use `std::fs::rename`

---

## Phase 3 — AI Analytics: Pure Rust with `linfa`

**Goal:** K-means clustering on session data + peak-hour detection. Zero Python.

### Add to `Cargo.toml`
```toml
linfa = "0.7"
linfa-clustering = "0.7"
ndarray = "0.16"
```

### Create `src/analytics.rs`

#### 3.1 Load & filter sessions
- Read `sessions.json`, deserialize into `Vec<SessionRecord>`
- Require at least **10 sessions** before running clustering (return early with a message otherwise — cold start problem)

#### 3.2 K-means clustering (k=3)
- Feature matrix: `[duration_minutes, hour_of_day as f32]` — shape `(n_sessions, 2)`
- Normalize both features to `[0, 1]` before fitting (duration range ≈ 0–120 min, hour range 0–23)
- Use `linfa_clustering::KMeans::params(3).fit(&dataset)` — Lloyd's algorithm, pure Rust
- Label clusters by mean duration: highest = **"Peak"**, middle = **"Moderate"**, lowest = **"Distracted"**

```rust
use linfa::prelude::*;
use linfa_clustering::KMeans;
use ndarray::Array2;

let data: Array2<f32> = /* build from sessions */;
let dataset = linfa::Dataset::from(data);
let model = KMeans::params(3)
    .max_n_iterations(200)
    .tolerance(1e-5)
    .fit(&dataset)
    .expect("KMeans failed");
let assignments = model.predict(&dataset);
```

#### 3.3 Peak-hour detection
- Group sessions by `hour_of_day` (0–23 buckets)
- For each hour, compute: `score = mean_duration * session_count` (rewards both quality and consistency)
- Sort descending → top 3 hours = recommended focus windows
- Format as "10:00 – 11:00" etc.

#### 3.4 Moving average on daily totals (7-day window)
- Group sessions by calendar date, sum durations per day
- Apply simple 7-day centered moving average: `avg[i] = sum(days[i-3..=i+3]) / count`
- Detect trend: if last 3 days average > overall average → "📈 Focus trending up", else "📉 Consider protecting your focus blocks"

#### 3.5 Distraction rate
- `distraction_rate = interrupted_sessions / total_sessions * 100`
- Threshold: > 30% distracted → trigger a specific recommendation in the report

---

## Phase 4 — Focus Health Report Generator

**Goal:** After each session (or via a `--report` CLI flag), generate a readable HTML report and open it in the browser.

### Steps

- Add to `Cargo.toml`:
  ```toml
  # Nothing new needed — use std::fs + format! strings for HTML
  ```
- Write `src/report.rs` with a `generate_report(stats: &AnalyticsResult) -> String` function
- Inline HTML/CSS using the existing **Monolithic Precision design system** from `M3Design.md`:
  - Background: `#0b0e12`, surface: `#1D2024`, accent: `#bbdaff`
  - Font: Inter (load from Google Fonts CDN inline in `<head>`)
  - Use the same card/list-item pattern from your firmware dashboard
- Write the HTML to a temp file: `std::env::temp_dir().join("focus_report.html")`
- Open it with the platform default browser:
  ```rust
  #[cfg(target_os = "linux")]
  Command::new("xdg-open").arg(&report_path).spawn();

  #[cfg(target_os = "windows")]
  Command::new("explorer").arg(&report_path).spawn();
  ```
- Report sections to include:
  1. **Weekly Summary** — total deep work hours, total sessions, this week vs last week delta
  2. **Top Focus Windows** — top 3 recommended hours with their score
  3. **Cluster Breakdown** — count/avg duration for Peak, Moderate, Distracted clusters
  4. **Trend Line** — 7-day moving average as a simple ASCII bar chart (or HTML `<div>` width trick)
  5. **AI Recommendation** — one plain-English paragraph generated from the stats

### CLI flag
- Parse `std::env::args()` manually (no need for `clap` yet):
  ```rust
  if args.contains(&"--report".to_string()) {
      run_report_only();
      return;
  }
  ```

---

## Phase 5 — Modern Rust Housekeeping

**Goal:** Remove legacy patterns. Improve code structure before it gets harder.

### 5.1 Replace `lazy_static` with `std::sync::LazyLock`
- `LazyLock` is stable since **Rust 1.80** — no external dependency needed
- Before: `lazy_static! { static ref ORIGINAL_WALLPAPER_PATH: Mutex<Option<String>> = Mutex::new(None); }`
- After:
  ```rust
  static ORIGINAL_WALLPAPER_PATH: LazyLock<Mutex<Option<String>>> =
      LazyLock::new(|| Mutex::new(None));
  ```
- Remove `lazy_static` from `Cargo.toml` entirely

### 5.2 Module structure
Split `src/main.rs` (currently ~500 lines and growing) into:
```
src/
├── main.rs          # entry point, main loop only
├── config.rs        # load_focus_apps(), AppsConfig, constants
├── automation.rs    # launch_app(), terminate_app(), wallpaper functions
├── discovery.rs     # discover_device(), mDNS logic
├── session.rs       # SessionRecord, logging, file I/O
├── analytics.rs     # k-means, peak-hour detection, moving average
└── report.rs        # HTML report generation, browser open
```

### 5.3 Error handling
- Replace `unwrap()` calls in non-fatal paths with `?` propagation or `if let Err(e)`
- `main()` should return `anyhow::Result<()>` — add `anyhow = "1"` to deps
- Gives you proper error context on any crash: `"Failed to write sessions.json: Permission denied (os error 13)"`

### 5.4 Remove `hello_gpui/` or graduate it
- It's currently a dead-end experiment with stale GPUI API calls
- Decision: either delete it and track the GUI idea in the roadmap, or wire it to the current `src/` logic
- Recommendation: **delete for now**, open a GitHub issue titled "GUI: GPUI integration" to track it properly

---

## Phase 6 — Async Refactor (Main Client)

**Goal:** Replace the `blocking reqwest + thread::sleep` loop with proper async. Cleaner cancellation, lower resource usage, future-proofs the GUI path.

### Steps

- Add `tokio = { version = "1", features = ["full"] }` to main `Cargo.toml`
- Switch `reqwest` to async: `reqwest = { version = "0.12", features = ["json"] }` (drop `blocking` feature)
- Annotate `main` with `#[tokio::main]`
- Replace `thread::sleep(Duration::from_secs(3))` with `tokio::time::sleep(...).await`
- The mDNS discovery (`mdns-sd`) uses its own thread internally — wrap `discover_device` in `tokio::task::spawn_blocking`
- All automation functions (`activate_focus_mode`, `deactivate_focus_mode`) stay sync — call them from `spawn_blocking` since they shell out to OS commands

### Why now and not earlier
- The mock server (Phase 1) already uses tokio, so the dependency is already in the workspace
- Sessions + analytics (Phases 2–4) are sync file I/O — they don't need async
- Doing this before adding a GUI is the right order

---

## Phase 7 — System Integration: Autostart & Background

**Goal:** The client should start automatically with the desktop session and run silently in the background.

### Linux (systemd user service)
- Create `focus-totem.service`:
  ```ini
  [Unit]
  Description=Focus Totem Client
  After=graphical-session.target

  [Service]
  Type=simple
  ExecStart=/path/to/focus_client_rust
  Restart=on-failure
  RestartSec=5s
  Environment=DISPLAY=:0

  [Install]
  WantedBy=graphical-session.target
  ```
- Install: `cp focus-totem.service ~/.config/systemd/user/ && systemctl --user enable --now focus-totem`
- Generate a `--install-service` CLI flag that writes and enables this automatically

### Windows (startup via registry)
- On `--install-service`, write the exe path to:
  `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
- Use the `winreg` crate: `winreg = "0.52"`
- Conditionally compile: `#[cfg(target_os = "windows")]`

### Tray icon (optional, post-autostart)
- `tray-icon = "0.19"` crate for a minimal system tray presence on both platforms
- States: searching (grey), connected (blue), focus active (green)
- Right-click menu: "Force Deactivate Focus" / "View Report" / "Quit"

---

## Phase 8 — ESP32 Enhancements (for when hardware is back)

**Goal:** Harden the firmware and add interactivity to the dashboard.

- **Manual toggle from dashboard:** Add a `POST /toggle` endpoint on the ESP32 that flips an `isFocusOverride` bool. The `/status` endpoint returns `FOCUS_OFF` when override is set. Lets you test deactivation without physically unplugging the device.
- **LED status indicator:** GPIO output on a pin → green LED for FOCUS_ON, off for idle. Wire from the `handleStatus` function.
- **Session count on dashboard:** Increment a `uint32_t sessionCount` in SRAM each time `/status` is polled and the focus state is active. Show it on the dashboard.
- **Button debounce interrupt:** Wire a physical push button → ISR on GPIO → toggle focus state directly from hardware, no HTTP needed. Use `attachInterrupt()` + `portENTER_CRITICAL` for thread safety with FreeRTOS.
- **OTA firmware updates:** `ArduinoOTA` library. Add to `setup()`, call `ArduinoOTA.handle()` in `loop()`. Lets you push new firmware over Wi-Fi from Arduino IDE without USB.

---

## Dependency Summary (end state)

```toml
[dependencies]
# Existing
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
toml        = "0.8"
reqwest     = { version = "0.12", features = ["json"] }   # async after Phase 6
wallpaper   = "3.2"
mdns-sd     = "0.15"

# New
tokio       = { version = "1", features = ["full"] }      # Phase 1 / Phase 6
axum        = "0.8"                                        # mock_server crate only
chrono      = { version = "0.4", features = ["serde"] }   # Phase 2
dirs        = "5"                                          # Phase 2
linfa       = "0.7"                                        # Phase 3
linfa-clustering = "0.7"                                   # Phase 3
ndarray     = "0.16"                                       # Phase 3
anyhow      = "1"                                          # Phase 5

# Windows-only
[target.'cfg(windows)'.dependencies]
winreg = "0.52"                                            # Phase 7

# Removed
# lazy_static — replaced by std::sync::LazyLock (Rust 1.80+)
```

---

## Execution Order (Recommended)

| Priority | Phase | Why first |
|---|---|---|
| 🔴 1 | **Phase 1** — Rust mock server | Unblocks ESP-free development immediately, kills Python dep |
| 🔴 2 | **Phase 2** — Session logging | Everything downstream (AI, reports) depends on this data |
| 🟠 3 | **Phase 5.1–5.3** — Housekeeping | Do this before the codebase gets bigger, or you'll regret it |
| 🟠 4 | **Phase 3** — Analytics | Core AI module for the CEP proposal |
| 🟠 5 | **Phase 4** — Report generator | Closes the AI loop, gives you something to demo |
| 🟡 6 | **Phase 6** — Async refactor | Quality of life, needed before any GUI work |
| 🟡 7 | **Phase 7** — Autostart | Usability polish |
| 🟢 8 | **Phase 8** — ESP32 hardening | Hardware-dependent, do when device is available |

```

`apps.toml`:

```toml
# App commands/paths to launch when Focus Mode turns ON.
# This file uses a structured OS-to-app map.
# Supported OS keys include: "windows", "linux", "macos"

[apps]
windows = [
  "C:\\Windows\\System32\\notepad.exe",
  "C:\\Users\\Faizy\\AppData\\Local\\BraveSoftware\\Brave-Browser\\Application\\brave.exe"
]

linux = [
  "brave-browser"
]

# Optional for future use:
# macos = ["TextEdit", "Google Chrome"]

```

`build.rs`:

```rs
extern crate embed_manifest;

fn main() {
    // Only embed a Windows manifest when targeting Windows.
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        embed_manifest::embed_manifest(embed_manifest::new_manifest("FocusClient"))
            .expect("unable to embed manifest");
        println!("cargo:rerun-if-changed=manifest.xml");
    }
}

```

`hello_gpui/Cargo.toml`:

```toml
[package]
name = "hello_gpui"
version = "0.1.0"
edition = "2024"
# In your Cargo.toml

[dependencies]
gpui = "0.2.2"
# THIS IS THE FIX: Add the "blocking" feature
reqwest = { version = "0.12", features = ["json", "blocking"] }
lazy_static = "1.5.0"
mdns-sd = "0.17.1"
smol = "2.0.2" # Add this line
wallpaper = "3.2.0"

[build-dependencies]
embed-manifest = "1.4"
```

`hello_gpui/src/main.rs`:

```rs
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
```

`hello_gpui/steps.txt`:

```txt
Install vulkan_lunar
compile & run
```

`manifest.xml`:

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
```

`mock_server/Cargo.toml`:

```toml
[package]
name = "mock_server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }

```

`mock_server/src/main.rs`:

```rs
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

```

`src/main.rs`:

```rs
mod session;

use chrono::{DateTime, Local};
use lazy_static::lazy_static;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
#[cfg(target_os = "windows")]
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use session::{append_session, SessionRecord};

#[cfg(debug_assertions)]
const DEV_MODE: bool = true; // Enable for local development
#[cfg(not(debug_assertions))]
const DEV_MODE: bool = false; // Disable for release/production

// --- Configuration ---
const SERVICE_NAME: &str = "_http._tcp.local.";
const DEVICE_HOSTNAME: &str = "focus-totem";
const FOCUS_WALLPAPER_NAME: &str = "focus_wallpaper.jpg";
const APPS_CONFIG_FILE: &str = "apps.toml";

lazy_static! {
    static ref ORIGINAL_WALLPAPER_PATH: Mutex<Option<String>> = Mutex::new(None);
}

#[derive(Debug, Deserialize)]
struct AppsConfig {
    apps: std::collections::HashMap<String, Vec<String>>,
}

fn default_focus_apps() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        vec![
            r"C:\Windows\System32\notepad.exe".to_string(),
            r"C:\Users\Faizy\AppData\Local\BraveSoftware\Brave-Browser\Application\brave.exe"
                .to_string(),
        ]
    }

    #[cfg(target_os = "linux")]
    {
        vec!["brave-browser".to_string()]
    }

    #[cfg(target_os = "macos")]
    {
        vec!["TextEdit".to_string(), "Google Chrome".to_string()]
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        vec![]
    }
}

fn parse_apps_config(contents: &str) -> Result<AppsConfig, toml::de::Error> {
    toml::from_str(contents)
}

fn select_apps_for_current_os(config: &AppsConfig) -> Option<Vec<String>> {
    config.apps.get(std::env::consts::OS).cloned()
}

fn sanitize_apps(apps: Vec<String>) -> Vec<String> {
    apps.into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn load_focus_apps() -> Vec<String> {
    match fs::read_to_string(APPS_CONFIG_FILE) {
        Ok(contents) => match parse_apps_config(&contents) {
            Ok(config) => {
                let selected = select_apps_for_current_os(&config).unwrap_or_default();
                let cleaned = sanitize_apps(selected);
                if cleaned.is_empty() {
                    let defaults = default_focus_apps();
                    println!(
                        "No apps configured for OS '{}' in '{}'. Using {} default app(s).",
                        std::env::consts::OS,
                        APPS_CONFIG_FILE,
                        defaults.len()
                    );
                    defaults
                } else {
                    println!(
                        "Loaded {} app(s) for OS '{}' from '{}'.",
                        cleaned.len(),
                        std::env::consts::OS,
                        APPS_CONFIG_FILE
                    );
                    cleaned
                }
            }
            Err(e) => {
                eprintln!(
                    "ERROR: Could not parse '{}': {}. Falling back to defaults.",
                    APPS_CONFIG_FILE, e
                );
                default_focus_apps()
            }
        },
        Err(e) => {
            eprintln!(
                "ERROR: Could not read '{}': {}. Falling back to defaults.",
                APPS_CONFIG_FILE, e
            );
            default_focus_apps()
        }
    }
}

fn launch_app(app: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        return Command::new("open").args(["-a", app]).spawn().map(|_| ());
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        return Command::new(app).spawn().map(|_| ());
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unsupported OS: {}", std::env::consts::OS),
        ))
    }
}

fn terminate_app(app: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let image_name = Path::new(app)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(app);

        let output = Command::new("taskkill")
            .args(["/F", "/IM", image_name])
            .output()?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("taskkill failed for '{}': {}", image_name, stderr.trim()),
        ));
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let output = Command::new("pkill").args(["-f", app]).output()?;

        // pkill exit code: 0 => matched/killed, 1 => no matching process (safe for us)
        if output.status.success() || output.status.code() == Some(1) {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("pkill failed for '{}': {}", app, stderr.trim()),
        ));
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unsupported OS: {}", std::env::consts::OS),
        ))
    }
}

#[cfg(target_os = "linux")]
fn gsettings_get_wallpaper_uri() -> Option<String> {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.background", "picture-uri"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let trimmed = raw.trim_matches('\'').trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

#[cfg(target_os = "linux")]
fn gsettings_set_wallpaper_uri(uri: &str) -> bool {
    let status = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri", uri])
        .status();

    match status {
        Ok(s) if s.success() => {
            // Try dark-variant key as well (GNOME 42+ / some distros), ignore failure.
            let _ = Command::new("gsettings")
                .args([
                    "set",
                    "org.gnome.desktop.background",
                    "picture-uri-dark",
                    uri,
                ])
                .status();
            true
        }
        _ => false,
    }
}

#[cfg(target_os = "linux")]
fn path_to_file_uri(path: &str) -> String {
    let normalized = path.replace(' ', "%20");
    if normalized.starts_with("file://") {
        normalized
    } else {
        format!("file://{}", normalized)
    }
}

fn save_original_wallpaper_path() {
    if let Ok(path) = wallpaper::get() {
        println!("Saved original wallpaper: {}", path);
        *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(path);
        return;
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(uri) = gsettings_get_wallpaper_uri() {
            println!("Saved original wallpaper from gsettings: {}", uri);
            *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(uri);
        }
    }
}

fn set_focus_wallpaper() {
    match fs::canonicalize(FOCUS_WALLPAPER_NAME) {
        Ok(absolute_path) => {
            let path_str = match absolute_path.to_str() {
                Some(p) => p,
                None => {
                    eprintln!("ERROR: Focus wallpaper path contains invalid UTF-8.");
                    return;
                }
            };

            #[cfg(target_os = "linux")]
            {
                let uri = path_to_file_uri(path_str);
                if gsettings_set_wallpaper_uri(&uri) {
                    println!("Focus wallpaper has been set (gsettings primary).");
                    return;
                }

                if wallpaper::set_from_path(path_str).is_ok() {
                    println!("Focus wallpaper has been set (wallpaper crate fallback).");
                    return;
                }

                eprintln!(
                    "ERROR: Failed to set focus wallpaper using gsettings and wallpaper crate."
                );
                return;
            }

            #[cfg(not(target_os = "linux"))]
            {
                if wallpaper::set_from_path(path_str).is_ok() {
                    println!("Focus wallpaper has been set (wallpaper crate).");
                    return;
                }

                eprintln!("ERROR: Failed to set focus wallpaper using wallpaper crate.");
            }
        }
        Err(e) => eprintln!(
            "ERROR: Could not find wallpaper '{}': {}",
            FOCUS_WALLPAPER_NAME, e
        ),
    }
}

fn restore_original_wallpaper() {
    let mut original_path = ORIGINAL_WALLPAPER_PATH.lock().unwrap();
    if let Some(path) = original_path.as_deref() {
        #[cfg(target_os = "linux")]
        {
            let uri = if path.starts_with("file://") {
                path.to_string()
            } else {
                path_to_file_uri(path)
            };

            if gsettings_set_wallpaper_uri(&uri) {
                println!("Restored original wallpaper (gsettings primary): {}", uri);
                *original_path = None;
                return;
            }

            if wallpaper::set_from_path(path).is_ok() {
                println!(
                    "Restored original wallpaper (wallpaper crate fallback): {}",
                    path
                );
                *original_path = None;
                return;
            }

            eprintln!(
                "ERROR: Failed to restore original wallpaper using gsettings and wallpaper crate."
            );
            *original_path = None;
            return;
        }

        #[cfg(not(target_os = "linux"))]
        {
            if wallpaper::set_from_path(path).is_ok() {
                println!("Restored original wallpaper (wallpaper crate): {}", path);
                *original_path = None;
                return;
            }

            eprintln!("ERROR: Failed to restore original wallpaper using wallpaper crate.");
        }
    }

    *original_path = None;
}

fn activate_focus_mode(focus_apps: &[String]) {
    println!("Activating focus mode automations...");

    // Save current wallpaper so we can restore it later
    save_original_wallpaper_path();

    // Set focus wallpaper (with Linux GNOME fallback)
    set_focus_wallpaper();

    // Launch apps
    println!(
        "Launching focus applications for {}...",
        std::env::consts::OS
    );
    for app in focus_apps {
        match launch_app(app) {
            Ok(_) => println!("Successfully launched '{}'", app),
            Err(e) => eprintln!("ERROR: Failed to launch '{}': {}", app, e),
        }
    }
}

fn deactivate_focus_mode(focus_apps: &[String]) {
    println!("Deactivating focus mode automations...");

    // Restore wallpaper (with Linux GNOME fallback)
    restore_original_wallpaper();

    // Close apps
    println!("Closing focus applications for {}...", std::env::consts::OS);
    for app in focus_apps {
        if let Err(e) = terminate_app(app) {
            eprintln!("ERROR: Failed to close '{}': {}", app, e);
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

fn discover_device(search_duration: Duration) -> Option<String> {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let receiver = mdns
        .browse(SERVICE_NAME)
        .expect("Failed to browse for service");
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < search_duration {
        if let Ok(event) = receiver.recv_timeout(Duration::from_secs(1)) {
            if let ServiceEvent::ServiceResolved(info) = event {
                if info.get_fullname().contains(DEVICE_HOSTNAME) {
                    let ip = info.get_addresses().iter().next()?;
                    let port = info.get_port();
                    let url = format!("http://{}:{}/status", ip, port);
                    println!("Resolved Focus Totem address: {}", url);
                    return Some(url);
                }
            }
        }
    }

    None
}

fn main() {
    let http_client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("Failed to build HTTP client");

    let focus_apps = load_focus_apps();

    let mut is_focused = false;
    let mut esp32_address: Option<String> = None;
    let mut session_start: Option<DateTime<Local>> = None;

    println!("Starting Focus Mode client (Rust version)...");
    println!("Detected OS: {}", std::env::consts::OS);
    println!("Configured apps: {:?}", focus_apps);

    if DEV_MODE {
        println!("==========================================");
        println!("=== DEVELOPMENT MODE ACTIVE (MOCK ESP32) ===");
        println!("Mock ESP32 endpoint: http://localhost:8080/status");
        println!("To exit mock mode, build release or set DEV_MODE to false.");
        println!("==========================================");
    }

    loop {
        if DEV_MODE {
            if esp32_address.is_none() {
                esp32_address = Some("http://localhost:8080/status".to_string());
                if let Some(address) = &esp32_address {
                    println!("[MOCK] Simulated ESP32 address: {}", address);
                }
            }

            if let Some(address) = &esp32_address {
                match http_client.get(address).send() {
                    Ok(response) => {
                        if response.status().is_success()
                            && response.text().unwrap_or_default() == "FOCUS_ON"
                        {
                            if !is_focused {
                                is_focused = true;
                                println!("[MOCK] --- FOCUS MODE ACTIVATED ---");
                                begin_focus_session(&mut session_start);
                                activate_focus_mode(&focus_apps);
                            }
                        } else if is_focused {
                            is_focused = false;
                            println!("[MOCK] --- FOCUS MODE DEACTIVATED (non-FOCUS_ON) ---");
                            deactivate_focus_mode(&focus_apps);
                            end_focus_session(&mut session_start);
                        }
                    }
                    Err(e) => {
                        eprintln!("[MOCK] Error polling mock ESP32: {:?}", e);
                        if is_focused {
                            is_focused = false;
                            println!("[MOCK] --- FOCUS MODE DEACTIVATED ---");
                            deactivate_focus_mode(&focus_apps);
                            end_focus_session(&mut session_start);
                        }
                        esp32_address = None;
                    }
                }
            }

            thread::sleep(Duration::from_secs(3));
            continue;
        }

        if esp32_address.is_none() {
            println!("Searching for Focus Totem on the network...");
            if let Some(found_address) = discover_device(Duration::from_secs(5)) {
                esp32_address = Some(found_address);
            } else {
                println!("Device not found. Will retry in 4 seconds.");
                thread::sleep(Duration::from_secs(4));
            }
        }

        if let Some(address) = &esp32_address {
            match http_client.get(address).send() {
                Ok(response) => {
                    if response.status().is_success()
                        && response.text().unwrap_or_default() == "FOCUS_ON"
                    {
                        if !is_focused {
                            is_focused = true;
                            println!("--- FOCUS MODE ACTIVATED ---");
                            begin_focus_session(&mut session_start);
                            activate_focus_mode(&focus_apps);
                        }
                    }
                }
                Err(_) => {
                    if is_focused {
                        is_focused = false;
                        println!("--- FOCUS MODE DEACTIVATED ---");
                        deactivate_focus_mode(&focus_apps);
                        end_focus_session(&mut session_start);
                    }
                    println!("Lost connection to device. Returning to search mode.");
                    esp32_address = None;
                }
            }
        }

        thread::sleep(Duration::from_secs(3));
    }
}

```

`src/session.rs`:

```rs
use chrono::{DateTime, Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

static FIRST_WRITE_PATH_LOGGED: OnceLock<()> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub duration_minutes: f32,
    pub hour_of_day: u8,
    pub day_of_week: u8,
    pub interrupted: bool,
}

impl SessionRecord {
    pub fn new(start_time: DateTime<Local>, end_time: DateTime<Local>) -> Self {
        let duration = end_time.signed_duration_since(start_time);
        let duration_minutes = (duration.num_seconds().max(0) as f32) / 60.0;
        let hour_of_day = start_time.hour() as u8;
        let day_of_week = start_time.weekday().num_days_from_monday() as u8;
        let interrupted = duration_minutes < 10.0;

        Self {
            start_time,
            end_time,
            duration_minutes,
            hour_of_day,
            day_of_week,
            interrupted,
        }
    }
}

pub fn sessions_file_path() -> PathBuf {
    if let Some(mut data_dir) = dirs::data_dir() {
        #[cfg(target_os = "windows")]
        data_dir.push("FocusTotem");

        #[cfg(not(target_os = "windows"))]
        data_dir.push("focus_totem");

        data_dir.push("sessions.json");
        return data_dir;
    }

    PathBuf::from("sessions.json")
}

pub fn append_session(record: SessionRecord) -> io::Result<PathBuf> {
    let path = sessions_file_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut sessions = read_sessions(&path)?;
    sessions.push(record);
    write_sessions_atomically(&path, &sessions)?;

    FIRST_WRITE_PATH_LOGGED.get_or_init(|| {
        println!("Session log file: {}", path.display());
    });

    Ok(path)
}

fn read_sessions(path: &Path) -> io::Result<Vec<SessionRecord>> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            if contents.trim().is_empty() {
                return Ok(Vec::new());
            }

            serde_json::from_str(&contents).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse existing sessions JSON: {e}"),
                )
            })
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

fn write_sessions_atomically(path: &Path, sessions: &[SessionRecord]) -> io::Result<()> {
    let tmp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(sessions).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize sessions JSON: {e}"),
        )
    })?;

    fs::write(&tmp_path, json)?;

    // On Unix-like systems, renaming over an existing file is atomic.
    // On Windows, std::fs::rename fails if the destination exists, so remove first.
    #[cfg(target_os = "windows")]
    {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }

    fs::rename(tmp_path, path)?;

    Ok(())
}

```

`totem.cpp`:

```cpp
#include <WiFi.h>
#include <ESPmDNS.h>
#include <WebServer.h>

// --- IMPORTANT: CHANGE THESE TO YOUR WI-FI CREDENTIALS ---
const char* ssid = "YOUR_WIFI_NAME";
const char* password = "YOUR_WIFI_PASSWORD";

// Create a WebServer object that will listen on port 80
WebServer server(80);

void handleStatus() {
  // This function is called when a client requests the /status URL
  Serial.println("Client requested status. Sending FOCUS_ON...");
  server.send(200, "text/plain", "FOCUS_ON"); // Send the response
}

void setup() {
  // Start the serial monitor for debugging
  Serial.begin(115200);
  Serial.println(); // Print a blank line

  // --- 1. Connect to Wi-Fi ---
  Serial.print("Connecting to ");
  Serial.println(ssid);
  WiFi.begin(ssid, password);

  // Wait for the connection to complete
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }

  Serial.println("");
  Serial.println("WiFi connected!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());

  // --- 2. Start mDNS ---
  // This announces the ESP32 on the network as 'focus-totem.local'
  if (!MDNS.begin("focus-totem")) {
    Serial.println("Error setting up mDNS responder!");
    while(1) { delay(1000); } // Halt if mDNS fails
  }
  Serial.println("mDNS responder started");

  // Announce that we are an HTTP (web) server
  MDNS.addService("http", "tcp", 80);
  Serial.println("Announced http service on port 80");
  
  // --- 3. Configure and Start the Web Server ---
  // Tell the server which function to call when it gets a request to "/status"
  server.on("/status", HTTP_GET, handleStatus);

  // Start the server
  server.begin();
  Serial.println("Web server started");
}

void loop() {
  // This is required for the server to process incoming client requests
  server.handleClient();
}
```
```

`apps.toml`:

```toml
# App commands/paths to launch when Focus Mode turns ON.
# This file uses a structured OS-to-app map.
# Supported OS keys include: "windows", "linux", "macos"

[apps]
windows = [
  "C:\\Windows\\System32\\notepad.exe",
  "C:\\Users\\Faizy\\AppData\\Local\\BraveSoftware\\Brave-Browser\\Application\\brave.exe"
]

linux = [
  "brave-browser"
]

# Optional for future use:
# macos = ["TextEdit", "Google Chrome"]

```

`build.rs`:

```rs
extern crate embed_manifest;

fn main() {
    // Only embed a Windows manifest when targeting Windows.
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        embed_manifest::embed_manifest(embed_manifest::new_manifest("FocusClient"))
            .expect("unable to embed manifest");
        println!("cargo:rerun-if-changed=manifest.xml");
    }
}

```

`hello_gpui/Cargo.toml`:

```toml
[package]
name = "hello_gpui"
version = "0.1.0"
edition = "2024"
# In your Cargo.toml

[dependencies]
gpui = "0.2.2"
# THIS IS THE FIX: Add the "blocking" feature
reqwest = { version = "0.12", features = ["json", "blocking"] }
lazy_static = "1.5.0"
mdns-sd = "0.17.1"
smol = "2.0.2" # Add this line
wallpaper = "3.2.0"

[build-dependencies]
embed-manifest = "1.4"
```

`hello_gpui/src/main.rs`:

```rs
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
```

`hello_gpui/steps.txt`:

```txt
Install vulkan_lunar
compile & run
```

`manifest.xml`:

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
```

`mock_server/Cargo.toml`:

```toml
[package]
name = "mock_server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }

```

`mock_server/src/main.rs`:

```rs
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

```

`src/analytics.rs`:

```rs
use crate::session::SessionRecord;
use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate};
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use linfa_trees::{DecisionTree, SplitQuality};
use ndarray::{Array1, Array2};
use std::collections::BTreeMap;

const TREND_MIN_DAYS: usize = 7;
const TREE_MIN_SESSIONS: usize = 30;

#[derive(Debug)]
pub struct AnalyticsResult {
    // Layer 1 — always present
    pub total_sessions: usize,
    pub distraction_rate: f32,
    pub top_focus_hours: Vec<(u8, f32)>,
    pub best_days: Vec<(u8, f32)>,
    pub weekly_total_minutes: f32,
    pub weekly_delta_minutes: f32,

    // Layer 2 — None if < 7 calendar days of data
    pub trend_slope: Option<f32>,
    pub trend_label: Option<String>,

    // Layer 3 — None if < 30 sessions
    pub tree_rules: Option<String>,
    pub quality_rate: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionQuality {
    Quality = 0,
    Shallow = 1,
    Distracted = 2,
}

impl SessionQuality {
    fn as_label(self) -> usize {
        self as usize
    }

    fn from_label(label: usize) -> Self {
        match label {
            0 => Self::Quality,
            1 => Self::Shallow,
            _ => Self::Distracted,
        }
    }
}

fn label_session(session: &SessionRecord) -> SessionQuality {
    if session.duration_minutes >= 20.0 && !session.interrupted {
        SessionQuality::Quality
    } else if session.duration_minutes >= 10.0 {
        SessionQuality::Shallow
    } else {
        SessionQuality::Distracted
    }
}

pub fn run_analytics(sessions: &[SessionRecord]) -> AnalyticsResult {
    let total_sessions = sessions.len();

    if sessions.is_empty() {
        return AnalyticsResult {
            total_sessions,
            distraction_rate: 0.0,
            top_focus_hours: Vec::new(),
            best_days: Vec::new(),
            weekly_total_minutes: 0.0,
            weekly_delta_minutes: 0.0,
            trend_slope: None,
            trend_label: None,
            tree_rules: None,
            quality_rate: None,
        };
    }

    let distraction_rate = distraction_rate(sessions);
    let top_focus_hours = top_focus_hours(sessions);
    let best_days = best_days(sessions);
    let (weekly_total_minutes, weekly_delta_minutes) = weekly_summary(sessions);
    let daily_totals = daily_totals(sessions);
    let (trend_slope, trend_label) = trend_detection(&daily_totals)
        .map(|(slope, label)| (Some(slope), Some(label)))
        .unwrap_or((None, None));
    let (tree_rules, quality_rate) = decision_tree_summary(sessions)
        .map(|(rules, rate)| (Some(rules), Some(rate)))
        .unwrap_or((None, None));

    AnalyticsResult {
        total_sessions,
        distraction_rate,
        top_focus_hours,
        best_days,
        weekly_total_minutes,
        weekly_delta_minutes,
        trend_slope,
        trend_label,
        tree_rules,
        quality_rate,
    }
}

pub fn format_analytics(result: &AnalyticsResult) -> String {
    let mut lines = Vec::new();

    lines.push("Focus Analytics".to_string());
    lines.push("===============".to_string());
    lines.push(format!("Total sessions: {}", result.total_sessions));
    lines.push(format!(
        "Distraction rate: {:.1}%{}",
        result.distraction_rate,
        if result.distraction_rate > 30.0 {
            " — high; protect your focus blocks"
        } else {
            ""
        }
    ));
    lines.push(format!(
        "This week: {} ({})",
        format_minutes(result.weekly_total_minutes),
        format_delta_minutes(result.weekly_delta_minutes)
    ));

    lines.push(String::new());
    lines.push("Top focus hours:".to_string());
    if result.top_focus_hours.is_empty() {
        lines.push("  Not enough completed sessions yet.".to_string());
    } else {
        for (hour, score) in &result.top_focus_hours {
            lines.push(format!(
                "  {}:00–{}:00 (score: {:.1})",
                hour,
                (*hour + 1) % 24,
                score
            ));
        }
    }

    lines.push(String::new());
    lines.push("Best days:".to_string());
    if result.best_days.is_empty() {
        lines.push("  Not enough completed sessions yet.".to_string());
    } else {
        for (day, mean_duration) in &result.best_days {
            lines.push(format!(
                "  {} (mean: {})",
                day_name(*day),
                format_minutes(*mean_duration)
            ));
        }
    }

    lines.push(String::new());
    lines.push("Trend model:".to_string());
    match (&result.trend_slope, &result.trend_label) {
        (Some(slope), Some(label)) => {
            lines.push(format!("  {} (slope: {:.2} min/day)", label, slope));
        }
        _ => {
            lines.push("  Needs at least 7 calendar days of data — collecting data.".to_string());
        }
    }

    lines.push(String::new());
    lines.push("Decision tree quality model:".to_string());
    match (&result.tree_rules, &result.quality_rate) {
        (Some(rules), Some(rate)) => {
            lines.push(format!("  Quality rate: {:.1}%", rate));
            for line in rules.lines() {
                lines.push(format!("  {}", line));
            }
        }
        _ => {
            let remaining = TREE_MIN_SESSIONS.saturating_sub(result.total_sessions);
            lines.push(format!(
                "  Predictive model needs {} more session(s) — collecting data.",
                remaining
            ));
        }
    }

    lines.join("\n")
}

fn distraction_rate(sessions: &[SessionRecord]) -> f32 {
    if sessions.is_empty() {
        return 0.0;
    }

    let interrupted = sessions.iter().filter(|s| s.interrupted).count();
    (interrupted as f32 / sessions.len() as f32) * 100.0
}

fn top_focus_hours(sessions: &[SessionRecord]) -> Vec<(u8, f32)> {
    let mut totals = [0.0_f32; 24];
    let mut counts = [0_u32; 24];

    for session in sessions {
        let hour = session.hour_of_day.min(23) as usize;
        totals[hour] += session.duration_minutes;
        counts[hour] += 1;
    }

    let mut scored: Vec<(u8, f32)> = (0..24)
        .filter(|hour| counts[*hour] > 0)
        .map(|hour| {
            let mean = totals[hour] / counts[hour] as f32;
            let score = mean * counts[hour] as f32;
            (hour as u8, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.truncate(3);
    scored
}

fn best_days(sessions: &[SessionRecord]) -> Vec<(u8, f32)> {
    let mut totals = [0.0_f32; 7];
    let mut counts = [0_u32; 7];

    for session in sessions {
        let day = session.day_of_week.min(6) as usize;
        totals[day] += session.duration_minutes;
        counts[day] += 1;
    }

    let mut scored: Vec<(u8, f32)> = (0..7)
        .filter(|day| counts[*day] > 0)
        .map(|day| (day as u8, totals[day] / counts[day] as f32))
        .collect();

    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.truncate(2);
    scored
}

fn weekly_summary(sessions: &[SessionRecord]) -> (f32, f32) {
    let now = Local::now();
    let current_week_start_date =
        now.date_naive() - ChronoDuration::days(now.weekday().num_days_from_monday() as i64);
    let current_week_start = current_week_start_date
        .and_hms_opt(0, 0, 0)
        .expect("valid current week start");
    let previous_week_start = current_week_start - ChronoDuration::days(7);

    let mut current_week_total = 0.0;
    let mut previous_week_total = 0.0;

    for session in sessions {
        let start = session.start_time.naive_local();
        if start >= current_week_start {
            current_week_total += session.duration_minutes;
        } else if start >= previous_week_start && start < current_week_start {
            previous_week_total += session.duration_minutes;
        }
    }

    (current_week_total, current_week_total - previous_week_total)
}

fn daily_totals(sessions: &[SessionRecord]) -> BTreeMap<NaiveDate, f32> {
    let mut totals = BTreeMap::new();
    for session in sessions {
        *totals.entry(session.start_time.date_naive()).or_insert(0.0) += session.duration_minutes;
    }
    totals
}

fn trend_detection(daily_totals: &BTreeMap<NaiveDate, f32>) -> Option<(f32, String)> {
    let n_days = daily_totals.len();
    if n_days < TREND_MIN_DAYS {
        return None;
    }

    let day_indices: Vec<f64> = (0..n_days).map(|i| i as f64).collect();
    let totals: Vec<f64> = daily_totals
        .values()
        .map(|minutes| *minutes as f64)
        .collect();

    let x = Array2::from_shape_vec((n_days, 1), day_indices).ok()?;
    let y = Array1::from_vec(totals);
    let dataset = linfa::Dataset::new(x, y);
    let model = LinearRegression::default().fit(&dataset).ok()?;
    let slope = model.params()[0] as f32;

    let label = if slope > 0.5 {
        "📈 Focus trending up".to_string()
    } else if slope < -0.5 {
        "📉 Focus declining — protect your blocks".to_string()
    } else {
        "Focus is steady".to_string()
    };

    Some((slope, label))
}

fn decision_tree_summary(sessions: &[SessionRecord]) -> Option<(String, f32)> {
    if sessions.len() < TREE_MIN_SESSIONS {
        return None;
    }

    let feature_values: Vec<f64> = sessions
        .iter()
        .flat_map(|session| [session.hour_of_day as f64, session.day_of_week as f64])
        .collect();
    let label_values: Vec<usize> = sessions
        .iter()
        .map(|session| label_session(session).as_label())
        .collect();

    let features = Array2::from_shape_vec((sessions.len(), 2), feature_values).ok()?;
    let labels = Array1::from_vec(label_values.clone());
    let dataset = linfa::Dataset::new(features, labels);

    let model = DecisionTree::params()
        .split_quality(SplitQuality::Gini)
        .max_depth(Some(4))
        .min_weight_split(5.0)
        .fit(&dataset)
        .ok()?;

    let predictions = model.predict(&dataset);
    let correct = predictions
        .iter()
        .zip(label_values.iter())
        .filter(|(predicted, actual)| **predicted == **actual)
        .count();
    let training_accuracy = (correct as f32 / sessions.len() as f32) * 100.0;

    let quality_count = sessions
        .iter()
        .filter(|session| label_session(session) == SessionQuality::Quality)
        .count();
    let quality_rate = (quality_count as f32 / sessions.len() as f32) * 100.0;
    let predicted_quality_windows = predicted_quality_windows(&model);

    let rules = if predicted_quality_windows.is_empty() {
        format!(
            "Decision tree trained (Gini, max_depth=4, min_samples_split=5).\nTraining accuracy: {:.1}%.\nNo hour/day window is currently predicted as Quality.",
            training_accuracy
        )
    } else {
        format!(
            "Decision tree trained (Gini, max_depth=4, min_samples_split=5).\nTraining accuracy: {:.1}%.\nPredicted quality windows: {}",
            training_accuracy,
            predicted_quality_windows.join(", ")
        )
    };

    Some((rules, quality_rate))
}

fn predicted_quality_windows(model: &DecisionTree<f64, usize>) -> Vec<String> {
    let mut windows = Vec::new();

    for day in 0..7 {
        for hour in 0..24 {
            let query = match Array2::from_shape_vec((1, 2), vec![hour as f64, day as f64]) {
                Ok(query) => query,
                Err(_) => continue,
            };
            let prediction = model.predict(&query);
            let predicted_quality = prediction
                .get(0)
                .map(|label| SessionQuality::from_label(*label))
                == Some(SessionQuality::Quality);

            if predicted_quality {
                windows.push(format!("{} {}:00", day_name(day), hour));
            }
        }
    }

    windows.truncate(12);
    windows
}

fn day_name(day: u8) -> &'static str {
    match day {
        0 => "Mon",
        1 => "Tue",
        2 => "Wed",
        3 => "Thu",
        4 => "Fri",
        5 => "Sat",
        6 => "Sun",
        _ => "Unknown",
    }
}

fn format_minutes(minutes: f32) -> String {
    if minutes > 0.0 && minutes < 1.0 {
        return "<1m".to_string();
    }

    let rounded = minutes.round() as i32;
    let hours = rounded / 60;
    let mins = rounded.abs() % 60;

    if hours == 0 {
        format!("{}m", rounded)
    } else {
        format!("{}h {}m", hours, mins)
    }
}

fn format_delta_minutes(minutes: f32) -> String {
    if minutes >= 0.0 {
        format!("up {} vs last week", format_minutes(minutes))
    } else {
        format!("down {} vs last week", format_minutes(minutes.abs()))
    }
}

```

`src/main.rs`:

```rs
mod analytics;
mod session;

use analytics::{format_analytics, run_analytics};
use chrono::{DateTime, Local};
use lazy_static::lazy_static;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
#[cfg(target_os = "windows")]
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use session::{append_session, load_sessions, sessions_file_path, SessionRecord};

#[cfg(debug_assertions)]
const DEV_MODE: bool = true; // Enable for local development
#[cfg(not(debug_assertions))]
const DEV_MODE: bool = false; // Disable for release/production

// --- Configuration ---
const SERVICE_NAME: &str = "_http._tcp.local.";
const DEVICE_HOSTNAME: &str = "focus-totem";
const FOCUS_WALLPAPER_NAME: &str = "focus_wallpaper.jpg";
const APPS_CONFIG_FILE: &str = "apps.toml";

lazy_static! {
    static ref ORIGINAL_WALLPAPER_PATH: Mutex<Option<String>> = Mutex::new(None);
}

#[derive(Debug, Deserialize)]
struct AppsConfig {
    apps: std::collections::HashMap<String, Vec<String>>,
}

fn default_focus_apps() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        vec![
            r"C:\Windows\System32\notepad.exe".to_string(),
            r"C:\Users\Faizy\AppData\Local\BraveSoftware\Brave-Browser\Application\brave.exe"
                .to_string(),
        ]
    }

    #[cfg(target_os = "linux")]
    {
        vec!["brave-browser".to_string()]
    }

    #[cfg(target_os = "macos")]
    {
        vec!["TextEdit".to_string(), "Google Chrome".to_string()]
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        vec![]
    }
}

fn parse_apps_config(contents: &str) -> Result<AppsConfig, toml::de::Error> {
    toml::from_str(contents)
}

fn select_apps_for_current_os(config: &AppsConfig) -> Option<Vec<String>> {
    config.apps.get(std::env::consts::OS).cloned()
}

fn sanitize_apps(apps: Vec<String>) -> Vec<String> {
    apps.into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn load_focus_apps() -> Vec<String> {
    match fs::read_to_string(APPS_CONFIG_FILE) {
        Ok(contents) => match parse_apps_config(&contents) {
            Ok(config) => {
                let selected = select_apps_for_current_os(&config).unwrap_or_default();
                let cleaned = sanitize_apps(selected);
                if cleaned.is_empty() {
                    let defaults = default_focus_apps();
                    println!(
                        "No apps configured for OS '{}' in '{}'. Using {} default app(s).",
                        std::env::consts::OS,
                        APPS_CONFIG_FILE,
                        defaults.len()
                    );
                    defaults
                } else {
                    println!(
                        "Loaded {} app(s) for OS '{}' from '{}'.",
                        cleaned.len(),
                        std::env::consts::OS,
                        APPS_CONFIG_FILE
                    );
                    cleaned
                }
            }
            Err(e) => {
                eprintln!(
                    "ERROR: Could not parse '{}': {}. Falling back to defaults.",
                    APPS_CONFIG_FILE, e
                );
                default_focus_apps()
            }
        },
        Err(e) => {
            eprintln!(
                "ERROR: Could not read '{}': {}. Falling back to defaults.",
                APPS_CONFIG_FILE, e
            );
            default_focus_apps()
        }
    }
}

fn launch_app(app: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        return Command::new("open").args(["-a", app]).spawn().map(|_| ());
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        return Command::new(app).spawn().map(|_| ());
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unsupported OS: {}", std::env::consts::OS),
        ))
    }
}

fn terminate_app(app: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let image_name = Path::new(app)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(app);

        let output = Command::new("taskkill")
            .args(["/F", "/IM", image_name])
            .output()?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("taskkill failed for '{}': {}", image_name, stderr.trim()),
        ));
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let output = Command::new("pkill").args(["-f", app]).output()?;

        // pkill exit code: 0 => matched/killed, 1 => no matching process (safe for us)
        if output.status.success() || output.status.code() == Some(1) {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("pkill failed for '{}': {}", app, stderr.trim()),
        ));
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unsupported OS: {}", std::env::consts::OS),
        ))
    }
}

#[cfg(target_os = "linux")]
fn gsettings_get_wallpaper_uri() -> Option<String> {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.background", "picture-uri"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let trimmed = raw.trim_matches('\'').trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

#[cfg(target_os = "linux")]
fn gsettings_set_wallpaper_uri(uri: &str) -> bool {
    let status = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri", uri])
        .status();

    match status {
        Ok(s) if s.success() => {
            // Try dark-variant key as well (GNOME 42+ / some distros), ignore failure.
            let _ = Command::new("gsettings")
                .args([
                    "set",
                    "org.gnome.desktop.background",
                    "picture-uri-dark",
                    uri,
                ])
                .status();
            true
        }
        _ => false,
    }
}

#[cfg(target_os = "linux")]
fn path_to_file_uri(path: &str) -> String {
    let normalized = path.replace(' ', "%20");
    if normalized.starts_with("file://") {
        normalized
    } else {
        format!("file://{}", normalized)
    }
}

fn save_original_wallpaper_path() {
    if let Ok(path) = wallpaper::get() {
        println!("Saved original wallpaper: {}", path);
        *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(path);
        return;
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(uri) = gsettings_get_wallpaper_uri() {
            println!("Saved original wallpaper from gsettings: {}", uri);
            *ORIGINAL_WALLPAPER_PATH.lock().unwrap() = Some(uri);
        }
    }
}

fn set_focus_wallpaper() {
    match fs::canonicalize(FOCUS_WALLPAPER_NAME) {
        Ok(absolute_path) => {
            let path_str = match absolute_path.to_str() {
                Some(p) => p,
                None => {
                    eprintln!("ERROR: Focus wallpaper path contains invalid UTF-8.");
                    return;
                }
            };

            #[cfg(target_os = "linux")]
            {
                let uri = path_to_file_uri(path_str);
                if gsettings_set_wallpaper_uri(&uri) {
                    println!("Focus wallpaper has been set (gsettings primary).");
                    return;
                }

                if wallpaper::set_from_path(path_str).is_ok() {
                    println!("Focus wallpaper has been set (wallpaper crate fallback).");
                    return;
                }

                eprintln!(
                    "ERROR: Failed to set focus wallpaper using gsettings and wallpaper crate."
                );
                return;
            }

            #[cfg(not(target_os = "linux"))]
            {
                if wallpaper::set_from_path(path_str).is_ok() {
                    println!("Focus wallpaper has been set (wallpaper crate).");
                    return;
                }

                eprintln!("ERROR: Failed to set focus wallpaper using wallpaper crate.");
            }
        }
        Err(e) => eprintln!(
            "ERROR: Could not find wallpaper '{}': {}",
            FOCUS_WALLPAPER_NAME, e
        ),
    }
}

fn restore_original_wallpaper() {
    let mut original_path = ORIGINAL_WALLPAPER_PATH.lock().unwrap();
    if let Some(path) = original_path.as_deref() {
        #[cfg(target_os = "linux")]
        {
            let uri = if path.starts_with("file://") {
                path.to_string()
            } else {
                path_to_file_uri(path)
            };

            if gsettings_set_wallpaper_uri(&uri) {
                println!("Restored original wallpaper (gsettings primary): {}", uri);
                *original_path = None;
                return;
            }

            if wallpaper::set_from_path(path).is_ok() {
                println!(
                    "Restored original wallpaper (wallpaper crate fallback): {}",
                    path
                );
                *original_path = None;
                return;
            }

            eprintln!(
                "ERROR: Failed to restore original wallpaper using gsettings and wallpaper crate."
            );
            *original_path = None;
            return;
        }

        #[cfg(not(target_os = "linux"))]
        {
            if wallpaper::set_from_path(path).is_ok() {
                println!("Restored original wallpaper (wallpaper crate): {}", path);
                *original_path = None;
                return;
            }

            eprintln!("ERROR: Failed to restore original wallpaper using wallpaper crate.");
        }
    }

    *original_path = None;
}

fn activate_focus_mode(focus_apps: &[String]) {
    println!("Activating focus mode automations...");

    // Save current wallpaper so we can restore it later
    save_original_wallpaper_path();

    // Set focus wallpaper (with Linux GNOME fallback)
    set_focus_wallpaper();

    // Launch apps
    println!(
        "Launching focus applications for {}...",
        std::env::consts::OS
    );
    for app in focus_apps {
        match launch_app(app) {
            Ok(_) => println!("Successfully launched '{}'", app),
            Err(e) => eprintln!("ERROR: Failed to launch '{}': {}", app, e),
        }
    }
}

fn deactivate_focus_mode(focus_apps: &[String]) {
    println!("Deactivating focus mode automations...");

    // Restore wallpaper (with Linux GNOME fallback)
    restore_original_wallpaper();

    // Close apps
    println!("Closing focus applications for {}...", std::env::consts::OS);
    for app in focus_apps {
        if let Err(e) = terminate_app(app) {
            eprintln!("ERROR: Failed to close '{}': {}", app, e);
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

fn discover_device(search_duration: Duration) -> Option<String> {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let receiver = mdns
        .browse(SERVICE_NAME)
        .expect("Failed to browse for service");
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < search_duration {
        if let Ok(event) = receiver.recv_timeout(Duration::from_secs(1)) {
            if let ServiceEvent::ServiceResolved(info) = event {
                if info.get_fullname().contains(DEVICE_HOSTNAME) {
                    let ip = info.get_addresses().iter().next()?;
                    let port = info.get_port();
                    let url = format!("http://{}:{}/status", ip, port);
                    println!("Resolved Focus Totem address: {}", url);
                    return Some(url);
                }
            }
        }
    }

    None
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args
        .iter()
        .any(|arg| arg == "--analytics" || arg == "--report")
    {
        run_analytics_cli();
        return;
    }

    let http_client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("Failed to build HTTP client");

    let focus_apps = load_focus_apps();

    let mut is_focused = false;
    let mut esp32_address: Option<String> = None;
    let mut session_start: Option<DateTime<Local>> = None;

    println!("Starting Focus Mode client (Rust version)...");
    println!("Detected OS: {}", std::env::consts::OS);
    println!("Configured apps: {:?}", focus_apps);

    if DEV_MODE {
        println!("==========================================");
        println!("=== DEVELOPMENT MODE ACTIVE (MOCK ESP32) ===");
        println!("Mock ESP32 endpoint: http://localhost:8080/status");
        println!("To exit mock mode, build release or set DEV_MODE to false.");
        println!("==========================================");
    }

    loop {
        if DEV_MODE {
            if esp32_address.is_none() {
                esp32_address = Some("http://localhost:8080/status".to_string());
                if let Some(address) = &esp32_address {
                    println!("[MOCK] Simulated ESP32 address: {}", address);
                }
            }

            if let Some(address) = &esp32_address {
                match http_client.get(address).send() {
                    Ok(response) => {
                        if response.status().is_success()
                            && response.text().unwrap_or_default() == "FOCUS_ON"
                        {
                            if !is_focused {
                                is_focused = true;
                                println!("[MOCK] --- FOCUS MODE ACTIVATED ---");
                                begin_focus_session(&mut session_start);
                                activate_focus_mode(&focus_apps);
                            }
                        } else if is_focused {
                            is_focused = false;
                            println!("[MOCK] --- FOCUS MODE DEACTIVATED (non-FOCUS_ON) ---");
                            deactivate_focus_mode(&focus_apps);
                            end_focus_session(&mut session_start);
                        }
                    }
                    Err(e) => {
                        eprintln!("[MOCK] Error polling mock ESP32: {:?}", e);
                        if is_focused {
                            is_focused = false;
                            println!("[MOCK] --- FOCUS MODE DEACTIVATED ---");
                            deactivate_focus_mode(&focus_apps);
                            end_focus_session(&mut session_start);
                        }
                        esp32_address = None;
                    }
                }
            }

            thread::sleep(Duration::from_secs(3));
            continue;
        }

        if esp32_address.is_none() {
            println!("Searching for Focus Totem on the network...");
            if let Some(found_address) = discover_device(Duration::from_secs(5)) {
                esp32_address = Some(found_address);
            } else {
                println!("Device not found. Will retry in 4 seconds.");
                thread::sleep(Duration::from_secs(4));
            }
        }

        if let Some(address) = &esp32_address {
            match http_client.get(address).send() {
                Ok(response) => {
                    if response.status().is_success()
                        && response.text().unwrap_or_default() == "FOCUS_ON"
                    {
                        if !is_focused {
                            is_focused = true;
                            println!("--- FOCUS MODE ACTIVATED ---");
                            begin_focus_session(&mut session_start);
                            activate_focus_mode(&focus_apps);
                        }
                    }
                }
                Err(_) => {
                    if is_focused {
                        is_focused = false;
                        println!("--- FOCUS MODE DEACTIVATED ---");
                        deactivate_focus_mode(&focus_apps);
                        end_focus_session(&mut session_start);
                    }
                    println!("Lost connection to device. Returning to search mode.");
                    esp32_address = None;
                }
            }
        }

        thread::sleep(Duration::from_secs(3));
    }
}

```

`src/session.rs`:

```rs
use chrono::{DateTime, Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

static FIRST_WRITE_PATH_LOGGED: OnceLock<()> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub duration_minutes: f32,
    pub hour_of_day: u8,
    pub day_of_week: u8,
    pub interrupted: bool,
}

impl SessionRecord {
    pub fn new(start_time: DateTime<Local>, end_time: DateTime<Local>) -> Self {
        let duration = end_time.signed_duration_since(start_time);
        let duration_minutes = (duration.num_seconds().max(0) as f32) / 60.0;
        let hour_of_day = start_time.hour() as u8;
        let day_of_week = start_time.weekday().num_days_from_monday() as u8;
        let interrupted = duration_minutes < 10.0;

        Self {
            start_time,
            end_time,
            duration_minutes,
            hour_of_day,
            day_of_week,
            interrupted,
        }
    }
}

pub fn sessions_file_path() -> PathBuf {
    if let Some(mut data_dir) = dirs::data_dir() {
        #[cfg(target_os = "windows")]
        data_dir.push("FocusTotem");

        #[cfg(not(target_os = "windows"))]
        data_dir.push("focus_totem");

        data_dir.push("sessions.json");
        return data_dir;
    }

    PathBuf::from("sessions.json")
}

pub fn load_sessions() -> io::Result<Vec<SessionRecord>> {
    read_sessions(&sessions_file_path())
}

pub fn append_session(record: SessionRecord) -> io::Result<PathBuf> {
    let path = sessions_file_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut sessions = read_sessions(&path)?;
    sessions.push(record);
    write_sessions_atomically(&path, &sessions)?;

    FIRST_WRITE_PATH_LOGGED.get_or_init(|| {
        println!("Session log file: {}", path.display());
    });

    Ok(path)
}

fn read_sessions(path: &Path) -> io::Result<Vec<SessionRecord>> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            if contents.trim().is_empty() {
                return Ok(Vec::new());
            }

            serde_json::from_str(&contents).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse existing sessions JSON: {e}"),
                )
            })
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

fn write_sessions_atomically(path: &Path, sessions: &[SessionRecord]) -> io::Result<()> {
    let tmp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(sessions).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize sessions JSON: {e}"),
        )
    })?;

    fs::write(&tmp_path, json)?;

    // On Unix-like systems, renaming over an existing file is atomic.
    // On Windows, std::fs::rename fails if the destination exists, so remove first.
    #[cfg(target_os = "windows")]
    {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }

    fs::rename(tmp_path, path)?;

    Ok(())
}

```

`totem.cpp`:

```cpp
#include <WiFi.h>
#include <ESPmDNS.h>
#include <WebServer.h>

// --- IMPORTANT: CHANGE THESE TO YOUR WI-FI CREDENTIALS ---
const char* ssid = "YOUR_WIFI_NAME";
const char* password = "YOUR_WIFI_PASSWORD";

// Create a WebServer object that will listen on port 80
WebServer server(80);

void handleStatus() {
  // This function is called when a client requests the /status URL
  Serial.println("Client requested status. Sending FOCUS_ON...");
  server.send(200, "text/plain", "FOCUS_ON"); // Send the response
}

void setup() {
  // Start the serial monitor for debugging
  Serial.begin(115200);
  Serial.println(); // Print a blank line

  // --- 1. Connect to Wi-Fi ---
  Serial.print("Connecting to ");
  Serial.println(ssid);
  WiFi.begin(ssid, password);

  // Wait for the connection to complete
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }

  Serial.println("");
  Serial.println("WiFi connected!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());

  // --- 2. Start mDNS ---
  // This announces the ESP32 on the network as 'focus-totem.local'
  if (!MDNS.begin("focus-totem")) {
    Serial.println("Error setting up mDNS responder!");
    while(1) { delay(1000); } // Halt if mDNS fails
  }
  Serial.println("mDNS responder started");

  // Announce that we are an HTTP (web) server
  MDNS.addService("http", "tcp", 80);
  Serial.println("Announced http service on port 80");
  
  // --- 3. Configure and Start the Web Server ---
  // Tell the server which function to call when it gets a request to "/status"
  server.on("/status", HTTP_GET, handleStatus);

  // Start the server
  server.begin();
  Serial.println("Web server started");
}

void loop() {
  // This is required for the server to process incoming client requests
  server.handleClient();
}
```