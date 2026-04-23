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