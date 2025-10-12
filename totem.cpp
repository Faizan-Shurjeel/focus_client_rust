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