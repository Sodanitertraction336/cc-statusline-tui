import { t } from './i18n.js';

// Color definitions matching statusline.sh
export const COLORS = {
  cyan:           '\\033[0;36m',
  green:          '\\033[0;32m',
  blue:           '\\033[1;34m',
  yellow:         '\\033[0;33m',
  magenta:        '\\033[0;35m',
  red:            '\\033[0;31m',
  white:          '\\033[0;37m',
  'soft-green':   '\\033[38;5;71m',
  'soft-yellow':  '\\033[38;5;179m',
  'soft-red':     '\\033[38;5;167m',
  'soft-blue':    '\\033[38;5;75m',
  'soft-cyan':    '\\033[38;5;80m',
  'soft-magenta': '\\033[38;5;176m',
  orange:         '\\033[38;5;208m',
  pink:           '\\033[38;5;212m',
  purple:         '\\033[38;5;141m',
};

export const ULTRATHINK = {
  main: ['235;95;87', '245;139;87', '250;195;95', '145;200;130', '130;170;220', '155;130;200', '200;130;180'],
  shimmer: ['250;155;147', '255;185;137', '255;225;155', '185;230;180', '180;205;240', '195;180;230', '230;180;210'],
  cr_m: [235, 245, 250, 145, 130, 155, 200],
  cg_m: [95, 139, 195, 200, 170, 130, 130],
  cb_m: [87, 87, 95, 130, 220, 200, 180],
  cr_s: [250, 255, 255, 185, 180, 195, 230],
  cg_s: [155, 185, 225, 230, 205, 180, 180],
  cb_s: [147, 137, 155, 180, 240, 230, 210],
};

export const BAR_CHARS_DATA = {
  'shade':          { char: '▓', empty: '░' },
  'full-block':     { char: '█', empty: '░' },
  'rectangle':      { char: '▬' },
};

export function getBarChars() {
  return {
    'shade':      { char: '▓', empty: '░', label: t('char.shade') },
    'full-block': { char: '█', empty: '░', label: t('char.fullBlock') },
    'rectangle':  { char: '▬', label: t('char.rectangle') },
  };
}

// Default style per segment
export const SEGMENT_DEFAULTS = {
  model:   'cyan',
  cost:    'green',
  usage:   'soft-blue',
  path:    'ultrathink',
  context: 'ultrathink-gradient',
  contextText: 'ultrathink',
  crypto:  'yellow',
};

// Style choices for UI — functions so t() picks up current language
export function getStyleChoices() {
  return [
    { name: t('style.rainbow'),  value: 'ultrathink' },
    { name: t('style.cyan'),     value: 'cyan' },
    { name: t('style.green'),    value: 'green' },
    { name: t('style.blue'),     value: 'blue' },
    { name: t('style.yellow'),   value: 'yellow' },
    { name: t('style.magenta'),  value: 'magenta' },
    { name: t('style.red'),      value: 'red' },
    { name: t('style.white'),    value: 'white' },
    { name: t('style.orange'),   value: 'orange' },
    { name: t('style.pink'),     value: 'pink' },
    { name: t('style.purple'),   value: 'purple' },
  ];
}

export function getBarStyleChoices() {
  return [
    { name: t('barStyle.gradient'),      value: 'ultrathink-gradient' },
    { name: t('barStyle.trafficLight'),   value: 'semantic' },
    { name: t('style.cyan'),             value: 'cyan' },
    { name: t('style.green'),            value: 'green' },
    { name: t('style.blue'),             value: 'blue' },
    { name: t('style.yellow'),           value: 'yellow' },
    { name: t('style.magenta'),          value: 'magenta' },
    { name: t('style.orange'),           value: 'orange' },
    { name: t('style.pink'),             value: 'pink' },
    { name: t('style.purple'),           value: 'purple' },
  ];
}

// Chalk-based preview helpers
export function previewColor(chalk, style, text) {
  if (style === 'ultrathink') {
    const colors = ULTRATHINK.main.map(c => c.split(';').map(Number));
    return [...text].map((ch, i) => {
      const [r, g, b] = colors[i % 7];
      return chalk.rgb(r, g, b)(ch);
    }).join('');
  }
  const map = {
    cyan: chalk.cyan, green: chalk.green, blue: chalk.blueBright,
    yellow: chalk.yellow, magenta: chalk.magenta, red: chalk.red, white: chalk.white,
    'soft-green': chalk.rgb(95, 175, 95), 'soft-yellow': chalk.rgb(215, 175, 95),
    'soft-red': chalk.rgb(215, 95, 95), 'soft-blue': chalk.rgb(95, 175, 255),
    'soft-cyan': chalk.rgb(95, 215, 215), 'soft-magenta': chalk.rgb(215, 175, 215),
    orange: chalk.rgb(255, 135, 0), pink: chalk.rgb(255, 135, 175), purple: chalk.rgb(175, 135, 255),
  };
  const fn = map[style] || chalk.white;
  return fn(text);
}

export function previewBar(chalk, barStyle, barChar, length, ratio = 0.6) {
  const filled = Math.round(length * ratio);
  const empty = length - filled;
  const charData = BAR_CHARS_DATA[barChar] || { char: '█' };
  const ch = charData.char;
  const emptyCh = charData.empty || ch;
  let filledStr = '';
  if (barStyle === 'ultrathink-gradient') {
    const colors = ULTRATHINK.main.map(c => c.split(';').map(Number));
    for (let i = 0; i < filled; i++) {
      const t = filled <= 1 ? 0 : i / (filled - 1);
      const pos = t * 6;
      const idx = Math.min(Math.floor(pos), 5);
      const frac = pos - idx;
      const [r1, g1, b1] = colors[idx];
      const [r2, g2, b2] = colors[idx + 1] || colors[idx];
      const r = Math.round(r1 + (r2 - r1) * frac);
      const g = Math.round(g1 + (g2 - g1) * frac);
      const b = Math.round(b1 + (b2 - b1) * frac);
      filledStr += chalk.rgb(r, g, b)(ch);
    }
  } else if (barStyle === 'semantic') {
    filledStr = chalk.rgb(95, 175, 95)(ch.repeat(filled));
  } else {
    filledStr = previewColor(chalk, barStyle, ch.repeat(filled));
  }
  const emptyStr = charData.empty
    ? chalk.rgb(68, 68, 68)(emptyCh.repeat(empty))
    : chalk.rgb(68, 68, 68)(ch.repeat(empty));
  return filledStr + emptyStr;
}
