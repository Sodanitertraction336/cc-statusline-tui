# claude-statusline-config

> Your Claude Code statusline is boring. Let's fix that.

[中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português](README.pt.md) | [Русский](README.ru.md)

![statusline preview](preview.png)

One command. Seven languages. Zero regrets.

## Install

Pick your poison:

```bash
# npm — the classic
npx claude-statusline-config

# Homebrew — for the refined palate
brew tap LokiQ0713/claude-statusline-config && brew install claude-statusline-config

# Cargo — for the Rustaceans
cargo install claude-statusline-config
```

## What Happens

A TUI wizard pops up. You pick stuff. It looks cool. That's it.

```
1/4 Segments      → What to show
2/4 Configure     → How it looks (live preview, instant feedback)
3/4 Reorder       → Where things go
4/4 Confirm       → Ship it
```

Too lazy to customize? Pick "Use defaults" and press Enter twice. We don't judge.

## Segments

| Segment | Looks Like | What It Does | You Can Tweak |
|---------|-----------|--------------|---------------|
| Model | `🔥 Opus4.6` | Shows which brain you're burning through | Icon (🔥🦊🐢🐰), color |
| Cost | `$0.42` | Your wallet's pain level | Color |
| Usage | `██░░ 25% 1h43m` | 5h rate limit: bar + % + reset countdown | Bar style/char/length, color, refresh |
| Path | `~/project` | Where you are | Color, max length |
| Git | `main* ↑2↓1` | Branch + dirty flag + ahead/behind | Components, color |
| Context | `▓▓▓▓░░░ 60% 600K/1M` | How much context window you've eaten | Bar style/char/length, color |
| Crypto | `BTC:$73748` | Because why not check prices while coding | Coins (BTC/ETH/BNB/SOL), color, refresh |

## Color Styles

| Style | Vibe |
|-------|------|
| **Ultrathink Rainbow** | Every character is a different color. Yes, it shimmers. |
| **Ultrathink Gradient** | Smooth rainbow across the bar. Chef's kiss. |
| **Traffic Light** | Green (≤30%) → Yellow (≤60%) → Red (>60%). Your context window is a traffic jam. |
| Cyan / Green / Blue / Yellow | The sensible choices |
| Magenta / Red / Orange / Pink / Purple / White | The expressive choices |

## Languages

English, 中文, 日本語, 한국어, Español, Português, Русский.

Selected on first run. Your statusline speaks your language.

## How It Works

1. Wizard saves config to `~/.claude/statusline/config.json`
2. Binary goes to `~/.claude/statusline/bin/`
3. `~/.claude/settings.json` gets updated automatically
4. Restart Claude Code. Admire your new statusline. Tell your coworkers.

Re-running the wizard loads your existing config as defaults. Non-destructive. Promise.

## Requirements

- Claude Code installed (`~/.claude/` exists)
- Node.js ≥ 18 (only for `npx` install — not needed at runtime)

## Contributing

Found a bug? Want a feature? [Open an issue](https://github.com/LokiQ0713/claude-statusline-config/issues). PRs welcome.

## License

MIT
