from http.server import BaseHTTPRequestHandler, HTTPServer


class MockESP32Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/status":
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.end_headers()
            self.wfile.write(b"FOCUS_ON")
        else:
            self.send_response(404)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.end_headers()
            self.wfile.write(b"Not Found")

    def log_message(self, format, *args):
        # Keep logs concise and prefixed for easier debugging.
        print(f"[Mock ESP32] {self.address_string()} - {format % args}")


def run_server(host: str = "127.0.0.1", port: int = 8080):
    server_address = (host, port)
    httpd = HTTPServer(server_address, MockESP32Handler)
    print(f"[Mock ESP32] Running on http://{host}:{port}")
    print("[Mock ESP32] Endpoint: GET /status -> FOCUS_ON")
    print("[Mock ESP32] Press Ctrl+C to stop.")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n[Mock ESP32] Shutting down...")
    finally:
        httpd.server_close()


if __name__ == "__main__":
    run_server()
