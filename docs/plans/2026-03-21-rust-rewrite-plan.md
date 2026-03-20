# Rust Rewrite Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rewrite claude-statusline-config from Node.js to Rust as a single zero-dependency binary handling both TUI wizard and statusline rendering.

**Architecture:** Single crate, single binary. No args = TUI wizard, `--render` = statusline renderer called by Claude Code. All files in `~/.claude/statusline/`. npm distribution via thin JS shell that downloads prebuilt binary.

**Tech Stack:** Rust, serde_json, crossterm, ureq, dirs

**Design doc:** `docs/plans/2026-03-21-rust-rewrite-design.md`

---

### Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "claude-statusline-config"
version = "2.0.0"
edition = "2021"
description = "Interactive CLI tool to configure Claude Code statusline"
license = "MIT"
repository = "https://github.com/LokiQ0713/claude-statusline-config"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
crossterm = "0.28"
ureq = "2"
dirs = "6"

[profile.release]
opt-level = "z"
lto = true
strip = true
panic = "abort"
```

**Step 2: Create minimal main.rs**

```rust
mod config;
mod i18n;
mod styles;
mod render;
mod install;
mod wizard;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--render") {
        render::run();
    } else {
        wizard::run();
    }
}
```

**Step 3: Create stub modules so it compiles**

Create `src/config.rs`, `src/i18n.rs`, `src/styles.rs`, `src/render.rs`, `src/install.rs`, `src/wizard/mod.rs` — all empty or with placeholder `pub fn run() {}`.

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles with no errors.

**Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/
git commit -m "feat: rust project scaffold with stub modules"
```

---

### Task 2: Config Module

**Files:**
- Create: `src/config.rs`

**Reference:** JS `config.js` — Config struct matches the JSON format from the design doc.

**Step 1: Write config structs and defaults**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn statusline_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".claude").join("statusline")
}

pub fn config_path() -> PathBuf {
    statusline_dir().join("config.json")
}

pub fn bin_path() -> PathBuf {
    statusline_dir().join("bin")
}

pub fn log_path() -> PathBuf {
    statusline_dir().join("statusline.log")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub order: Vec<String>,
    pub segments: Segments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segments {
    pub model: ModelSegment,
    pub cost: CostSegment,
    pub usage: UsageSegment,
    pub path: PathSegment,
    pub git: GitSegment,
    pub context: ContextSegment,
    pub crypto: CryptoSegment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSegment {
    pub enabled: bool,
    pub style: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSegment {
    pub enabled: bool,
    pub style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSegment {
    pub enabled: bool,
    pub style: String,
    pub bar_char: String,
    pub bar_length: usize,
    pub show_bar: bool,
    pub show_percent: bool,
    pub show_reset: bool,
    pub refresh_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathSegment {
    pub enabled: bool,
    pub style: String,
    pub max_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitSegment {
    pub enabled: bool,
    pub style: String,
    pub show_dirty: bool,
    pub show_remote: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSegment {
    pub enabled: bool,
    pub style: String,
    pub bar_char: String,
    pub bar_length: usize,
    pub show_bar: bool,
    pub show_percent: bool,
    pub show_size: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CryptoSegment {
    pub enabled: bool,
    pub style: String,
    pub refresh_interval: u64,
    pub coins: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            lang: None,
            order: vec![
                "model".into(), "cost".into(), "usage".into(),
                "path".into(), "git".into(), "context".into(), "crypto".into(),
            ],
            segments: Segments {
                model: ModelSegment { enabled: true, style: "ultrathink".into(), icon: String::new() },
                cost: CostSegment { enabled: true, style: "green".into() },
                usage: UsageSegment {
                    enabled: true, style: "semantic".into(), bar_char: "shade".into(),
                    bar_length: 8, show_bar: false, show_percent: true, show_reset: true,
                    refresh_interval: 120,
                },
                path: PathSegment { enabled: true, style: "cyan".into(), max_length: 15 },
                git: GitSegment { enabled: true, style: "cyan".into(), show_dirty: true, show_remote: true },
                context: ContextSegment {
                    enabled: true, style: "ultrathink-gradient".into(), bar_char: "shade".into(),
                    bar_length: 12, show_bar: true, show_percent: true, show_size: true,
                },
                crypto: CryptoSegment {
                    enabled: true, style: "green".into(), refresh_interval: 60, coins: vec!["BTC".into()],
                },
            },
        }
    }
}

pub fn load_config() -> Config {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save_config(config: &Config) -> std::io::Result<()> {
    let dir = statusline_dir();
    fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(config).unwrap();
    fs::write(config_path(), json + "\n")
}
```

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/config.rs
git commit -m "feat: config module with structs, defaults, load/save"
```

---

### Task 3: I18n Module

**Files:**
- Create: `src/i18n.rs`

**Reference:** JS `i18n.js` — Port all en/zh message keys. Use a global `AtomicU8` for language state, a `t!` macro or function for lookups.

**Step 1: Write i18n module**

Use a `static` language variable and a `t()` function. Port all keys from JS `i18n.js` (lines 1-213). Dynamic messages (with args) use format strings.

Structure:
```rust
use std::sync::atomic::{AtomicU8, Ordering};

static LANG: AtomicU8 = AtomicU8::new(0); // 0=en, 1=zh

pub const SUPPORTED_LANGS: &[&str] = &["en", "zh"];

pub fn set_lang(lang: &str) {
    LANG.store(if lang == "zh" { 1 } else { 0 }, Ordering::Relaxed);
}

pub fn get_lang() -> &'static str {
    if LANG.load(Ordering::Relaxed) == 1 { "zh" } else { "en" }
}

pub fn t(key: &str) -> &'static str {
    let zh = LANG.load(Ordering::Relaxed) == 1;
    match key {
        "step.start" => if zh { "开始" } else { "Start" },
        "step.segments" => if zh { "1/4 选择段落" } else { "1/4 Segments" },
        // ... all keys from i18n.js
        _ => key,
    }
}

// For dynamic messages with arguments
pub fn tf(key: &str, args: &[&str]) -> String {
    let zh = LANG.load(Ordering::Relaxed) == 1;
    match key {
        "step.configSegment" => {
            if zh { format!("2/4 配置段落 ({}/{})", args[0], args[1]) }
            else { format!("2/4 Configure ({}/{})", args[0], args[1]) }
        },
        "msg.missingDeps" => {
            if zh { format!("缺少必要依赖: {}", args[0]) }
            else { format!("Missing dependencies: {}", args[0]) }
        },
        // ... all dynamic keys
        _ => key.to_string(),
    }
}
```

Port every key from JS `i18n.js` lines 1-196. There are ~60 keys total.

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/i18n.rs
git commit -m "feat: i18n module with en/zh translations"
```

---

### Task 4: Styles Module

**Files:**
- Create: `src/styles.rs`

**Reference:** JS `styles.js` — Port COLORS map, ULTRATHINK arrays, BAR_CHARS_DATA, and preview functions.

**Step 1: Write styles module**

```rust
/// ANSI color escape sequences
pub fn color_code(style: &str) -> &'static str {
    match style {
        "cyan" => "\x1b[0;36m",
        "green" => "\x1b[0;32m",
        "blue" => "\x1b[1;34m",
        "yellow" => "\x1b[0;33m",
        "magenta" => "\x1b[0;35m",
        "red" => "\x1b[0;31m",
        "white" => "\x1b[0;37m",
        "soft-green" => "\x1b[38;5;71m",
        "soft-yellow" => "\x1b[38;5;179m",
        "soft-red" => "\x1b[38;5;167m",
        "soft-blue" => "\x1b[38;5;75m",
        "soft-cyan" => "\x1b[38;5;80m",
        "soft-magenta" => "\x1b[38;5;176m",
        "orange" => "\x1b[38;5;208m",
        "pink" => "\x1b[38;5;212m",
        "purple" => "\x1b[38;5;141m",
        _ => "\x1b[0;37m",
    }
}

pub const RESET: &str = "\x1b[0m";
pub const DIM: &str = "\x1b[38;5;239m";

/// Ultrathink 7-color rainbow
pub struct UltrathinkColors {
    pub main: [(u8, u8, u8); 7],
    pub shimmer: [(u8, u8, u8); 7],
}

pub const ULTRATHINK: UltrathinkColors = UltrathinkColors {
    main: [
        (235, 95, 87), (245, 139, 87), (250, 195, 95),
        (145, 200, 130), (130, 170, 220), (155, 130, 200), (200, 130, 180),
    ],
    shimmer: [
        (250, 155, 147), (255, 185, 137), (255, 225, 155),
        (185, 230, 180), (180, 205, 240), (195, 180, 230), (230, 180, 210),
    ],
};

pub struct BarChars {
    pub filled: char,
    pub empty: char,
    pub empty_uses_dim: bool, // true = same char with dim color
}

pub fn bar_chars(name: &str) -> BarChars {
    match name {
        "full-block" => BarChars { filled: '█', empty: '█', empty_uses_dim: true },
        "rectangle" => BarChars { filled: '▬', empty: '▬', empty_uses_dim: true },
        _ /* shade */ => BarChars { filled: '▓', empty: '░', empty_uses_dim: false },
    }
}
```

Also port:
- `format_rainbow(text, offset, use_shimmer)` → returns ANSI rainbow string
- `format_bar(style, bar_char, length, ratio, timestamp)` → returns ANSI bar string
- `format_colored(style, text, timestamp)` → returns colored or rainbow string
- `semantic_color(pct)` → returns green/yellow/red based on percentage

These are used by both render mode and wizard preview.

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/styles.rs
git commit -m "feat: styles module with colors, ultrathink, bar rendering"
```

---

### Task 5: Render Mode — Stdin Parsing

**Files:**
- Create: `src/render.rs`

**Reference:** JS `generator.js` `genInputParsing()` and `genFormatting()` — parse the JSON that Claude Code pipes via stdin, format model name and path.

**Step 1: Write stdin JSON structs and parsing**

```rust
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize, Default)]
pub struct StdinInput {
    #[serde(default)]
    pub model: StdinModel,
    #[serde(default)]
    pub workspace: StdinWorkspace,
    #[serde(default)]
    pub context_window: StdinContext,
    #[serde(default)]
    pub cost: StdinCost,
}

#[derive(Deserialize, Default)]
pub struct StdinModel {
    #[serde(default)]
    pub id: String,
}

#[derive(Deserialize, Default)]
pub struct StdinWorkspace {
    #[serde(default)]
    pub current_dir: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct StdinContext {
    #[serde(default)]
    pub context_window_size: Option<u64>,
    #[serde(default)]
    pub used_percentage: Option<f64>,
}

#[derive(Deserialize, Default)]
pub struct StdinCost {
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
}

fn read_stdin() -> StdinInput {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap_or_default();
    serde_json::from_str(&buf).unwrap_or_default()
}

/// "claude-opus-4-6" → "Opus4.6"
fn format_model(id: &str) -> String {
    let s = id.strip_prefix("claude-").unwrap_or(id);
    // "opus-4-6" → "Opus4.6", "sonnet-4-6" → "Sonnet4.6"
    // Split on first '-' between name and version
    // Pattern: name-major-minor[...] → NameMajor.Minor
    let parts: Vec<&str> = s.splitn(3, '-').collect();
    if parts.len() >= 3 {
        let name = capitalize(parts[0]);
        format!("{}{}.{}", name, parts[1], parts[2].split('[').next().unwrap_or(parts[2]))
    } else {
        s.to_string()
    }
}

/// "~/Desktop/very-long-path" → truncate to dir name if > maxLength
fn format_path(cwd: &str, home: &str, max_length: usize) -> String {
    let p = if cwd.starts_with(home) {
        format!("~{}", &cwd[home.len()..])
    } else {
        cwd.to_string()
    };
    if p.len() > max_length {
        p.rsplit('/').next().unwrap_or("/").to_string()
    } else {
        p
    }
}

/// Format bytes: 1000000 → "1M", 600000 → "600K"
fn format_size(n: u64) -> String {
    if n >= 1_000_000 {
        let v = n as f64 / 1_000_000.0;
        if v == v.floor() { format!("{}M", v as u64) } else { format!("{:.1}M", v) }
    } else if n >= 1000 {
        let v = n as f64 / 1000.0;
        if v == v.floor() { format!("{}K", v as u64) } else { format!("{:.1}K", v) }
    } else {
        n.to_string()
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/render.rs
git commit -m "feat: render stdin parsing and formatting helpers"
```

---

### Task 6: Render Mode — Segment Rendering + Output

**Files:**
- Modify: `src/render.rs`

**Reference:** JS `generator.js` `genOutput()`, `genContextBar()`, `genUsageOutput()` — render each segment as ANSI string.

**Step 1: Write the main render pipeline**

Add to `render.rs`:

```rust
pub fn run() {
    let config = crate::config::load_config();
    let input = read_stdin();
    let home = dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

    let mut parts: Vec<String> = Vec::new();

    for key in &config.order {
        let seg = render_segment(key, &config.segments, &input, &home, now);
        if let Some(s) = seg {
            parts.push(s);
        }
    }

    print!("{}", parts.join(" "));
}

fn render_segment(key: &str, segs: &Segments, input: &StdinInput, home: &str, now: u64) -> Option<String> {
    match key {
        "model" if segs.model.enabled => {
            let name = format_model(&input.model.id);
            let text = if segs.model.icon.is_empty() { name } else { format!("{} {}", segs.model.icon, name) };
            Some(crate::styles::format_colored(&segs.model.style, &text, now))
        },
        "cost" if segs.cost.enabled => {
            input.cost.total_cost_usd.map(|c| {
                let text = format!("${:.2}", c);
                crate::styles::format_colored(&segs.cost.style, &text, now)
            })
        },
        "path" if segs.path.enabled => {
            let cwd = input.workspace.current_dir.as_deref()
                .or(input.workspace.cwd.as_deref()).unwrap_or("?");
            let text = format_path(cwd, home, segs.path.max_length);
            Some(crate::styles::format_colored(&segs.path.style, &text, now))
        },
        "git" if segs.git.enabled => render_git(&segs.git, now),
        "context" if segs.context.enabled => render_context(&segs.context, input, now),
        "usage" if segs.usage.enabled => render_usage(&segs.usage, now),
        "crypto" if segs.crypto.enabled => render_crypto(&segs.crypto, now),
        _ => None,
    }
}
```

Then implement `render_context`, `render_usage`, `render_crypto`, `render_git` functions:

- `render_context`: read used_percentage + context_window_size from stdin, format bar + percent + size using `styles::format_bar` and `styles::format_colored`
- `render_git`: call `git rev-parse --abbrev-ref HEAD`, `git status --porcelain`, `git rev-list --left-right --count HEAD...@{u}` via `std::process::Command`
- `render_usage` and `render_crypto`: read from cache files (Task 7 handles HTTP/cache)

**Step 2: Verify it compiles and test with mock stdin**

Run: `echo '{"model":{"id":"claude-opus-4-6"},"cost":{"total_cost_usd":0.42},"context_window":{"context_window_size":1000000,"used_percentage":60}}' | cargo run -- --render`

Expected: Colored ANSI output with model, cost, context segments.

**Step 3: Commit**

```bash
git add src/render.rs
git commit -m "feat: render segment pipeline with model/cost/path/git/context output"
```

---

### Task 7: Cache + HTTP (Usage & Crypto)

**Files:**
- Create: `src/cache.rs`
- Modify: `src/render.rs` (wire up usage/crypto rendering)
- Modify: `src/main.rs` (add `mod cache`)

**Reference:** JS `generator.js` `genCryptoFetcher()` and `genUsageFetcher()` — file-based cache with background refresh.

**Step 1: Write cache module**

```rust
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Read cache if fresh enough, otherwise return stale + spawn background refresh.
pub fn read_or_refresh(
    cache_path: &Path,
    lock_path: &Path,
    max_age_secs: u64,
    fetch_fn: fn() -> Option<String>,
) -> Option<String> {
    let content = fs::read_to_string(cache_path).ok();
    let age = file_age_secs(cache_path);

    if age.map_or(true, |a| a >= max_age_secs) {
        // Try to acquire lock (mkdir-based, same as .sh version)
        if fs::create_dir(lock_path).is_ok() {
            std::thread::spawn(move || {
                let lock = lock_path.to_path_buf();
                let cache = cache_path.to_path_buf();
                if let Some(data) = fetch_fn() {
                    let _ = fs::write(&cache, &data);
                } else {
                    // Touch file so we don't retry immediately
                    let _ = fs::write(&cache, "");
                }
                let _ = fs::remove_dir(&lock);
            });
        }
    }

    content.filter(|s| !s.is_empty())
}

fn file_age_secs(path: &Path) -> Option<u64> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    SystemTime::now().duration_since(modified).ok().map(|d| d.as_secs())
}
```

**Step 2: Write crypto fetcher**

```rust
/// Fetch prices from Binance API
pub fn fetch_crypto(coins: &[String]) -> Option<String> {
    let pairs: Vec<String> = coins.iter().map(|c| format!("{}USDT", c)).collect();
    let prices: Vec<String> = pairs.iter().map(|pair| {
        let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", pair);
        ureq::get(&url).call().ok()
            .and_then(|r| r.into_json::<serde_json::Value>().ok())
            .and_then(|v| v["price"].as_str().map(String::from))
            .unwrap_or_default()
    }).collect();
    if prices.iter().all(|p| !p.is_empty()) {
        Some(prices.join("|"))
    } else {
        None
    }
}
```

**Step 3: Write usage fetcher**

```rust
/// Fetch 5h usage from Anthropic API (macOS: read token from Keychain)
pub fn fetch_usage() -> Option<String> {
    let token = get_oauth_token()?;
    let resp = ureq::get("https://api.anthropic.com/api/oauth/usage")
        .set("Authorization", &format!("Bearer {}", token))
        .set("anthropic-beta", "oauth-2025-04-20")
        .set("User-Agent", "claude-statusline-config/2.0.0")
        .call().ok()?;
    let json: serde_json::Value = resp.into_json().ok()?;
    let utilization = json["five_hour"]["utilization"].as_f64()?;
    let resets_at = json["five_hour"]["resets_at"].as_str().unwrap_or("");
    Some(format!("{}|{}", utilization as u64, resets_at))
}

#[cfg(target_os = "macos")]
fn get_oauth_token() -> Option<String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output().ok()?;
    if !output.status.success() { return None; }
    let creds: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    creds["claudeAiOauth"]["accessToken"].as_str().map(String::from)
}

#[cfg(not(target_os = "macos"))]
fn get_oauth_token() -> Option<String> {
    // Linux: read from ~/.claude/credentials or similar
    // TODO: implement Linux credential reading
    None
}
```

**Step 4: Wire into render.rs**

Implement `render_usage` and `render_crypto` using cache module:

```rust
fn render_crypto(seg: &CryptoSegment, now: u64) -> Option<String> {
    let cache = Path::new("/tmp/claude-statusline-crypto-cache");
    let lock = Path::new("/tmp/claude-statusline-crypto-lock");
    let coins = seg.coins.clone();
    let data = cache::read_or_refresh(cache, lock, seg.refresh_interval, move || {
        cache::fetch_crypto(&coins)
    })?;
    let prices: Vec<&str> = data.split('|').collect();
    let parts: Vec<String> = seg.coins.iter().zip(prices.iter()).map(|(coin, price)| {
        let p: f64 = price.parse().unwrap_or(0.0);
        format!("{}:${:.0}", coin, p)
    }).collect();
    Some(styles::format_colored(&seg.style, &parts.join(" "), now))
}
```

Similar for `render_usage` — parse cache "pct|resets_at", compute countdown, format bar+percent+reset.

**Step 5: Verify end-to-end render**

Run: `echo '{"model":{"id":"claude-opus-4-6"},"cost":{"total_cost_usd":0.42},"context_window":{"context_window_size":1000000,"used_percentage":60}}' | cargo run -- --render`

Expected: Full statusline with all segments.

**Step 6: Commit**

```bash
git add src/cache.rs src/render.rs src/main.rs
git commit -m "feat: cache module + HTTP fetchers for usage and crypto"
```

---

### Task 8: Error Logging

**Files:**
- Create: `src/log.rs`
- Modify: `src/main.rs` (add `mod log`)
- Modify: `src/render.rs` (use log on errors)

**Step 1: Write log module**

```rust
use std::fs::{self, OpenOptions};
use std::io::Write;

const MAX_LOG_SIZE: u64 = 100_000; // 100KB

pub fn error(msg: &str) {
    let path = crate::config::log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Truncate if over size limit
    if let Ok(meta) = fs::metadata(&path) {
        if meta.len() > MAX_LOG_SIZE {
            let _ = fs::write(&path, ""); // truncate
        }
    }

    let now = chrono_free_timestamp(); // avoid chrono dependency
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "[{}] ERROR: {}", now, msg);
    }
}

fn chrono_free_timestamp() -> String {
    // Use std::process::Command to get formatted time, or calculate from UNIX_EPOCH
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    // Simple UTC timestamp: enough for log debugging
    format!("{}", secs)
}
```

**Step 2: Wire into render.rs**

Replace all `unwrap_or_default()` in render pipeline with proper error logging:
```rust
let input = match serde_json::from_str::<StdinInput>(&buf) {
    Ok(v) => v,
    Err(e) => {
        crate::log::error(&format!("stdin parse: {}", e));
        StdinInput::default()
    }
};
```

**Step 3: Commit**

```bash
git add src/log.rs src/render.rs src/main.rs
git commit -m "feat: error logging to ~/.claude/statusline/statusline.log"
```

---

### Task 9: TUI Terminal Layer

**Files:**
- Create: `src/wizard/terminal.rs`

**Reference:** JS `cli.js` lines 94-99 (ANSI cursor positioning), lines 120-150 (keypress interception).

**Step 1: Write terminal abstraction**

```rust
use crossterm::{
    cursor, terminal,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::Print,
};
use std::io::{self, Write, stdout};

pub fn enable_raw_mode() {
    terminal::enable_raw_mode().unwrap();
}

pub fn disable_raw_mode() {
    terminal::disable_raw_mode().unwrap();
}

pub fn clear_screen() {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0)).unwrap();
}

pub fn move_to(row: u16, col: u16) {
    execute!(stdout(), cursor::MoveTo(col, row)).unwrap();
}

pub fn clear_line() {
    execute!(stdout(), terminal::Clear(terminal::ClearType::CurrentLine)).unwrap();
}

pub fn print_at(row: u16, text: &str) {
    // Save cursor, move to row, clear line, print, restore cursor
    let mut out = stdout();
    queue!(out, cursor::SavePosition, cursor::MoveTo(0, row),
           terminal::Clear(terminal::ClearType::CurrentLine),
           Print(text), cursor::RestorePosition).unwrap();
    out.flush().unwrap();
}

pub fn flush() {
    stdout().flush().unwrap();
}

pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Space,
    Char(char),
    Escape,
    CtrlC,
}

pub fn read_key() -> Key {
    loop {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read().unwrap() {
            return match code {
                KeyCode::Up => Key::Up,
                KeyCode::Down => Key::Down,
                KeyCode::Left => Key::Left,
                KeyCode::Right => Key::Right,
                KeyCode::Enter => Key::Enter,
                KeyCode::Char(' ') => Key::Space,
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Key::CtrlC,
                KeyCode::Char(c) => Key::Char(c),
                KeyCode::Esc => Key::Escape,
                _ => continue,
            };
        }
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/wizard/terminal.rs
git commit -m "feat: TUI terminal layer with crossterm key handling"
```

---

### Task 10: TUI Components — Select

**Files:**
- Create: `src/wizard/select.rs`

**Reference:** JS `cli.js` `liveSelect()` — single-select with live preview on arrow keys.

**Step 1: Write select component**

```rust
use super::terminal::{self, Key};

pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub hint: Option<String>,
}

pub enum SelectResult {
    Selected(String),   // Enter / →
    Back,               // ←
    Cancelled,          // Esc / Ctrl+C
}

/// Returns SelectResult: Selected, Back, or Cancelled.
/// `on_change` is called on every arrow key for live preview.
pub fn select(
    message: &str,
    options: &[SelectOption],
    initial: Option<&str>,
    on_change: &mut dyn FnMut(&str),
) -> SelectResult {
    let mut idx = initial
        .and_then(|v| options.iter().position(|o| o.value == v))
        .unwrap_or(0);

    draw_select(message, options, idx);

    terminal::enable_raw_mode();
    let result = loop {
        match terminal::read_key() {
            Key::Up => {
                if idx > 0 {
                    idx -= 1;
                    on_change(&options[idx].value);
                    draw_select(message, options, idx);
                }
            }
            Key::Down => {
                if idx < options.len() - 1 {
                    idx += 1;
                    on_change(&options[idx].value);
                    draw_select(message, options, idx);
                }
            }
            Key::Enter | Key::Right => break SelectResult::Selected(options[idx].value.clone()),
            Key::Left => break SelectResult::Back,
            Key::Escape | Key::CtrlC => break SelectResult::Cancelled,
            _ => {}
        }
    };
    terminal::disable_raw_mode();
    result
}
```

`draw_select` renders the option list with `●` for selected, `○` for others, using @clack/prompts visual style (vertical line `│` on left side).

No wrap-around: `idx > 0` and `idx < len - 1` guards.

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/wizard/select.rs
git commit -m "feat: TUI select component with live preview callback"
```

---

### Task 11: TUI Components — Multiselect, Confirm, Spinner

**Files:**
- Create: `src/wizard/multiselect.rs`
- Create: `src/wizard/confirm.rs`
- Create: `src/wizard/spinner.rs`

**Step 1: Write multiselect**

Same pattern as select but with a `HashSet<String>` for selected items. Space toggles, Enter/→ confirms, ← goes back. `on_change` called on every Space toggle for live preview. Returns `MultiselectResult::Selected(Vec)`, `Back`, or `Cancelled`.

No wrap-around on up/down.

**Step 2: Write confirm**

Simple y/n prompt. Returns `bool`. Handle `y`, `n`, `Enter` (default value).

**Step 3: Write spinner**

Braille animation `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` running in a background thread. `start(message)` begins animation, `stop(message)` ends it.

**Step 4: Verify it compiles**

Run: `cargo build`

**Step 5: Commit**

```bash
git add src/wizard/multiselect.rs src/wizard/confirm.rs src/wizard/spinner.rs
git commit -m "feat: TUI multiselect, confirm, and spinner components"
```

---

### Task 12: TUI Preview + Vertical Step Progress

**Files:**
- Create: `src/wizard/preview.rs`
- Create: `src/wizard/step_progress.rs`

**Reference:** JS `cli.js` `renderPreview()`, `renderSegment()`, `updatePreviewInPlace()` — renders the preview bar with sample data, updates in-place via cursor positioning.

**Step 1: Write preview rendering**

Port `renderSegment` for each segment type using sample data:
- model: `"Opus4.6"` with icon
- cost: `"$0.42"`
- usage: bar + `"25%"` + `"1h43m"`
- path: `"~/Desktop/web3"`
- git: `"main*"` + `" ↑2↓1"`
- context: bar + `"60%"` + `"600K/1M"`
- crypto: `"BTC:$73748"`

Use `styles::format_colored` and `styles::format_bar` for coloring.

`update_preview_in_place(config)` uses `terminal::print_at(PREVIEW_ROW, ...)` to update line 4.

**Step 2: Write vertical step progress bar**

```rust
pub struct StepInfo {
    pub label: String,          // "1/4 Select segments"
    pub summary: Option<String>, // "Segments: Model, Cost, Context" (set after completion)
}

pub enum StepStatus {
    Completed,  // ●
    Current,    // ◆
    Pending,    // ○
}

/// Render the vertical step progress bar to a string
pub fn render_step_progress(steps: &[StepInfo], current_idx: usize) -> String {
    let mut lines = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let status = if i < current_idx {
            StepStatus::Completed
        } else if i == current_idx {
            StepStatus::Current
        } else {
            StepStatus::Pending
        };

        let icon = match status {
            StepStatus::Completed => "●",   // filled — dim color
            StepStatus::Current => "◆",     // diamond — bright/highlighted
            StepStatus::Pending => "○",     // empty — dim color
        };

        // Step label line
        lines.push(format!("  {} {}", icon, step.label));

        // Summary line (only for completed steps)
        if let Some(ref summary) = step.summary {
            lines.push(format!("  │   {}", summary)); // dim text
        }

        // Connector line (except last step)
        if i < steps.len() - 1 {
            lines.push("  │".to_string());
        }
    }
    lines.join("\n")
}
```

Visual output:
```
  ● 1/4 Select segments
  │   Segments: Model, Cost, Context
  │
  ◆ 2/4 Configure (1/5)
  │
  ○ 3/4 Reorder
  │
  ○ 4/4 Confirm
```

**Step 3: Verify it compiles**

Run: `cargo build`

**Step 4: Commit**

```bash
git add src/wizard/preview.rs src/wizard/step_progress.rs
git commit -m "feat: TUI preview bar + vertical step progress indicator"
```

---

### Task 13: Wizard Main Flow (State Machine with Back Navigation)

**Files:**
- Modify: `src/wizard/mod.rs`

**Reference:** JS `cli.js` `main()` — full wizard flow with all 4 steps, enhanced with back navigation and vertical step progress.

**Step 1: Write wizard state machine**

The wizard is a state machine where each step can advance (Enter/→) or go back (←). Config snapshots are taken before each step for rollback.

```rust
use crate::config::Config;
use super::step_progress::StepInfo;

pub fn run() {
    preflight();
    let mut config = crate::config::load_config();

    // Language selection (no back from here)
    if config.lang.is_none() { /* select en/zh */ }
    crate::i18n::set_lang(config.lang.as_deref().unwrap_or("en"));

    // Mode selection (no back from here)
    // If defaults: save and apply, return
    // If custom: enter step loop

    // Define major steps for the progress bar
    let mut steps: Vec<StepInfo> = vec![
        StepInfo { label: t("step.segments").into(), summary: None },
        StepInfo { label: t("step.configSegment").into(), summary: None },
        StepInfo { label: t("step.reorder").into(), summary: None },
        StepInfo { label: t("step.confirm").into(), summary: None },
    ];

    // Step state machine
    let mut current_step: usize = 0;
    let mut snapshots: Vec<Config> = Vec::new();  // config before each step

    loop {
        // Save snapshot before executing step
        if snapshots.len() <= current_step {
            snapshots.push(config.clone());
        }

        // Render: clear screen → header → preview → step progress → current step UI
        show_screen(&config, &steps, current_step);

        let result = match current_step {
            0 => step_select_segments(&mut config, &mut steps[0]),
            1 => step_configure_segments(&mut config, &mut steps[1]),
            2 => step_reorder(&mut config, &mut steps[2]),
            3 => step_confirm(&config),
            _ => break,
        };

        match result {
            StepResult::Next => {
                current_step += 1;
                if current_step >= 4 {
                    // Step 4 (confirm) said yes → save and apply
                    install::save_and_apply(&config);
                    break;
                }
            }
            StepResult::Back => {
                if current_step > 0 {
                    current_step -= 1;
                    config = snapshots[current_step].clone();
                    snapshots.truncate(current_step);
                    steps[current_step].summary = None; // clear old summary
                }
            }
            StepResult::Cancelled => {
                // Esc / Ctrl+C
                eprintln!("{}", t("msg.cancelled"));
                break;
            }
        }
    }
}

enum StepResult {
    Next,
    Back,
    Cancelled,
}
```

**Step 2: Implement Step 1 — Select segments**

Multiselect of all 7 segments with live preview. Returns `StepResult`.

**Step 3: Implement Step 2 — Configure each segment**

This step has internal sub-steps (one per enabled segment, each with multiple prompts). Back navigation within this step goes to the previous sub-step first; only when at the first sub-step does ← bubble up to the previous major step.

```rust
fn step_configure_segments(config: &mut Config, step_info: &mut StepInfo) -> StepResult {
    let enabled: Vec<String> = config.order.iter()
        .filter(|k| segment_enabled(config, k))
        .cloned().collect();

    let mut sub_idx: usize = 0;
    let mut sub_snapshots: Vec<Config> = Vec::new();

    loop {
        if sub_idx >= enabled.len() {
            step_info.summary = Some(/* summary of configured segments */);
            return StepResult::Next;
        }

        if sub_snapshots.len() <= sub_idx {
            sub_snapshots.push(config.clone());
        }

        let key = &enabled[sub_idx];
        let result = configure_segment(config, key, sub_idx, enabled.len());

        match result {
            StepResult::Next => sub_idx += 1,
            StepResult::Back => {
                if sub_idx > 0 {
                    sub_idx -= 1;
                    *config = sub_snapshots[sub_idx].clone();
                    sub_snapshots.truncate(sub_idx);
                } else {
                    return StepResult::Back; // bubble up to previous major step
                }
            }
            StepResult::Cancelled => return StepResult::Cancelled,
        }
    }
}
```

For each segment type, show appropriate prompts:
- model: style + icon
- cost: style
- usage: parts multiselect + style + barChar + barLength + refresh
- path: style + maxLength
- git: parts multiselect + style
- context: parts multiselect + style + barChar + barLength
- crypto: coins multiselect + style + refresh

Every select/multiselect updates preview in real-time.

**Step 4: Implement Step 3 — Reorder**

If >1 segment enabled, show current order and offer to reorder. ← goes back to Step 2.

**Step 5: Implement Step 4 — Confirm & save**

Show final preview with vertical progress (all steps ●), confirm prompt. ← goes back to Step 3.

**Step 6: Screen layout**

Each screen render:
```
  Claude Statusline Configurator
  ──────────────────────────────────────────
  Preview: 🔥 Opus4.6 $0.42 ▓▓▓▓▓▓░░░░ 60%
  ──────────────────────────────────────────

  ● 1/4 Select segments
  │   Segments: Model, Cost, Context
  │
  ◆ 2/4 Configure (1/5)               ← vertical step progress
  │
  ○ 3/4 Reorder
  │
  ○ 4/4 Confirm

  ┌ Context — Style                    ← current prompt
  │  ● Ultrathink Gradient
  │  ○ Traffic light
  │  ○ Cyan
  └
  (← back  ↑↓ select  Enter confirm)  ← key hints
```

**Step 7: Verify full wizard flow**

Run: `cargo run`
Expected: Full interactive wizard with vertical step progress, live preview, and ← back navigation at every step.

**Step 8: Commit**

```bash
git add src/wizard/mod.rs
git commit -m "feat: wizard state machine with step progress and back navigation"
```

---

### Task 14: Install Module

**Files:**
- Create: `src/install.rs`

**Step 1: Write save_and_apply**

```rust
use std::fs;
use std::path::PathBuf;

pub fn save_and_apply(config: &crate::config::Config) -> Result<(), String> {
    // 1. Save config
    crate::config::save_config(config).map_err(|e| e.to_string())?;

    // 2. Copy current binary to ~/.claude/statusline/bin
    let self_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let target = crate::config::bin_path();
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    // Only copy if paths differ
    if self_path != target {
        fs::copy(&self_path, &target).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(0o755))
                .map_err(|e| e.to_string())?;
        }
    }

    // 3. Update ~/.claude/settings.json
    update_settings()?;

    Ok(())
}

fn update_settings() -> Result<(), String> {
    let settings_path = dirs::home_dir().unwrap().join(".claude").join("settings.json");
    let mut settings: serde_json::Value = if settings_path.exists() {
        let raw = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&raw).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let command = "~/.claude/statusline/bin --render";
    settings["statusLine"] = serde_json::json!({
        "type": "command",
        "command": command,
        "padding": 0
    });

    let json = serde_json::to_string_pretty(&settings).unwrap();
    fs::write(&settings_path, json + "\n").map_err(|e| e.to_string())?;
    Ok(())
}
```

**Step 2: Wire into wizard — call from save step**

In `wizard/mod.rs`, the Step 4 confirm handler calls `install::save_and_apply(&config)`.

**Step 3: Verify end-to-end**

Run: `cargo run`, go through wizard, confirm save.
Check: `~/.claude/statusline/bin` exists, `~/.claude/statusline/config.json` exists, `~/.claude/settings.json` has correct statusLine.

**Step 4: Commit**

```bash
git add src/install.rs src/wizard/mod.rs
git commit -m "feat: install module — binary self-copy + settings.json update"
```

---

### Task 15: npm Distribution Shell

**Files:**
- Create: `npm/package.json`
- Create: `npm/postinstall.js`
- Create: `npm/cli.js`

**Step 1: Write package.json**

```json
{
  "name": "claude-statusline-config",
  "version": "2.0.0",
  "description": "Interactive CLI tool to configure Claude Code statusline",
  "bin": { "claude-statusline-config": "./cli.js" },
  "scripts": { "postinstall": "node postinstall.js" },
  "license": "MIT",
  "repository": "https://github.com/LokiQ0713/claude-statusline-config",
  "os": ["darwin", "linux"],
  "cpu": ["arm64", "x64"]
}
```

**Step 2: Write postinstall.js**

Detects `process.platform` + `process.arch`, maps to GitHub Release asset name, downloads via `https.get` (no deps), extracts tar.gz, copies to `~/.claude/statusline/bin`, chmod +x.

**Step 3: Write cli.js**

```js
#!/usr/bin/env node
const { execFileSync } = require('child_process');
const { join } = require('path');
const { homedir } = require('os');
const bin = join(homedir(), '.claude', 'statusline', 'bin');
try {
  execFileSync(bin, process.argv.slice(2), { stdio: 'inherit' });
} catch (e) {
  console.error('Binary not found. Run: npx claude-statusline-config');
  process.exit(1);
}
```

**Step 4: Commit**

```bash
git add npm/
git commit -m "feat: npm distribution shell (postinstall + cli.js)"
```

---

### Task 16: CI/CD Workflows

**Files:**
- Create: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release.yml`

**Step 1: Write ci.yml**

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check
      - run: cargo test
      - run: cargo clippy -- -D warnings
```

**Step 2: Write release.yml**

```yaml
name: Release
on:
  push:
    tags: ['v*']
permissions:
  contents: write
jobs:
  build:
    strategy:
      matrix:
        include:
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
          - target: aarch64-unknown-linux-musl
            os: ubuntu-latest
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install cross-compilation tools (Linux ARM)
        if: matrix.target == 'aarch64-unknown-linux-musl'
        run: sudo apt-get install -y gcc-aarch64-linux-gnu
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      - name: Package
        run: |
          cd target/${{ matrix.target }}/release
          tar czf ../../../claude-statusline-config-${{ matrix.target }}.tar.gz claude-statusline-config
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.target }}
          path: claude-statusline-config-${{ matrix.target }}.tar.gz

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
      - name: Create GitHub Release
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          gh release create "$GITHUB_REF_NAME" \
            --title "$GITHUB_REF_NAME" \
            --notes "Release ${GITHUB_REF_NAME#v}" \
            --latest \
            */*.tar.gz

  publish-npm:
    needs: release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          registry-url: https://registry.npmjs.org
      - run: cd npm && npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

**Step 3: Commit**

```bash
git add .github/workflows/
git commit -m "feat: CI + cross-platform release workflow"
```

---

### Task 17: Final Integration Test + Cleanup

**Files:**
- Modify: `CLAUDE.md` (update for Rust version)
- Modify: `README.md` + `README.zh.md` (update prerequisites — no more jq/perl/curl)

**Step 1: Full end-to-end test**

```bash
# Build release binary
cargo build --release

# Test render mode
echo '{"model":{"id":"claude-opus-4-6"},"cost":{"total_cost_usd":0.42},"context_window":{"context_window_size":1000000,"used_percentage":60}}' | ./target/release/claude-statusline-config --render

# Test wizard mode
./target/release/claude-statusline-config

# Verify install
cat ~/.claude/statusline/config.json
cat ~/.claude/settings.json | grep statusline
ls -la ~/.claude/statusline/bin
```

**Step 2: Update docs**

- `README.md` / `README.zh.md`: remove jq/perl/curl from prerequisites, note "Node.js >= 18" is only needed for npm install, not for runtime
- `CLAUDE.md`: update tech stack, file structure, dependencies section

**Step 3: Commit**

```bash
git add CLAUDE.md README.md README.zh.md
git commit -m "docs: update for Rust rewrite — remove system deps requirement"
```

---

## Task Dependency Graph

```
Task 1 (scaffold)
  ├── Task 2 (config)
  │     ├── Task 3 (i18n)
  │     └── Task 4 (styles)
  │           ├── Task 5 (render: parsing)
  │           │     └── Task 6 (render: output)
  │           │           └── Task 7 (cache + HTTP)
  │           │                 └── Task 8 (logging)
  │           └── Task 12 (preview + step progress)
  │
  ├── Task 9 (terminal layer)
  │     ├── Task 10 (select — with Back result)
  │     └── Task 11 (multiselect, confirm, spinner — with Back result)
  │           └── Task 13 (wizard state machine) ← depends on 12 too
  │                 └── Task 14 (install)
  │
  ├── Task 15 (npm shell) — independent
  ├── Task 16 (CI/CD) — independent
  └── Task 17 (integration test) ← depends on all above
```

Parallelizable pairs:
- Tasks 3 + 4 (i18n + styles)
- Tasks 9 + 5 (terminal + render parsing)
- Tasks 15 + 16 (npm + CI/CD)

Key enhancement over JS version:
- Vertical step progress bar (● ◆ ○ with │ connectors)
- ← back navigation at every step (config snapshots for rollback)
- Sub-step back within Step 2 (configure) before bubbling to previous major step
