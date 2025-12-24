#include <WiFi.h>
#include <ESPmDNS.h>
#include <WebServer.h>
#include <ArduinoJson.h>

// --- IMPORTANT: CHANGE THESE TO YOUR WI-FI CREDENTIALS ---
const char *ssid = "realme 9";
const char *password = "i885qfej";

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