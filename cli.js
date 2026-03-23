#!/usr/bin/env node
const { execFileSync, execSync } = require('child_process');
const { join } = require('path');
const { homedir } = require('os');
const { existsSync, readFileSync } = require('fs');

const bin = join(homedir(), '.claude', 'statusline', 'bin', 'cc-statusline');
const versionFile = join(homedir(), '.claude', 'statusline', 'bin', '.version');
const expectedVersion = require('./package.json').version;

// Check if binary exists and version matches
let needsUpdate = !existsSync(bin);
if (!needsUpdate) {
  try {
    const installed = readFileSync(versionFile, 'utf8').trim();
    needsUpdate = installed !== expectedVersion;
  } catch {
    needsUpdate = true;
  }
}

if (needsUpdate) {
  console.log(`[cc-statusline] Updating binary to v${expectedVersion}...`);
  try {
    execSync(`node "${join(__dirname, 'postinstall.js')}"`, { stdio: 'inherit' });
  } catch {
    console.error('[cc-statusline] Failed to update binary. Try: npx cc-statusline-tui@latest');
    process.exit(1);
  }
}

if (!existsSync(bin)) {
  console.error('cc-statusline binary not found at', bin);
  console.error('Try reinstalling: npx cc-statusline-tui@latest');
  process.exit(1);
}

try {
  execFileSync(bin, process.argv.slice(2), { stdio: 'inherit' });
} catch (e) {
  if (e.status) process.exit(e.status);
  process.exit(1);
}
