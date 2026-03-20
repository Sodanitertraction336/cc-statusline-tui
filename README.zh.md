# claude-statusline-config

> Claude Code 状态栏太素了？给它整点花活。

[English](README.md)

![statusline preview](preview.png)

## 一行搞定

```bash
npx claude-statusline-config
```

## 两种模式

**首次安装？** 选"使用默认配置"，按两下回车直接生效。

**想自定义？** 向导式流程带你一步步走完：

```
1/4 选择段落      → 勾选要显示哪些信息
2/4 配置段落      → 逐个选样式、子组件、参数（实时预览）
3/4 排列顺序      → 调整显示位置
4/4 确认          → 保存并应用
```

预览栏始终可见，切换选项时实时更新效果。

## 能配什么

| 段落 | 长啥样 | 干啥的 | 可配项 |
|------|--------|--------|--------|
| 模型 | `🔥 Opus4.6` | 当前模型 | 图标（🔥🧠🦊🤖）、颜色 |
| 费用 | `$0.42` | 会话费用 | 颜色 |
| 5h限额 | `██░░ 25% 1h43m` | 限额进度条 + 百分比 + 重置倒计时 | 子组件开关、进度条样式/字符/长度、文字颜色、刷新间隔 |
| 目录 | `~/Desktop/web3` | 当前目录 | 颜色、最大长度 |
| Git状态 | `main* ↑2↓1` | 分支名 + 脏状态 + 远程差异 | 子组件开关（脏状态、远程差异）、颜色 |
| 上下文 | `▬▬▬▬▬▬░░░░ 60% 600K/1M` | 上下文窗口用量：进度条 + 百分比 + 容量 | 子组件开关、进度条样式/字符/长度、文字颜色 |
| 加密货币 | `BTC:$73748` | 实时币价 | 币种（BTC/ETH/BNB/SOL，最多3个）、颜色、刷新间隔 |

## 颜色风格

| 风格 | 说明 |
|------|------|
| Ultrathink (彩虹) | 每个字符不同颜色，带闪烁动画 |
| Cyan / Green / Blue / Yellow / Magenta / Red / White | 纯色 |
| Orange / Pink / Purple | 扩展纯色 |
| Ultrathink 渐变 | 进度条专用，平滑彩虹渐变 |
| 红绿灯 | 进度条专用，绿→黄→红 随用量变化 |

## 多语言

支持英文和中文。首次运行时选择语言，保存到配置中。

## 工作原理

1. 向导式选配 → 保存到 `~/.claude/statusline/config.json`
2. 复制二进制文件到 `~/.claude/statusline/bin`
3. 自动更新 `~/.claude/settings.json`
4. 重启 Claude Code 生效

不碰你的代码，只动配置文件。再次运行会读取已有配置作为默认值。

## 前提

- Node.js >= 18（仅 `npx` 安装时需要，运行时不需要）
- 装过 Claude Code（`~/.claude/` 存在）

## 贡献

[提 Issue](https://github.com/LokiQ0713/claude-statusline-config/issues) 或 [提 PR](https://github.com/LokiQ0713/claude-statusline-config/pulls)
