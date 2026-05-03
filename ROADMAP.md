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
