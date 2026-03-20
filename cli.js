#!/usr/bin/env node
import * as p from '@clack/prompts';
import chalk from 'chalk';
import { writeFileSync, readFileSync, existsSync, mkdirSync } from 'fs';
import { join } from 'path';
import { homedir } from 'os';
import { execSync } from 'child_process';
import { loadConfig, saveConfig, DEFAULT_CONFIG, getSegmentLabels, CRYPTO_LIST } from './config.js';
import { getStyleChoices, getBarStyleChoices, getBarChars, BAR_CHARS_DATA, previewColor, previewBar } from './styles.js';
import { generateScript } from './generator.js';
import { t, setLang, getLang, SUPPORTED_LANGS } from './i18n.js';

const SCRIPT_PATH = join(homedir(), '.claude', 'scripts', 'statusline.sh');
const AI_HINT = '\n  Tip: Copy this error to AI for analysis\n  See https://github.com/LokiQ0713/claude-statusline-config#troubleshooting';

// Preview line is always at row 4 after console.clear()
const PREVIEW_ROW = 4;

// --- Pre-flight ---

function preflight() {
  const missing = ['jq', 'perl', 'curl'].filter(cmd => {
    try { execSync(`which ${cmd}`, { stdio: 'ignore' }); return false; }
    catch { return true; }
  });
  if (missing.length > 0) {
    p.cancel(`${t('msg.missingDeps', missing.join(', '))}\n  ${t('msg.installDeps', missing.join(' '))}${AI_HINT}`);
    process.exit(1);
  }
  if (!existsSync(join(homedir(), '.claude'))) {
    p.cancel(`${t('msg.noClaudeCode')}\n  ${t('msg.installClaudeCode')}${AI_HINT}`);
    process.exit(1);
  }
}

// --- Preview ---

const SAMPLE_PRICES = { BTC: 73748, ETH: 2265, BNB: 612, SOL: 178 };

function renderSegment(key, seg) {
  if (!seg.enabled) return null;
  switch (key) {
    case 'model':   return previewColor(chalk, seg.style, `${seg.icon ? seg.icon + ' ' : ''}Opus4.6`);
    case 'cost':    return previewColor(chalk, seg.style, '$0.42');
    case 'usage': {
      const parts = [];
      if (seg.showBar) {
        parts.push(previewBar(chalk, seg.barStyle, seg.barChar, seg.barLength, 0.25));
      }
      if (seg.showPercent) {
        parts.push(previewColor(chalk, seg.textStyle, '25%'));
      }
      if (seg.showReset) {
        parts.push(previewColor(chalk, seg.textStyle, '1h43m'));
      }
      return parts.length > 0 ? parts.join(' ') : null;
    }
    case 'path':    return previewColor(chalk, seg.style, '~/Desktop/web3');
    case 'git': {
      let gitText = 'main';
      if (seg.showDirty !== false) gitText += '*';
      if (seg.showRemote !== false) gitText += ' ↑2↓1';
      return previewColor(chalk, seg.style, gitText);
    }
    case 'context': {
      const parts = [];
      if (seg.showBar !== false) {
        parts.push(previewBar(chalk, seg.barStyle, seg.barChar, seg.barLength));
      }
      if (seg.showPercent !== false) {
        parts.push(previewColor(chalk, seg.textStyle, '60%'));
      }
      if (seg.showSize !== false) {
        parts.push(previewColor(chalk, seg.textStyle, '600K/1M'));
      }
      return parts.length > 0 ? parts.join(' ') : null;
    }
    case 'crypto': {
      const coins = seg.coins || ['BTC', 'ETH'];
      return previewColor(chalk, seg.style, coins.map(c => `${c}:$${SAMPLE_PRICES[c] ?? '?'}`).join(' '));
    }
    default: return null;
  }
}

function renderPreview(config) {
  const order = config.order || Object.keys(config.segments);
  return order
    .map(key => renderSegment(key, config.segments[key]))
    .filter(Boolean)
    .join(' ');
}

// Update preview line in-place using ANSI cursor positioning
function updatePreviewInPlace(config) {
  const preview = '  ' + chalk.dim(`${t('msg.preview')} `) + renderPreview(config);
  // \x1b7 = save cursor, \x1b[row;1H = move to row, \x1b[2K = clear line, \x1b8 = restore cursor
  process.stdout.write(`\x1b7\x1b[${PREVIEW_ROW};1H\x1b[2K${preview}\x1b8`);
}

const doneSteps = [];

function showHeader(config, step) {
  console.clear();
  console.log();
  console.log('  ' + chalk.bold('Claude Statusline Configurator') + chalk.dim(` — ${step}`));
  console.log('  ' + chalk.dim('─'.repeat(56)));
  console.log('  ' + chalk.dim(`${t('msg.preview')} `) + renderPreview(config));
  console.log('  ' + chalk.dim('─'.repeat(56)));
  if (doneSteps.length > 0) {
    for (const s of doneSteps) {
      console.log('  ' + chalk.dim('◇ ') + chalk.dim(s));
    }
    console.log();
  } else {
    console.log();
  }
}

// --- Live preview: intercept arrow keys to update preview in real-time ---

function startLivePreview(config, setter, options) {
  let idx = options.findIndex(o => o.value === setter());
  if (idx < 0) idx = 0;
  const original = setter();

  const handler = (_str, key) => {
    if (!key) return;
    if (key.name === 'up') {
      idx = (idx - 1 + options.length) % options.length;
    } else if (key.name === 'down') {
      idx = (idx + 1) % options.length;
    } else {
      return;
    }
    setter(options[idx].value);
    updatePreviewInPlace(config);
  };

  process.stdin.on('keypress', handler);

  return (finalValue) => {
    process.stdin.removeListener('keypress', handler);
    if (finalValue !== undefined) {
      setter(finalValue);
    } else {
      setter(original); // restore on cancel
    }
  };
}

// --- Live preview for multiselect: intercept space to toggle and update preview ---

function startLiveMultiselectPreview(config, options, initialValues, applyFn) {
  let idx = 0;
  const selected = new Set(initialValues || []);
  const originalSelected = new Set(selected);

  const handler = (_str, key) => {
    if (!key) return;
    if (key.name === 'up') {
      idx = (idx - 1 + options.length) % options.length;
    } else if (key.name === 'down') {
      idx = (idx + 1) % options.length;
    } else if (key.name === 'space') {
      const val = options[idx].value;
      if (selected.has(val)) {
        selected.delete(val);
      } else {
        selected.add(val);
      }
      applyFn([...selected]);
      updatePreviewInPlace(config);
    } else {
      return;
    }
  };

  process.stdin.on('keypress', handler);

  return (finalValues) => {
    process.stdin.removeListener('keypress', handler);
    if (finalValues !== undefined) {
      applyFn(finalValues);
    } else {
      applyFn([...originalSelected]); // restore on cancel
    }
  };
}

async function liveMultiselect(config, applyFn, selectOpts) {
  const stop = startLiveMultiselectPreview(
    config, selectOpts.options, selectOpts.initialValues, applyFn
  );
  const result = await p.multiselect(selectOpts);
  if (p.isCancel(result)) {
    stop(); // restore
    p.cancel(t('msg.cancelled'));
    process.exit(0);
  }
  stop(result);
  return result;
}

// --- Helpers ---

function guard(value) {
  if (p.isCancel(value)) {
    p.cancel(t('msg.cancelled'));
    process.exit(0);
  }
  return value;
}

function styleOptions() {
  return getStyleChoices().map(c => ({ value: c.value, label: c.name }));
}

function barStyleOptions() {
  return getBarStyleChoices().map(c => ({ value: c.value, label: c.name }));
}

function barCharOptions() {
  return Object.entries(getBarChars()).map(([k, v]) => ({ value: k, label: v.label }));
}

// Select with live preview updating
async function liveSelect(config, seg, prop, selectOpts) {
  const accessor = (val) => {
    if (val !== undefined) seg[prop] = val;
    return seg[prop];
  };
  const stop = startLivePreview(config, accessor, selectOpts.options);
  const result = await p.select(selectOpts);
  if (p.isCancel(result)) {
    stop(); // restore original
    p.cancel(t('msg.cancelled'));
    process.exit(0);
  }
  stop(result); // set final value
  return result;
}

// --- Main ---

async function main() {
  preflight();

  const config = loadConfig();
  const s = config.segments;
  const segmentKeys = Object.keys(s);

  // Language selection (first run or no lang set)
  if (!config.lang || !SUPPORTED_LANGS.includes(config.lang)) {
    showHeader(config, 'Language / 语言');
    const lang = guard(await p.select({
      message: t('lang.prompt'),
      options: [
        { value: 'en', label: 'English' },
        { value: 'zh', label: '中文' },
      ],
      initialValue: 'en',
    }));
    config.lang = lang;
    setLang(lang);
  } else {
    setLang(config.lang);
  }

  const SL = getSegmentLabels;

  // Step 0: Quick start or customize?
  showHeader(config, t('step.start'));
  const mode = guard(await p.select({
    message: t('mode.prompt'),
    options: [
      { value: 'defaults', label: t('mode.defaults'), hint: t('mode.defaultsHint') },
      { value: 'custom', label: t('mode.custom'), hint: t('mode.customHint') },
    ],
    initialValue: 'defaults',
  }));

  if (mode === 'defaults') {
    const defaults = structuredClone(DEFAULT_CONFIG);
    defaults.lang = config.lang;
    showHeader(defaults, t('step.confirm'));
    const doSave = guard(await p.confirm({ message: t('prompt.saveDefaults') }));
    if (!doSave) {
      p.cancel(t('msg.cancelledShort'));
      process.exit(0);
    }
    return await saveAndApply(defaults);
  }

  // Step 1: Select which segments to enable
  showHeader(config, t('step.segments'));
  const enabled = await liveMultiselect(config, (selected) => {
    for (const key of segmentKeys) {
      s[key].enabled = selected.includes(key);
    }
  }, {
    message: t('prompt.selectSegments'),
    options: segmentKeys.map(key => ({
      value: key,
      label: `${(SL()[key]?.label || key).padEnd(10)} ${chalk.dim(SL()[key]?.sample || '')}`,
    })),
    initialValues: segmentKeys.filter(k => s[k].enabled),
    required: true,
  });
  doneSteps.push(t('done.segments', enabled.map(k => SL()[k]?.label || k).join(', ')));
  showHeader(config, t('step.segments'));

  // Step 2: Configure each enabled segment
  const currentOrder = (config.order || segmentKeys).filter(k => enabled.includes(k));
  for (const k of enabled) {
    if (!currentOrder.includes(k)) currentOrder.push(k);
  }
  config.order = currentOrder;

  const configOrder = config.order;
  const total = configOrder.length;
  let current = 0;
  for (const key of configOrder) {
    current++;
    const seg = s[key];
    const label = SL()[key]?.label || key;
    const step = t('step.configSegment', current, total);

    if (key === 'context') {
      showHeader(config, step);
      const ctxParts = await liveMultiselect(config, (selected) => {
        seg.showBar = selected.includes('bar');
        seg.showPercent = selected.includes('percent');
        seg.showSize = selected.includes('size');
      }, {
        message: `${label} — ${t('prompt.showParts')}`,
        options: [
          { value: 'bar', label: t('part.bar'), hint: '██████░░░░' },
          { value: 'percent', label: t('part.percent'), hint: '60%' },
          { value: 'size', label: t('part.size'), hint: '600K/1M' },
        ],
        initialValues: [
          ...(seg.showBar !== false ? ['bar'] : []),
          ...(seg.showPercent !== false ? ['percent'] : []),
          ...(seg.showSize !== false ? ['size'] : []),
        ],
        required: true,
      });
      doneSteps.push(`${label}: ${ctxParts.join('+')}`);

      if (seg.showBar) {
        showHeader(config, step);
        await liveSelect(config, seg, 'barStyle', {
          message: `${label} — ${t('prompt.barStyle')}`,
          options: barStyleOptions(),
          initialValue: seg.barStyle,
        });

        showHeader(config, step);
        await liveSelect(config, seg, 'barChar', {
          message: `${label} — ${t('prompt.barChar')}`,
          options: barCharOptions(),
          initialValue: seg.barChar,
        });

        showHeader(config, step);
        await liveSelect(config, seg, 'barLength', {
          message: `${label} — ${t('prompt.barLength')}`,
          options: [10, 12, 15, 20, 25].map(n => ({ value: n, label: `${n} ${t('unit.chars')}` })),
          initialValue: seg.barLength || 15,
        });
      }

      if (seg.showPercent || seg.showSize) {
        showHeader(config, step);
        await liveSelect(config, seg, 'textStyle', {
          message: `${label} — ${t('prompt.textStyle')}`,
          options: styleOptions(),
          initialValue: seg.textStyle || 'ultrathink',
        });
        doneSteps.push(t('done.text', seg.textStyle));
      }
      continue;
    }

    if (key === 'crypto') {
      showHeader(config, step);
      const coins = await liveMultiselect(config, (selected) => {
        seg.coins = selected.slice(0, 3);
      }, {
        message: `${label} — ${t('prompt.selectCoins')}`,
        options: CRYPTO_LIST.map(c => ({
          value: c.symbol,
          label: `${c.symbol.padEnd(6)} ${chalk.dim(c.name)}`,
        })),
        initialValues: seg.coins || ['BTC', 'ETH'],
        required: true,
      });
      seg.coins = coins.slice(0, 3);
      doneSteps.push(t('done.coins', seg.coins.join(', ')));

      showHeader(config, step);
      await liveSelect(config, seg, 'style', {
        message: `${label} — ${t('prompt.style')}`,
        options: styleOptions(),
        initialValue: seg.style,
      });
      doneSteps.push(`${label}: ${seg.style}`);

      showHeader(config, step);
      const interval = guard(await p.select({
        message: `${label} — ${t('prompt.refreshInterval')}`,
        options: [30, 60, 120, 300].map(n => ({ value: n, label: `${n} ${t('unit.seconds')}` })),
        initialValue: seg.refreshInterval || 60,
      }));
      seg.refreshInterval = interval;
      doneSteps.push(t('done.refresh', interval));
      continue;
    }

    if (key === 'usage') {
      showHeader(config, step);
      const usageParts = await liveMultiselect(config, (selected) => {
        seg.showBar = selected.includes('bar');
        seg.showPercent = selected.includes('percent');
        seg.showReset = selected.includes('reset');
      }, {
        message: `${label} — ${t('prompt.showParts')}`,
        options: [
          { value: 'bar', label: t('part.bar'), hint: '████░░░░' },
          { value: 'percent', label: t('part.percent'), hint: '25%' },
          { value: 'reset', label: t('part.reset'), hint: '1h43m' },
        ],
        initialValues: [
          ...(seg.showBar ? ['bar'] : []),
          ...(seg.showPercent ? ['percent'] : []),
          ...(seg.showReset ? ['reset'] : []),
        ],
        required: true,
      });
      doneSteps.push(`${label}: ${usageParts.join('+')}`);
      showHeader(config, step);

      if (seg.showBar) {
        await liveSelect(config, seg, 'barStyle', {
          message: `${label} — ${t('prompt.barStyle')}`,
          options: barStyleOptions(),
          initialValue: seg.barStyle,
        });

        showHeader(config, step);
        await liveSelect(config, seg, 'barChar', {
          message: `${label} — ${t('prompt.barChar')}`,
          options: barCharOptions(),
          initialValue: seg.barChar,
        });

        showHeader(config, step);
        await liveSelect(config, seg, 'barLength', {
          message: `${label} — ${t('prompt.barLength')}`,
          options: [6, 8, 10, 12, 15].map(n => ({ value: n, label: `${n} ${t('unit.chars')}` })),
          initialValue: seg.barLength || 8,
        });
      }

      if (seg.showPercent || seg.showReset) {
        showHeader(config, step);
        await liveSelect(config, seg, 'textStyle', {
          message: `${label} — ${t('prompt.textStyle')}`,
          options: styleOptions(),
          initialValue: seg.textStyle || 'soft-blue',
        });
        doneSteps.push(t('done.text', seg.textStyle));
      }

      showHeader(config, step);
      const interval = guard(await p.select({
        message: `${label} — ${t('prompt.refreshInterval')}`,
        options: [60, 120, 180, 300].map(n => ({ value: n, label: `${n} ${t('unit.seconds')}` })),
        initialValue: seg.refreshInterval || 120,
      }));
      seg.refreshInterval = interval;
      doneSteps.push(t('done.refresh', interval));
      continue;
    }

    if (key === 'path') {
      showHeader(config, step);
      await liveSelect(config, seg, 'style', {
        message: `${label} — ${t('prompt.style')}`,
        options: styleOptions(),
        initialValue: seg.style,
      });
      doneSteps.push(`${label}: ${seg.style}`);

      showHeader(config, step);
      const maxLen = guard(await p.select({
        message: `${label} — ${t('prompt.maxLength')}`,
        options: [15, 20, 30, 40, 50].map(n => ({ value: n, label: `${n} ${t('unit.chars')}` })),
        initialValue: seg.maxLength || 20,
      }));
      seg.maxLength = maxLen;
      doneSteps.push(t('done.length', maxLen));
      continue;
    }

    if (key === 'git') {
      showHeader(config, step);
      const gitParts = await liveMultiselect(config, (selected) => {
        seg.showDirty = selected.includes('dirty');
        seg.showRemote = selected.includes('remote');
      }, {
        message: `${label} — ${t('prompt.showParts')}`,
        options: [
          { value: 'dirty', label: t('part.dirty'), hint: '*' },
          { value: 'remote', label: t('part.remote'), hint: '↑2↓1' },
        ],
        initialValues: [
          ...(seg.showDirty !== false ? ['dirty'] : []),
          ...(seg.showRemote !== false ? ['remote'] : []),
        ],
      });
      doneSteps.push(`${label}: ${[t('part.branch'), ...gitParts].join('+')}`);
      showHeader(config, step);

      await liveSelect(config, seg, 'style', {
        message: `${label} — ${t('prompt.style')}`,
        options: styleOptions(),
        initialValue: seg.style,
      });
      doneSteps.push(`${label}: ${seg.style}`);
      continue;
    }

    if (key === 'model') {
      showHeader(config, step);
      await liveSelect(config, seg, 'style', {
        message: `${label} — ${t('prompt.style')}`,
        options: styleOptions(),
        initialValue: seg.style,
      });
      doneSteps.push(`${label}: ${seg.style}`);

      showHeader(config, step);
      await liveSelect(config, seg, 'icon', {
        message: `${label} — 前缀图标`,
        options: [
          { value: '', label: '无' },
          { value: '🔥', label: '🔥 火焰' },
          { value: '🧠', label: '🧠 大脑' },
          { value: '🦊', label: '🦊 狐狸' },
          { value: '🤖', label: '🤖 机器人' },
        ],
        initialValue: seg.icon || '',
      });
      if (seg.icon) doneSteps.push(`${label}: 图标=${seg.icon}`);
      continue;
    }

    // Generic: style only (cost)
    if (seg.style !== undefined) {
      showHeader(config, step);
      await liveSelect(config, seg, 'style', {
        message: `${label} — ${t('prompt.style')}`,
        options: styleOptions(),
        initialValue: seg.style,
      });
      doneSteps.push(`${label}: ${seg.style}`);
    }
  }

  // Step 3: Reorder segments
  if (enabled.length > 1) {
    showHeader(config, t('step.reorder'));
    const orderStr = config.order.map(k => SL()[k]?.label || k).join(' → ');
    const wantReorder = guard(await p.confirm({
      message: t('prompt.currentOrder', orderStr),
      initialValue: false,
    }));

    if (wantReorder) {
      const newOrder = [];
      const remaining = [...config.order];
      for (let i = 0; i < config.order.length; i++) {
        showHeader({ ...config, order: [...newOrder, ...remaining] }, t('step.reorderN', i + 1, config.order.length));
        const pick = guard(await p.select({
          message: t('prompt.pickN', i + 1),
          options: remaining.map(k => ({
            value: k,
            label: SL()[k]?.label || k,
          })),
        }));
        newOrder.push(pick);
        remaining.splice(remaining.indexOf(pick), 1);
      }
      config.order = newOrder;
    }
  }
  doneSteps.push(t('done.order', config.order.map(k => SL()[k]?.label || k).join(' → ')));

  // Step 4: Preview & Save
  showHeader(config, t('step.confirm'));
  const doSave = guard(await p.confirm({ message: t('prompt.save') }));
  if (!doSave) {
    p.cancel(t('msg.cancelled'));
    process.exit(0);
  }

  await saveAndApply(config);
}

async function saveAndApply(config) {
  const sp = p.spinner();
  sp.start(t('msg.saving'));

  try {
    const claudeDir = join(homedir(), '.claude');
    const scriptsDir = join(claudeDir, 'scripts');
    const settingsPath = join(claudeDir, 'settings.json');

    mkdirSync(scriptsDir, { recursive: true });
    saveConfig(config);

    const script = generateScript(config);
    writeFileSync(SCRIPT_PATH, script, { mode: 0o755 });

    // Update settings.json
    let settings = null;
    if (existsSync(settingsPath)) {
      try { settings = JSON.parse(readFileSync(settingsPath, 'utf8')); }
      catch { settings = null; }
    } else {
      settings = {};
    }

    if (settings !== null) {
      if (!settings.statusLine || settings.statusLine.command !== '~/.claude/scripts/statusline.sh') {
        settings.statusLine = { type: 'command', command: '~/.claude/scripts/statusline.sh', padding: 0 };
        writeFileSync(settingsPath, JSON.stringify(settings, null, 2) + '\n');
      }
    }

    sp.stop(t('msg.saved'));
    p.outro(t('msg.restart'));
  } catch (err) {
    sp.stop(t('msg.saveFailed'));
    p.cancel(`${err.message}${AI_HINT}`);
    process.exit(1);
  }
}

main().catch(err => {
  p.cancel(`${t('msg.unknownError')}: ${err.message}${AI_HINT}`);
  process.exit(1);
});
