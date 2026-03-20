const messages = {
  en: {
    // Language selector (always bilingual)
    'lang.prompt': 'Language / 语言',

    // Steps
    'step.start': 'Start',
    'step.segments': '1/4 Segments',
    'step.configSegment': (cur, tot) => `2/4 Configure (${cur}/${tot})`,
    'step.reorder': '3/4 Reorder',
    'step.reorderN': (i, n) => `3/4 Reorder (${i}/${n})`,
    'step.confirm': '4/4 Confirm',

    // Mode selection
    'mode.prompt': 'Config mode',
    'mode.defaults': 'Use defaults',
    'mode.defaultsHint': 'Save directly, just press Enter',
    'mode.custom': 'Customize',
    'mode.customHint': 'Adjust styles and options step by step',

    // Segments
    'seg.model': 'Model',
    'seg.cost': 'Cost',
    'seg.usage': '5h Limit',
    'seg.path': 'Path',
    'seg.git': 'Git',
    'seg.context': 'Context',
    'seg.crypto': 'Crypto',

    // Sub-parts — shared
    'part.bar': 'Progress bar',
    'part.percent': 'Percentage',
    // Context parts
    'part.size': 'Capacity',
    // Usage parts
    'part.reset': 'Reset timer',
    // Git parts
    'part.dirty': 'Dirty marker',
    'part.remote': 'Remote diff',
    'part.branch': 'branch',

    // Prompts
    'prompt.selectSegments': 'Select segments to enable',
    'prompt.showParts': 'Show which parts',
    'prompt.barStyle': 'Bar style',
    'prompt.barChar': 'Bar character',
    'prompt.barLength': 'Bar length',
    'prompt.textStyle': 'Text style',
    'prompt.style': 'Style',
    'prompt.selectCoins': 'Select coins (max 3)',
    'prompt.refreshInterval': 'Refresh interval',
    'prompt.maxLength': 'Max length (shows dir name if exceeded)',
    'prompt.currentOrder': (order) => `Current order: ${order}, adjust?`,
    'prompt.pickN': (i) => `Position ${i}`,
    'prompt.saveDefaults': 'Save and apply defaults?',
    'prompt.save': 'Save and apply?',

    // Styles
    'style.rainbow': 'Ultrathink (Rainbow)',
    'style.cyan': 'Cyan',
    'style.green': 'Green',
    'style.blue': 'Blue',
    'style.yellow': 'Yellow',
    'style.magenta': 'Magenta',
    'style.red': 'Red',
    'style.white': 'White',
    'style.orange': 'Orange',
    'style.pink': 'Pink',
    'style.purple': 'Purple',

    // Bar styles
    'barStyle.gradient': 'Ultrathink Gradient (Rainbow)',
    'barStyle.trafficLight': 'Traffic light (G→Y→R)',

    // Bar chars
    'char.shade': '▓░ Shade',
    'char.fullBlock': '█░ Full block',
    'char.rectangle': '▬ Rectangle',

    // Units
    'unit.chars': 'chars',
    'unit.seconds': 's',

    // Messages
    'msg.preview': 'Preview:',
    'msg.cancelled': 'Cancelled, config not saved',
    'msg.cancelledShort': 'Cancelled',
    'msg.saving': 'Saving config...',
    'msg.saved': 'Config saved',
    'msg.saveFailed': 'Save failed',
    'msg.restart': 'Restart Claude Code or wait for next refresh',
    'msg.unknownError': 'Unknown error',
    'msg.missingDeps': (deps) => `Missing dependencies: ${deps}`,
    'msg.installDeps': (deps) => `Please install: brew install ${deps}`,
    'msg.noClaudeCode': 'Claude Code not detected (~/.claude does not exist)',
    'msg.installClaudeCode': 'Please install: https://claude.ai/code',

    // Done steps
    'done.segments': (names) => `Segments: ${names}`,
    'done.coins': (coins) => `Coins: ${coins}`,
    'done.refresh': (n) => `Refresh: ${n}s`,
    'done.length': (n) => `Length: ${n}`,
    'done.text': (s) => `Text: ${s}`,
    'done.order': (names) => `Order: ${names}`,
  },

  zh: {
    'lang.prompt': 'Language / 语言',

    'step.start': '开始',
    'step.segments': '1/4 选择段落',
    'step.configSegment': (cur, tot) => `2/4 配置段落 (${cur}/${tot})`,
    'step.reorder': '3/4 排列顺序',
    'step.reorderN': (i, n) => `3/4 排列顺序 (${i}/${n})`,
    'step.confirm': '4/4 确认',

    'mode.prompt': '配置模式',
    'mode.defaults': '使用默认配置',
    'mode.defaultsHint': '直接保存，按回车即可',
    'mode.custom': '自定义配置',
    'mode.customHint': '逐项调整样式和选项',

    'seg.model': '模型',
    'seg.cost': '费用',
    'seg.usage': '5h限额',
    'seg.path': '目录',
    'seg.git': 'Git状态',
    'seg.context': '上下文',
    'seg.crypto': '加密货币',

    'part.bar': '进度条',
    'part.percent': '百分比',
    'part.size': '容量',
    'part.reset': '重置倒计时',
    'part.dirty': '脏状态标记',
    'part.remote': '远程差异',
    'part.branch': '分支',

    'prompt.selectSegments': '选择要启用的段落',
    'prompt.showParts': '显示哪些部分',
    'prompt.barStyle': '进度条样式',
    'prompt.barChar': '进度条字符',
    'prompt.barLength': '进度条长度',
    'prompt.textStyle': '文字样式',
    'prompt.style': '样式',
    'prompt.selectCoins': '选择币种 (最多3个)',
    'prompt.refreshInterval': '刷新间隔',
    'prompt.maxLength': '最大长度 (超出只显示目录名)',
    'prompt.currentOrder': (order) => `当前顺序: ${order}，要调整吗？`,
    'prompt.pickN': (i) => `第 ${i} 个段落`,
    'prompt.saveDefaults': '保存并应用默认配置？',
    'prompt.save': '保存并应用？',

    'style.rainbow': 'Ultrathink (彩虹)',
    'style.cyan': 'Cyan',
    'style.green': 'Green',
    'style.blue': 'Blue',
    'style.yellow': 'Yellow',
    'style.magenta': 'Magenta',
    'style.red': 'Red',
    'style.white': 'White',
    'style.orange': 'Orange',
    'style.pink': 'Pink',
    'style.purple': 'Purple',

    'barStyle.gradient': 'Ultrathink 渐变 (彩虹)',
    'barStyle.trafficLight': '红绿灯 (绿→黄→红)',

    'char.shade': '▓░ 阴影',
    'char.fullBlock': '█░ 全块',
    'char.rectangle': '▬ 水平矩形',

    'unit.chars': '字符',
    'unit.seconds': '秒',

    'msg.preview': '预览:',
    'msg.cancelled': '已取消，配置未保存',
    'msg.cancelledShort': '已取消',
    'msg.saving': '保存配置...',
    'msg.saved': '配置已保存',
    'msg.saveFailed': '保存失败',
    'msg.restart': '重启 Claude Code 或等待下次刷新即可生效',
    'msg.unknownError': '未知错误',
    'msg.missingDeps': (deps) => `缺少必要依赖: ${deps}`,
    'msg.installDeps': (deps) => `请先安装: brew install ${deps}`,
    'msg.noClaudeCode': '未检测到 Claude Code (~/.claude 不存在)',
    'msg.installClaudeCode': '请先安装: https://claude.ai/code',

    'done.segments': (names) => `段落: ${names}`,
    'done.coins': (coins) => `币种=${coins}`,
    'done.refresh': (n) => `刷新=${n}s`,
    'done.length': (n) => `长度=${n}`,
    'done.text': (s) => `文字=${s}`,
    'done.order': (names) => `顺序: ${names}`,
  },
};

let currentLang = 'en';

export function t(key, ...args) {
  const val = messages[currentLang]?.[key] ?? messages.en[key] ?? key;
  return typeof val === 'function' ? val(...args) : val;
}

export function setLang(lang) {
  currentLang = lang;
}

export function getLang() {
  return currentLang;
}

export const SUPPORTED_LANGS = ['en', 'zh'];
