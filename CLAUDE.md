# claude-statusline-config

交互式 CLI 工具，配置 Claude Code 状态栏。

## 项目信息

- 包名: `claude-statusline-config`
- GitHub: `https://github.com/LokiQ0713/claude-statusline-config`
- Registry: npm 公共 registry
- 用户安装: `npx claude-statusline-config`

## 技术栈

- 语言: JavaScript（ESM, `"type": "module"`）
- 入口: `cli.js`
- 依赖: `@clack/prompts`（交互式菜单）, `chalk`（终端着色）
- 系统依赖: `jq`, `perl`, `curl`（生成的 statusline.sh 脚本使用）

## 文件结构

- `cli.js` — 主入口，菜单逻辑和导航
- `config.js` — 配置加载/保存/默认值
- `styles.js` — 样式定义和预览函数
- `generator.js` — 生成 statusline.sh 脚本
- `i18n.js` — 国际化（中文/英文）

## 开发流程

```bash
# 开发（无需编译）
node cli.js            # 本地测试

# 发布（GitHub Actions 自动发布）
npm version patch      # 或 minor / major — 自动改 package.json + git commit + tag
git push origin main --tags  # 推送代码和 tag → 触发 GitHub Actions 自动发布到 npm
```

## 版本管理规范

- 使用语义化版本: MAJOR.MINOR.PATCH
  - PATCH: bug 修复、文案调整、样式调整
  - MINOR: 新增段落类型、新增配置选项
  - MAJOR: 配置格式变更、生成脚本格式变更
- `npm version` 自动创建 git tag（v1.0.1 等）
- Push tag 触发 GitHub Actions release workflow 自动发布到 npm

## 发布检查清单

1. `node cli.js` — 本地验证可运行
2. `npm version patch/minor/major` — 递增版本
3. `git push origin main --tags` — 推送代码和 tag
4. 观察 GitHub Actions 运行状态确认发布成功

## CI/CD

- `.github/workflows/ci.yml` — push/PR 触发，Node 18/20/22 矩阵，语法和模块加载检查
- `.github/workflows/release.yml` — `v*` tag 触发，自动发布到 npm + 创建 GitHub Release
- 所需 Secret: `NPM_TOKEN`（在 GitHub repo Settings → Secrets → Actions 添加）

## 错误处理规范

所有用户可见的错误信息必须附带 AI 分析提示：
```
Tip: Copy this error to AI for analysis
See https://github.com/LokiQ0713/claude-statusline-config#troubleshooting
```

## 关键逻辑备忘

- 生成产物: `~/.claude/scripts/statusline.sh`（shell 脚本）
- 配置文件: `~/.claude/statusline.config.json`
- 自动更新: `~/.claude/settings.json` 的 `statusLine` 字段
