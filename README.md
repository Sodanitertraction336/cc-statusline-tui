# claude-statusline-config

> Spice up your Claude Code statusline.

[中文文档](README.zh.md)

![statusline preview](preview.png)

## Quick Start

```bash
npx claude-statusline-config
```

## Two Modes

**First time?** Pick "Use defaults", press Enter twice.

**Want to customize?** The wizard walks you through step by step:

```
1/4 Segments      → Pick which segments to show
2/4 Configure     → Style, sub-components, parameters (live preview)
3/4 Reorder       → Arrange display order
4/4 Confirm       → Save and apply
```

The preview bar stays visible and updates in real-time as you make changes.

## Segments

| Segment | Example | Description | Config |
|---------|---------|-------------|--------|
| Model | `🔥 Opus4.6` | Current model | Icon (🔥🧠🦊🤖), color |
| Cost | `$0.42` | Session cost | Color |
| Usage | `██░░ 25% 1h43m` | 5h limit: bar + percentage + reset timer | Sub-components, bar style/char/length, text color, refresh |
| Path | `~/Desktop/web3` | Current directory | Color, max length |
| Git | `main* ↑2↓1` | Branch + dirty + remote diff | Sub-components (dirty, remote), color |
| Context | `▬▬▬▬▬▬░░░░ 60% 600K/1M` | Context window usage: bar + percentage + capacity | Sub-components, bar style/char/length, text color |
| Crypto | `BTC:$73748` | Live prices | Coins (BTC/ETH/BNB/SOL, max 3), color, refresh |

## Colors

| Style | Description |
|-------|-------------|
| Ultrathink (Rainbow) | Each character in a different color, with shimmer animation |
| Cyan / Green / Blue / Yellow / Magenta / Red / White | Solid colors |
| Orange / Pink / Purple | Extended solid colors |
| Ultrathink Gradient | Bar only — smooth rainbow gradient |
| Traffic light | Bar only — green → yellow → red based on usage |

## Language

Supports English and Chinese. Language is selected on first run and saved to config.

## How It Works

1. Wizard → saves to `~/.claude/statusline.config.json`
2. Generates shell script → `~/.claude/scripts/statusline.sh`
3. Updates `~/.claude/settings.json`
4. Restart Claude Code to apply

Only touches config files. Re-running reads your saved config as defaults.

## Prerequisites

- Node.js >= 18
- Claude Code installed (`~/.claude/` exists)
- `jq`, `perl`, `curl` (macOS built-in, Linux needs install)

## Contributing

[Open an issue](https://github.com/LokiQ0713/claude-statusline-config/issues) or [submit a PR](https://github.com/LokiQ0713/claude-statusline-config/pulls)
