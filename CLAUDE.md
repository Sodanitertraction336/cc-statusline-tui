# cc-statusline-tui

Interactive CLI tool to configure the Claude Code statusline.

## Project Info

- Package: `cc-statusline-tui`
- GitHub: `https://github.com/LokiQ0713/cc-statusline-tui`
- Registry: npm public registry (ships prebuilt Rust binaries via npm postinstall) + crates.io
- Install: `npx cc-statusline-tui` or `cargo install cc-statusline-tui`

## Tech Stack

- Language: Rust (2021 edition)
- Entry: `src/main.rs`
- Dependencies: `serde` / `serde_json` (config serialization), `crossterm` (terminal UI), `ureq` (HTTP), `dirs` (home directory), `chrono` (time parsing), `tempfile` (tests)
- System deps: none (was jq/perl/curl in the JS version)

## File Structure

- `src/main.rs` — Entry point, dispatches `--render` vs wizard
- `src/config.rs` — Config structs, load/save, path helpers
- `src/i18n.rs` — i18n (en/zh/ja/ko/es/pt/ru), static + template translations
- `src/styles.rs` — ANSI color codes, rainbow/gradient rendering, bar formatting
- `src/render.rs` — Render pipeline: reads stdin JSON, outputs formatted statusline
- `src/cache.rs` — Stale-while-revalidate cache for crypto prices and usage data
- `src/log.rs` — Error logging to `~/.claude/statusline/statusline.log`
- `src/install.rs` — Save config, copy binary, update settings.json
- `src/wizard/` — Interactive TUI wizard
  - `mod.rs` — Wizard state machine (4-step flow)
  - `terminal.rs` — Terminal control (raw mode, cursor, key reading)
  - `select.rs` — Single-select component
  - `multiselect.rs` — Multi-select component
  - `confirm.rs` — Yes/No confirmation
  - `spinner.rs` — Loading spinner
  - `preview.rs` — Live preview rendering
  - `step_progress.rs` — Step progress indicator

## npm Distribution (esbuild-style platform packages)

- `package.json` — Main npm package (thin wrapper with optionalDependencies)
- `cli.js` — Resolves platform binary from node_modules, executes it
- `npm/` — Platform package templates (binary added by CI)
  - `darwin-arm64/package.json` — macOS ARM64
  - `darwin-x64/package.json` — macOS x64
  - `linux-x64/package.json` — Linux x64
  - `linux-arm64/package.json` — Linux ARM64
- npm auto-installs only the matching platform package (via `os`/`cpu` fields)
- No postinstall scripts, no runtime downloads

## Key Directories

- `~/.claude/statusline/` — Runtime directory
  - `config.json` — User configuration
  - `bin` — Compiled binary (copied during install)
  - `statusline.log` — Error log
- `/tmp/claude-statusline-*` — Cache files (crypto prices, usage data)

## Development

```bash
cargo run               # Run wizard
cargo run -- --render   # Test render pipeline (reads JSON from stdin)
cargo test              # Run all tests
cargo clippy -- -D warnings  # Lint check
```

## Versioning

Semantic versioning: MAJOR.MINOR.PATCH

- PATCH: bug fixes, copy/style tweaks
- MINOR: new segment types, new config options
- MAJOR: config format changes, binary interface changes

## Release Workflow

### Steps

```bash
# 1. Verify locally
cargo test && cargo clippy -- -D warnings

# 2. Bump version in Cargo.toml, then create git commit + tag
npm version patch      # or minor / major

# 3. Push code and tag → triggers GitHub Actions auto-publish
git push origin main --tags
```

### How it works

1. `npm version` bumps `package.json`, creates a commit and `v*` tag
2. `git push --tags` pushes the tag to GitHub
3. GitHub Actions `release.yml` triggers on `v*` tags
4. Workflow runs `npm publish` using `NPM_TOKEN` secret
5. Workflow creates a GitHub Release via `gh release create`

### Verify publish succeeded

```bash
# Check GitHub Actions status
gh run list --repo LokiQ0713/cc-statusline-tui --workflow=release.yml --limit 3

# Check npm registry
npm view cc-statusline-tui version

# Check crates.io
cargo search cc-statusline-tui

# Check GitHub Release was created
gh release list --repo LokiQ0713/cc-statusline-tui
```

### Troubleshooting publish failures

| Error | Cause | Fix |
|-------|-------|-----|
| `ENEEDAUTH` | `NPM_TOKEN` secret missing or invalid | Add/update token in GitHub repo Settings → Secrets → Actions |
| `E403 previously published` | Version already exists on npm | Bump version again with `npm version patch` |
| GitHub Release not created | `contents: write` permission missing | Check `release.yml` has `permissions: contents: write` |
| crates.io publish failed | `CARGO_REGISTRY_TOKEN` missing or invalid | Add/update token in GitHub repo Settings → Secrets → Actions |

## CI/CD

### ci.yml — 质量门禁

- 触发: push/PR to main
- Runner: ubuntu-latest
- 步骤: `cargo check` → `cargo test` → `cargo clippy -- -D warnings`

### release.yml — 三渠道并行发布

- 触发: push tag `v*`

**Job 1: `build`（4 并行矩阵）**

| Rust target | Runner | npm 包 |
|-------------|--------|--------|
| aarch64-apple-darwin | macos-latest | darwin-arm64 |
| x86_64-apple-darwin | macos-latest | darwin-x64 |
| x86_64-unknown-linux-musl | ubuntu-latest | linux-x64 |
| aarch64-unknown-linux-musl | ubuntu-latest | linux-arm64 |

- Linux ARM 交叉编译需 `gcc-aarch64-linux-gnu` + `.cargo/config.toml` linker 配置
- 产出: `binary-*` artifact (tar.gz) + `npm-*` artifact (平台包目录)
- npm 包版本从 tag 自动提取 (`${GITHUB_REF_NAME#v}`)

**Job 2: `release`** — 下载 binary artifacts → `gh release create` 附带 4 个 tar.gz

**Job 3: `publish-npm`** — 下载 npm artifacts → `chmod +x npm/*/bin/*` → 先发 4 个平台包 → 再发主包

**Job 4: `publish-crate`** — `cargo publish`

### Required Secrets

| Secret | 用途 |
|--------|------|
| `NPM_TOKEN` | npm 发布（4 平台包 + 1 主包） |
| `CARGO_REGISTRY_TOKEN` | crates.io 发布 |
| `GITHUB_TOKEN` (自动) | GitHub Release |

### npm 分发架构（Biome 模式）

平台包声明 `"bin"` 字段 → npm 安装时自动 chmod +x，无需 postinstall hack。
cli.js 有 `fs.chmodSync` 自愈兜底 + EACCES 错误诊断。

### 已知坑

- `upload-artifact`/`download-artifact` 会丢失文件权限 → publish-npm 步骤有显式 `chmod +x`
- `npm version` 只改 package.json，需手动同步 Cargo.toml 版本
- Homebrew tap 需手动更新 SHA256（无自动化）

## Error Handling Convention

All user-facing errors must include an AI analysis hint:

```
Tip: Copy this error to AI for analysis
See https://github.com/LokiQ0713/cc-statusline-tui#troubleshooting
```

## Key Internals

- Binary output: `~/.claude/statusline/bin` (compiled Rust binary)
- Config file: `~/.claude/statusline/config.json`
- Auto-updates: `statusLine` field in `~/.claude/settings.json`
