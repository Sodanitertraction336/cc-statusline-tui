import { readFileSync, writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';
import { homedir } from 'os';
import { t } from './i18n.js';

const CONFIG_PATH = join(homedir(), '.claude', 'statusline.config.json');

export const CRYPTO_LIST = [
  { symbol: 'BTC',  name: 'Bitcoin',  pair: 'BTCUSDT' },
  { symbol: 'ETH',  name: 'Ethereum', pair: 'ETHUSDT' },
  { symbol: 'BNB',  name: 'BNB',      pair: 'BNBUSDT' },
  { symbol: 'SOL',  name: 'Solana',   pair: 'SOLUSDT' },
];

export const DEFAULT_CONFIG = {
  order: ['model', 'cost', 'usage', 'path', 'git', 'context', 'crypto'],
  segments: {
    model:   { enabled: true, style: 'ultrathink', icon: '' },
    cost:    { enabled: true, style: 'green' },
    usage:   {
      enabled: true,
      showBar: false,
      showPercent: true,
      showReset: true,
      barStyle: 'semantic',
      barChar: 'full-block',
      barLength: 8,
      textStyle: 'white',
      refreshInterval: 120,
    },
    path:    { enabled: true, style: 'cyan', maxLength: 15 },
    git:     { enabled: true, style: 'cyan', showDirty: true, showRemote: true },
    context: {
      enabled: true,
      showBar: true,
      showPercent: true,
      showSize: true,
      barStyle: 'ultrathink-gradient',
      barChar: 'rectangle',
      barLength: 12,
      textStyle: 'white',
    },
    crypto:  { enabled: true, style: 'green', refreshInterval: 60, coins: ['BTC'] },
  },
};

export function getSegmentLabels() {
  return {
    model:   { label: t('seg.model'),   sample: 'Opus4.6' },
    cost:    { label: t('seg.cost'),    sample: '$0.42' },
    usage:   { label: t('seg.usage'),   sample: '████░░░░ 25% 1h43m' },
    path:    { label: t('seg.path'),    sample: '~/Desktop/web3' },
    git:     { label: t('seg.git'),     sample: 'main* ↑2↓1' },
    context: { label: t('seg.context'), sample: '██████░░░░ 60% 600K/1M' },
    crypto:  { label: t('seg.crypto'),  sample: 'BTC:$73748 ETH:$2265' },
  };
}

export function loadConfig() {
  try {
    const raw = readFileSync(CONFIG_PATH, 'utf8');
    const saved = JSON.parse(raw);
    // Merge with defaults to handle new fields
    return deepMerge(DEFAULT_CONFIG, saved);
  } catch {
    return structuredClone(DEFAULT_CONFIG);
  }
}

export function saveConfig(config) {
  mkdirSync(join(homedir(), '.claude'), { recursive: true });
  writeFileSync(CONFIG_PATH, JSON.stringify(config, null, 2) + '\n');
}

function deepMerge(target, source) {
  const result = structuredClone(target);
  for (const key of Object.keys(source)) {
    if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
      result[key] = deepMerge(result[key] || {}, source[key]);
    } else {
      result[key] = source[key];
    }
  }
  return result;
}
