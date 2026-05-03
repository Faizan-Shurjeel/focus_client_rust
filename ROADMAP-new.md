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
