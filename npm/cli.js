#!/usr/bin/env node
const { execFileSync } = require('child_process');
const { join } = require('path');
const { homedir } = require('os');
const { existsSync } = require('fs');

const bin = join(homedir(), '.claude', 'statusline', 'bin', 'claude-statusline-config');

if (!existsSync(bin)) {
  console.error('claude-statusline-config binary not found at', bin);
  console.error('Try reinstalling: npm install -g claude-statusline-config');
  process.exit(1);
}

try {
  execFileSync(bin, process.argv.slice(2), { stdio: 'inherit' });
} catch (e) {
  if (e.status) process.exit(e.status);
  process.exit(1);
}
