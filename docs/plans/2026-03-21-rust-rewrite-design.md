# Rust Rewrite Design

## Overview

Rewrite `claude-statusline-config` from Node.js to Rust. Single binary handles both the interactive TUI wizard and the statusline renderer. Zero system dependencies in compiled output.

## Decisions

| Decision | Choice |
|----------|--------|
| Scope | Full rewrite: CLI wizard + statusline renderer |
| Architecture | Single crate, single binary. `--render` flag switches mode |
| Dependencies | Unrestricted crates, statically linked binary |
| Distribution | npm only (postinstall downloads prebuilt binary) |
| File layout | All in `~/.claude/statusline/` (bin, config.json, statusline.log) |
| Fallback | None. No .sh script generation |
| Platform targets | macOS arm64/x64, Linux x64/arm64 (musl) |

## Project Structure

```
claude-statusline-config/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry: parse args, route to wizard or render
│   ├── config.rs            # Config struct, JSON read/write, defaults
│   ├── i18n.rs              # en/zh translations
│   ├── styles.rs            # Color defs, Ultrathink rainbow data, bar rendering
│   ├── render.rs            # --render mode: stdin JSON → ANSI output
│   ├── install.rs           # Copy binary to ~/.claude/statusline/bin, update settings.json
│   └── wizard/
│       ├── mod.rs           # Wizard main flow (4 steps)
│       ├── terminal.rs      # Raw mode, keyboard input, cursor control
│       ├── select.rs        # Single-select with live preview
│       ├── multiselect.rs   # Multi-select with live preview
│       ├── confirm.rs       # y/n confirmation
│       ├── spinner.rs       # Save spinner animation
│       └── preview.rs       # Preview bar rendering (ANSI cursor positioning)
├── npm/
│   ├── package.json         # npm distribution shell
│   ├── postinstall.js       # Download prebuilt binary to ~/.claude/statusline/bin
│   └── cli.js               # npx entry, forwards to ~/.claude/statusline/bin
├── .github/
│   └── workflows/
│       ├── ci.yml           # PR/push checks
│       └── release.yml      # v* tag → cross-compile + npm publish
```

## Dependencies

| Purpose | Crate | Reason |
|---------|-------|--------|
| JSON | `serde` + `serde_json` | Stdin input and config file are both JSON |
| Terminal | `crossterm` | Raw mode, key events, cursor positioning, colors |
| HTTP | `ureq` | Sync, lightweight, pure Rust TLS (no OpenSSL) |
| Home dir | `dirs` | Cross-platform `~/` resolution |

## Config Format

New format. Not compatible with JS version. Users re-run wizard after upgrade.

```json
{
  "lang": "zh",
  "order": ["model", "cost", "usage", "path", "git", "context", "crypto"],
  "segments": {
    "model":   { "enabled": true, "style": "ultrathink", "icon": "🔥" },
    "cost":    { "enabled": true, "style": "green" },
    "usage":   {
      "enabled": true,
      "style": "semantic",
      "barChar": "shade",
      "barLength": 8,
      "showBar": false,
      "showPercent": true,
      "showReset": true,
      "refreshInterval": 120
    },
    "path":    { "enabled": true, "style": "cyan", "maxLength": 15 },
    "git":     { "enabled": true, "style": "cyan", "showDirty": true, "showRemote": true },
    "context": {
      "enabled": true,
      "style": "ultrathink-gradient",
      "barChar": "shade",
      "barLength": 12,
      "showBar": true,
      "showPercent": true,
      "showSize": true
    },
    "crypto":  { "enabled": true, "style": "green", "refreshInterval": 60, "coins": ["BTC"] }
  }
}
```

Key changes from JS version:
- `barStyle` + `textStyle` merged into single `style` (per module, one color for everything)
- Default `barChar` is `shade` (was `full-block` or `rectangle`)
- `full-block` uses `█` for both filled and empty (equal height, color differentiation)

## Render Mode (`--render`)

Called by Claude Code via `settings.json`:
```json
{ "statusLine": { "type": "command", "command": "~/.claude/statusline/bin --render", "padding": 0 } }
```

Data flow:
1. Read stdin JSON from Claude Code (model, cost, context window, cwd)
2. Read `~/.claude/statusline/config.json`
3. Render each segment in configured order
4. Output ANSI string to stdout

Segment data sources:
- model, cost, path, context: from stdin JSON
- git: local `git` command execution
- usage: HTTP to Anthropic API (file-cached, background refresh)
- crypto: HTTP to Binance API (file-cached, background refresh)

Cache files: `/tmp/claude-statusline-{usage,crypto}-cache`
Cache strategy: read stale cache immediately, spawn background thread to refresh if expired.

Performance target: < 5ms per render (excluding background HTTP).

## Bar Rendering

Three bar styles:
1. **ultrathink-gradient**: 7-color rainbow smooth interpolation, per-character `\x1b[38;2;R;G;Bm`
2. **semantic (traffic light)**: green < 50%, yellow 50-75%, red > 75%
3. **solid color**: single color fill

Three bar characters:
| Name | Filled | Empty | Visual |
|------|--------|-------|--------|
| shade (default) | `▓` | `░` | `▓▓▓▓▓▓░░░░` |
| full-block | `█` | `█` (dim) | `██████████` (color diff) |
| rectangle | `▬` | `▬` (dim) | `▬▬▬▬▬▬▬▬▬▬` (color diff) |

Shimmer animation: `timestamp % 2` toggles main/shimmer color arrays, `timestamp % 7` controls offset.

## TUI Wizard

Replicates @clack/prompts style interaction, enhanced with vertical step progress and back navigation.

### Components
- `select`: up/down to move highlight, Enter to confirm, live preview update
- `multiselect`: up/down + Space to toggle, Enter to confirm, live preview update
- `confirm`: y/n
- `spinner`: braille animation during save

### Key Bindings
- `↑` `↓`: move between options within current step
- `Enter` / `→`: confirm current step, advance to next
- `←`: go back to previous step (restore config snapshot)
- `Space`: toggle item (multiselect only)
- `Esc` / `Ctrl+C`: cancel and exit

### Vertical Step Progress Bar

Left side shows a vertical progress indicator connecting all steps:

```
  ● 1/4 Select segments        ← completed (filled dot)
  │   Segments: Model, Cost, Context
  │
  ◆ 2/4 Configure (1/5)        ← current step (diamond, highlighted)
  │   ┌ Context — Style
  │   │  ● Ultrathink Gradient
  │   │  ○ Traffic light
  │   │  ○ Cyan
  │   └
  │
  ○ 3/4 Reorder                ← pending (empty dot)
  │
  ○ 4/4 Confirm
```

Symbols:
- `●` completed step (with summary text below)
- `◆` current active step
- `○` pending step
- `│` vertical connector line

### Back Navigation

Each step stores a config snapshot before executing. Left arrow restores the snapshot and re-renders the previous step.

```rust
struct WizardState {
    steps: Vec<Step>,        // all steps
    current: usize,          // current step index
    snapshots: Vec<Config>,  // config clone before each step
}

// ← left arrow:
if current > 0 {
    config = snapshots[current - 1].clone();
    current -= 1;
    // re-render previous step
}
```

Steps that produce sub-steps (e.g., "Configure" has one sub-step per segment) track their internal position too, so back goes to the previous sub-step before going to the previous major step.

### Other Behavior
- **No wrap-around**: hitting top/bottom boundary stops, does not cycle
- **Live preview**: every arrow/space key press updates the preview bar in-place via ANSI cursor positioning

### Wizard Flow
Language → Default/Custom → 1/4 Select segments → 2/4 Configure each → 3/4 Reorder → 4/4 Confirm & save

## Error Logging

- Render mode: errors written to `~/.claude/statusline/statusline.log` with timestamps
- Wizard mode: errors shown directly via stderr
- Log file capped at 100KB, truncates old entries when exceeded
- Format: `[2026-03-21 14:30:05] ERROR: message`

## npm Distribution

npm package is a thin shell:
- `postinstall.js`: detects platform/arch, downloads binary from GitHub Releases to `~/.claude/statusline/bin`
- `cli.js`: forwards all args to `~/.claude/statusline/bin`

File layout after install:
```
~/.claude/statusline/
├── bin                 # Rust binary
├── config.json         # User configuration
└── statusline.log      # Error log (render mode only)
```

## CI/CD

### ci.yml
- Trigger: push/PR to main
- Matrix: `cargo check`, `cargo test`, `cargo clippy`

### release.yml
- Trigger: `v*` tag push
- Matrix compile 4 targets:
  - `aarch64-apple-darwin` (macOS arm64)
  - `x86_64-apple-darwin` (macOS x64)
  - `x86_64-unknown-linux-musl` (Linux x64, static)
  - `aarch64-unknown-linux-musl` (Linux arm64, static)
- Upload `.tar.gz` to GitHub Release
- npm publish
