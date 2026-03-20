import { COLORS, ULTRATHINK, BAR_CHARS_DATA } from './styles.js';
import { CRYPTO_LIST } from './config.js';

export function generateScript(config) {
  const s = config.segments;
  const needsUltrathink = Object.values(s).some(seg =>
    seg.enabled && (seg.style === 'ultrathink' || seg.barStyle === 'ultrathink-gradient' || seg.textStyle === 'ultrathink'
      || (seg.barStyle === 'ultrathink-gradient' && seg.showBar))
  );
  const needsCrypto = s.crypto.enabled;
  const needsUsage = s.usage.enabled;

  return [
    genHeader(s),
    genInputParsing(),
    genFormatting(s),
    s.git.enabled ? genGitStatus(s) : '',
    genColors(s),
    genCostFormat(s),
    genContextColor(s),
    needsCrypto ? genCryptoFetcher(s) : '',
    needsUsage ? genUsageFetcher(s) : '',
    genOutput(config, needsUltrathink),
  ].filter(Boolean).join('\n');
}

function genHeader(s) {
  const flags = [
    `cfg_model=${s.model.enabled}`,
    `cfg_cost=${s.cost.enabled}`,
    `cfg_path=${s.path.enabled}`,
    `cfg_context=${s.context.enabled}`,
    `cfg_usage=${s.usage.enabled}`,
    `cfg_git=${s.git.enabled}`,
    `cfg_crypto=${s.crypto.enabled}`,
  ].join('\n');
  return `#!/usr/bin/env bash
input=$(cat)

# Segment config
${flags}
`;
}

function genInputParsing() {
  return `# Timestamp — single call, reused everywhere
_now=$(date +%s)

# Extract fields from input
_jq_out=$(echo "$input" | jq -r '
  @sh "model_id=\\(.model.id // "unknown")",
  @sh "cwd=\\(.workspace.current_dir // .cwd // "?")",
  @sh "ctx_size=\\(.context_window.context_window_size // "")",
  @sh "used_pct=\\(.context_window.used_percentage // "")",
  @sh "cost_usd=\\(.cost.total_cost_usd // "")"
' 2>/dev/null)
[ $? -eq 0 ] && [ -n "$_jq_out" ] && eval "$_jq_out"
`;
}

function genFormatting(s) {
  const maxLen = s.path.maxLength || 20;
  return `# Format via perl — use | delimiter for space-safe paths
IFS='|' read -r model_short short_path ctx_size_fmt ctx_used_fmt used_int < <(
  perl -e '
    my ($model_id, $cwd, $home, $ctx_size, $used_pct) = @ARGV;

    my $m = $model_id;
    $m =~ s/^claude-//;
    $m =~ s/^(\\w+)-(\\d+)-(\\d+)/\\u$1$2.$3/;
    $m =~ s/\\[.*\\]//;

    my $p = $cwd;
    $p =~ s/^\\Q$home\\E/~/;
    if (length($p) > ${maxLen}) {
      my @parts = split m{/}, $p;
      $p = $parts[-1] || "/";
    }

    sub fmt {
      my $n = shift;
      return "" unless defined $n && $n ne "";
      if ($n >= 1000000) { my $v=$n/1000000; return ($v==int($v)) ? sprintf("%dM",$v) : sprintf("%.1fM",$v) }
      elsif ($n >= 1000) { my $v=$n/1000; return ($v==int($v)) ? sprintf("%dK",$v) : sprintf("%.1fK",$v) }
      else { return $n }
    }

    my $cs = fmt($ctx_size);
    my $cu = "";
    my $ui = "";
    if ($used_pct ne "" && $ctx_size ne "") {
      my $used = int($used_pct * $ctx_size / 100);
      $cu = fmt($used);
      $ui = sprintf("%.0f", $used_pct);
    }

    print "$m|$p|$cs|$cu|$ui\\n";
  ' "$model_id" "$cwd" "$HOME" "$ctx_size" "$used_pct"
)
`;
}

function genGitStatus(s) {
  const showDirty = s.git.showDirty !== false;
  const showRemote = s.git.showRemote !== false;

  let code = `# Git status
git_info=""
_branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
if [ -n "$_branch" ]; then
`;

  if (showDirty) {
    code += `  _dirty=""
  [ -n "$(git status --porcelain 2>/dev/null)" ] && _dirty="*"
`;
  }

  if (showRemote) {
    code += `  _ahead_behind=""
  _counts=$(git rev-list --left-right --count HEAD...@{u} 2>/dev/null)
  if [ -n "$_counts" ]; then
    _ahead=$(echo "$_counts" | awk '{print $1}')
    _behind=$(echo "$_counts" | awk '{print $2}')
    [ "$_ahead" -gt 0 ] 2>/dev/null && _ahead_behind="\${_ahead_behind} ↑\${_ahead}"
    [ "$_behind" -gt 0 ] 2>/dev/null && _ahead_behind="\${_ahead_behind} ↓\${_behind}"
  fi
`;
  }

  const parts = ['${_branch}'];
  if (showDirty) parts.push('${_dirty}');
  if (showRemote) parts.push('${_ahead_behind}');
  code += `  git_info="${parts.join('')}"
fi
`;
  return code;
}

function genColors(s) {
  // Collect only the colors actually used
  const used = new Set();
  for (const seg of Object.values(s)) {
    if (seg.style && seg.style !== 'ultrathink') used.add(seg.style);
    if (seg.barStyle && seg.barStyle !== 'ultrathink-gradient' && seg.barStyle !== 'semantic') used.add(seg.barStyle);
    if (seg.textStyle && seg.textStyle !== 'ultrathink') used.add(seg.textStyle);
  }
  // Always need these for semantic context color
  used.add('soft-green'); used.add('soft-yellow'); used.add('soft-red');

  const defs = [...used].filter(c => COLORS[c]).map(c => `${c.replace(/-/g, '_')}='${COLORS[c]}'`).join('\n');
  return `# Colors
${defs}
reset='\\033[0m'
`;
}

function genCostFormat() {
  return `# Format cost
cost_fmt=""
if [ -n "$cost_usd" ] && [ "$cost_usd" != "null" ]; then
  cost_fmt=$(printf '$%.2f' "$cost_usd")
fi
`;
}

function genContextColor() {
  return `# Context color (semantic)
ctx_color="$soft_green"
if [ -n "$used_int" ]; then
  if [ "$used_int" -ge 75 ] 2>/dev/null; then
    ctx_color="$soft_red"
  elif [ "$used_int" -ge 50 ] 2>/dev/null; then
    ctx_color="$soft_yellow"
  fi
fi
`;
}

function genCryptoFetcher(s) {
  const interval = s.crypto.refreshInterval || 60;
  const coins = s.crypto.coins || ['BTC', 'ETH'];
  const coinMap = Object.fromEntries(CRYPTO_LIST.map(c => [c.symbol, c.pair]));

  // Generate fetch commands for each coin
  const fetchLines = coins.map((sym, i) => {
    const pair = coinMap[sym] || `${sym}USDT`;
    const varName = `price_${i}`;
    return `    ${varName}=$(curl -s --max-time 5 "https://api.binance.com/api/v3/ticker/price?symbol=${pair}" 2>/dev/null | jq -r '.price // empty')`;
  }).join('\n');

  const varNames = coins.map((_, i) => `$price_${i}`);
  const checkAll = coins.map((_, i) => `[ -n "$price_${i}" ]`).join(' && ');

  // Use different printf format for small-value coins (< $1)
  const printfParts = coins.map((sym, i) => {
    const isSmallValue = ['DOGE', 'TRX', 'SHIB', 'ADA', 'XRP', 'MATIC'].includes(sym);
    return isSmallValue ? `%s` : `%.0f`;
  }).join('|');
  const printfArgs = coins.map((_, i) => `"$price_${i}"`).join(' ');

  // Build display string
  const readVars = coins.map((_, i) => `coin_${i}`).join(' ');
  const displayParts = coins.map((sym, i) => {
    const isSmallValue = ['DOGE', 'TRX', 'SHIB', 'ADA', 'XRP', 'MATIC'].includes(sym);
    if (isSmallValue) {
      return `${sym}:\\$\${coin_${i}}`;
    }
    return `${sym}:\\$\${coin_${i}}`;
  }).join(' ');

  return `# Crypto prices (cached, background refresh every ${interval}s)
CACHE_FILE="/tmp/claude-statusline-crypto-cache"
CACHE_LOCK="/tmp/claude-statusline-crypto-lock"

fetch_prices() {
  mkdir "$CACHE_LOCK" 2>/dev/null || return
  (
${fetchLines}
    if ${checkAll}; then
      printf "${printfParts}" ${printfArgs} > "$CACHE_FILE"
    else
      touch "$CACHE_FILE"
    fi
    rmdir "$CACHE_LOCK" 2>/dev/null
  ) & disown
}

crypto_info=""
if [ "$cfg_crypto" != "true" ]; then :
elif [ -f "$CACHE_FILE" ]; then
  cache_age=$((_now - $(stat -f %m "$CACHE_FILE" 2>/dev/null || echo 0)))
  [ "$cache_age" -ge ${interval} ] && fetch_prices
  IFS='|' read -r ${readVars} < "$CACHE_FILE"
  [ -n "$coin_0" ] && crypto_info="${displayParts}"
else
  fetch_prices
fi
`;
}

function genUsageFetcher(s) {
  const interval = s.usage.refreshInterval || 120;
  return `# Usage limits (cached, background refresh every ${interval}s)
USAGE_CACHE="/tmp/claude-statusline-usage-cache"
USAGE_LOCK="/tmp/claude-statusline-usage-lock"

fetch_usage() {
  mkdir "$USAGE_LOCK" 2>/dev/null || return
  (
    _creds=$(security find-generic-password -s "Claude Code-credentials" -w 2>/dev/null)
    _token=$(echo "$_creds" | jq -r '.claudeAiOauth.accessToken' 2>/dev/null)
    unset _creds
    if [ -n "$_token" ] && [ "$_token" != "null" ]; then
      result=$(curl -s --max-time 5 \\
        -H "Authorization: Bearer $_token" \\
        -H "anthropic-beta: oauth-2025-04-20" \\
        -H "User-Agent: claude-code/2.1.76" \\
        "https://api.anthropic.com/api/oauth/usage" 2>/dev/null)
      unset _token
      if [ $? -eq 0 ] && echo "$result" | jq -e '.five_hour' > /dev/null 2>&1; then
        echo "$result" | jq -r '"\\(.five_hour.utilization | floor)|\\(.five_hour.resets_at)"' > "$USAGE_CACHE"
      else
        touch "$USAGE_CACHE"
      fi
    else
      unset _token
      touch "$USAGE_CACHE"
    fi
    rmdir "$USAGE_LOCK" 2>/dev/null
  ) & disown
}

usage_pct=""
usage_reset=""
if [ "$cfg_usage" != "true" ]; then :
elif [ -f "$USAGE_CACHE" ]; then
  cache_age=$((_now - $(stat -f %m "$USAGE_CACHE" 2>/dev/null || echo 0)))
  [ "$cache_age" -ge ${interval} ] && fetch_usage
  IFS='|' read -r usage_5h resets_at < "$USAGE_CACHE"
  if [ -n "$usage_5h" ]; then
    usage_pct="$usage_5h"
    if [ -n "$resets_at" ] && [ "$resets_at" != "null" ]; then
      clean_ts=$(echo "$resets_at" | perl -pe 's/\\.\\d+//; s/(\\d{2}):(\\d{2})$/$1$2/')
      reset_epoch=$(date -juf "%Y-%m-%dT%H:%M:%S%z" "$clean_ts" "+%s" 2>/dev/null)
      if [ -n "$reset_epoch" ]; then
        diff=$((reset_epoch - _now))
        if [ "$diff" -gt 0 ]; then
          hours=$((diff / 3600))
          mins=$(( (diff % 3600) / 60 ))
          [ "$hours" -gt 0 ] && usage_reset="\${hours}h\${mins}m" || usage_reset="\${mins}m"
        fi
      fi
    fi
  fi
else
  fetch_usage
fi
`;
}

function bashColorVar(style) {
  return `$${style.replace(/-/g, '_')}`;
}

function genRainbowText(varName, textExpr) {
  return `  ${varName}=""
  _text="${textExpr}"
  for (( i=0; i<\${#_text}; i++ )); do
    ch="\${_text:\$i:1}"
    ci=$(( (i + offset) % 7 ))
    if [ "$use_shimmer" -eq 1 ]; then
      ${varName}="\${${varName}}\\033[38;2;\${r_shim[\$ci]}m\${ch}"
    else
      ${varName}="\${${varName}}\\033[38;2;\${r_main[\$ci]}m\${ch}"
    fi
  done`;
}

// Generate a segment's render code (handles both plain color and ultrathink)
function genSegmentRender(key, seg, textExpr, { prefix = ' ', guard = '' } = {}) {
  if (!seg.enabled) return '';
  const style = seg.style;
  const condStart = guard ? `${guard} ` : '';

  if (style === 'ultrathink') {
    const varName = `rainbow_${key}`;
    return `${condStart && `if ${guard}; then\n`}${genRainbowText(varName, textExpr)}
  parts="\${parts}${prefix}\${${varName}}\${reset}"
${condStart && `fi\n`}`;
  } else {
    const colorVar = bashColorVar(style).slice(1);
    if (guard) {
      return `${guard} && parts="\${parts}${prefix}\${${colorVar}}${textExpr}\${reset}"\n`;
    }
    return `parts="\${parts}${prefix}\${${colorVar}}${textExpr}\${reset}"\n`;
  }
}

function genOutput(config, needsUltrathink) {
  const s = config.segments;
  const order = config.order || Object.keys(s);
  let code = '# Build output\nparts=""\n';

  // Ultrathink setup FIRST (before any segment rendering)
  if (needsUltrathink) {
    code += `
# Ultrathink rainbow colors
r_main=(${ULTRATHINK.main.map(c => `"${c}"`).join(' ')})
r_shim=(${ULTRATHINK.shimmer.map(c => `"${c}"`).join(' ')})
cr_m=(${ULTRATHINK.cr_m.join(' ')}); cg_m=(${ULTRATHINK.cg_m.join(' ')}); cb_m=(${ULTRATHINK.cb_m.join(' ')})
cr_s=(${ULTRATHINK.cr_s.join(' ')}); cg_s=(${ULTRATHINK.cg_s.join(' ')}); cb_s=(${ULTRATHINK.cb_s.join(' ')})
offset=$((_now % 7))
use_shimmer=$((_now % 2))
if [ "$use_shimmer" -eq 1 ]; then
  _cr=("\${cr_s[@]}"); _cg=("\${cg_s[@]}"); _cb=("\${cb_s[@]}")
else
  _cr=("\${cr_m[@]}"); _cg=("\${cg_m[@]}"); _cb=("\${cb_m[@]}")
fi
`;
  }

  // Render segments in configured order
  let isFirst = true;
  for (const key of order) {
    const seg = s[key];
    if (!seg || !seg.enabled) continue;

    const prefix = isFirst ? '' : ' ';
    isFirst = false;

    switch (key) {
      case 'model':
        code += genSegmentRender('model', seg, `${seg.icon ? seg.icon + ' ' : ''}\${model_short}`, {
          prefix, guard: '[ "$cfg_model" = "true" ]'
        });
        break;
      case 'cost':
        code += genSegmentRender('cost', seg, '${cost_fmt}', {
          prefix, guard: '[ "$cfg_cost" = "true" ] && [ -n "$cost_fmt" ]'
        });
        break;
      case 'usage':
        code += genUsageOutput(seg, prefix);
        break;
      case 'path':
        code += genSegmentRender('path', seg, '${short_path}', {
          prefix, guard: '[ "$cfg_path" = "true" ]'
        });
        break;
      case 'git':
        code += genSegmentRender('git', seg, '${git_info}', {
          prefix, guard: '[ "$cfg_git" = "true" ] && [ -n "$git_info" ]'
        });
        break;
      case 'context': {
        const charData = BAR_CHARS_DATA[seg.barChar] || { char: '█' };
        const ch = charData.char;
        const emptyCh = charData.empty || ch;
        const len = seg.barLength || 15;
        code += genContextBar(seg, ch, emptyCh, len);
        break;
      }
      case 'crypto':
        code += genSegmentRender('crypto', seg, '${crypto_info}', {
          prefix, guard: '[ "$cfg_crypto" = "true" ] && [ -n "$crypto_info" ]'
        });
        break;
    }
  }

  code += `parts="\${parts# }"

printf "%b" "$parts"
`;
  return code;
}

function genUsageOutput(seg, prefix) {
  const showBar = seg.showBar !== false;
  const showPercent = seg.showPercent !== false;
  const showReset = seg.showReset !== false;
  const barStyle = seg.barStyle || 'semantic';
  const textStyle = seg.textStyle || 'soft-blue';
  const barLen = seg.barLength || 8;
  const charData = BAR_CHARS_DATA[seg.barChar] || { char: '█' };
  const ch = charData.char;
  const emptyCh = charData.empty || ch;

  let code = `if [ "$cfg_usage" = "true" ] && [ -n "$usage_pct" ]; then
  usage_out=""
`;

  // Bar sub-component
  if (showBar) {
    code += `  # Usage bar
  u_total=${barLen}
  u_filled=$((usage_pct * u_total / 100))
  [ "$u_filled" -gt "$u_total" ] && u_filled=$u_total
  u_empty=$((u_total - u_filled))
  u_dim='\\033[38;5;239m'
  u_bar=""
`;
    // Determine semantic color for usage bar
    code += `  u_bar_color="$soft_green"
  if [ "$usage_pct" -ge 75 ] 2>/dev/null; then
    u_bar_color="$soft_red"
  elif [ "$usage_pct" -ge 50 ] 2>/dev/null; then
    u_bar_color="$soft_yellow"
  fi
`;
    if (barStyle === 'ultrathink-gradient') {
      code += `  for (( i=0; i<u_filled; i++ )); do
    if [ "$u_filled" -le 1 ]; then
      u_bar="\${u_bar}\\033[38;2;\${_cr[0]};\${_cg[0]};\${_cb[0]}m${ch}"
    else
      scaled=$(( i * 600 / (u_filled - 1) ))
      idx=$(( scaled / 100 ))
      frac=$(( scaled % 100 ))
      [ "$idx" -ge 6 ] && idx=5 && frac=100
      next=$(( idx + 1 ))
      r=$(( _cr[idx] + (_cr[next] - _cr[idx]) * frac / 100 ))
      g=$(( _cg[idx] + (_cg[next] - _cg[idx]) * frac / 100 ))
      b=$(( _cb[idx] + (_cb[next] - _cb[idx]) * frac / 100 ))
      u_bar="\${u_bar}\\033[38;2;\${r};\${g};\${b}m${ch}"
    fi
  done
`;
    } else if (barStyle === 'semantic') {
      code += `  for (( i=0; i<u_filled; i++ )); do u_bar="\${u_bar}\${u_bar_color}${ch}"; done
`;
    } else {
      const color = COLORS[barStyle] || COLORS.green;
      code += `  u_fill_color='${color}'
  for (( i=0; i<u_filled; i++ )); do u_bar="\${u_bar}\${u_fill_color}${ch}"; done
`;
    }
    code += `  if [ "$u_empty" -gt 0 ]; then
    u_bar="\${u_bar}\${u_dim}"
    for (( i=0; i<u_empty; i++ )); do u_bar="\${u_bar}${emptyCh}"; done
  fi
  usage_out="\${u_bar}\${reset}"
`;
  }

  // Percent sub-component
  if (showPercent) {
    if (textStyle === 'ultrathink') {
      code += `  _upct_text="\${usage_pct}%"
  rainbow_upct=""
  for (( i=0; i<\${#_upct_text}; i++ )); do
    ch="\${_upct_text:\$i:1}"
    ci=$(( (i + offset) % 7 ))
    if [ "$use_shimmer" -eq 1 ]; then
      rainbow_upct="\${rainbow_upct}\\033[38;2;\${r_shim[\$ci]}m\${ch}"
    else
      rainbow_upct="\${rainbow_upct}\\033[38;2;\${r_main[\$ci]}m\${ch}"
    fi
  done
  [ -n "$usage_out" ] && usage_out="\${usage_out} \${rainbow_upct}\${reset}" || usage_out="\${rainbow_upct}\${reset}"
`;
    } else {
      const color = COLORS[textStyle] || COLORS['soft-blue'];
      code += `  [ -n "$usage_out" ] && usage_out="\${usage_out} ${color}\${usage_pct}%\${reset}" || usage_out="${color}\${usage_pct}%\${reset}"
`;
    }
  }

  // Reset sub-component
  if (showReset) {
    if (textStyle === 'ultrathink') {
      code += `  if [ -n "$usage_reset" ]; then
    _urst_text="\${usage_reset}"
    rainbow_urst=""
    for (( i=0; i<\${#_urst_text}; i++ )); do
      ch="\${_urst_text:\$i:1}"
      ci=$(( (i + offset) % 7 ))
      if [ "$use_shimmer" -eq 1 ]; then
        rainbow_urst="\${rainbow_urst}\\033[38;2;\${r_shim[\$ci]}m\${ch}"
      else
        rainbow_urst="\${rainbow_urst}\\033[38;2;\${r_main[\$ci]}m\${ch}"
      fi
    done
    [ -n "$usage_out" ] && usage_out="\${usage_out} \${rainbow_urst}\${reset}" || usage_out="\${rainbow_urst}\${reset}"
  fi
`;
    } else {
      const color = COLORS[textStyle] || COLORS['soft-blue'];
      code += `  if [ -n "$usage_reset" ]; then
    [ -n "$usage_out" ] && usage_out="\${usage_out} ${color}\${usage_reset}\${reset}" || usage_out="${color}\${usage_reset}\${reset}"
  fi
`;
    }
  }

  code += `  [ -n "$usage_out" ] && parts="\${parts}${prefix}\${usage_out}"
fi
`;
  return code;
}

function genContextBar(ctx, ch, emptyCh, len) {
  const showBar = ctx.showBar !== false;
  const showPercent = ctx.showPercent !== false;
  const showSize = ctx.showSize !== false;
  const barStyle = ctx.barStyle || 'semantic';
  const textStyle = ctx.textStyle || 'ultrathink';

  let code = `if [ "$cfg_context" = "true" ] && [ -n "$used_int" ]; then
  ctx_out=""
`;

  // Bar sub-component
  if (showBar) {
    code += `  total=${len}
  filled=$((used_int * total / 100))
  [ "$filled" -gt "$total" ] && filled=$total
  empty=$((total - filled))
  dim='\\033[38;5;239m'
  bar=""
`;
    if (barStyle === 'ultrathink-gradient') {
      code += `  for (( i=0; i<filled; i++ )); do
    if [ "$filled" -le 1 ]; then
      bar="\${bar}\\033[38;2;\${_cr[0]};\${_cg[0]};\${_cb[0]}m${ch}"
    else
      scaled=$(( i * 600 / (filled - 1) ))
      idx=$(( scaled / 100 ))
      frac=$(( scaled % 100 ))
      [ "$idx" -ge 6 ] && idx=5 && frac=100
      next=$(( idx + 1 ))
      r=$(( _cr[idx] + (_cr[next] - _cr[idx]) * frac / 100 ))
      g=$(( _cg[idx] + (_cg[next] - _cg[idx]) * frac / 100 ))
      b=$(( _cb[idx] + (_cb[next] - _cb[idx]) * frac / 100 ))
      bar="\${bar}\\033[38;2;\${r};\${g};\${b}m${ch}"
    fi
  done
`;
    } else if (barStyle === 'semantic') {
      code += `  for (( i=0; i<filled; i++ )); do bar="\${bar}\${ctx_color}${ch}"; done
`;
    } else {
      const color = COLORS[barStyle] || COLORS.green;
      code += `  fill_color='${color}'
  for (( i=0; i<filled; i++ )); do bar="\${bar}\${fill_color}${ch}"; done
`;
    }
    code += `  if [ "$empty" -gt 0 ]; then
    bar="\${bar}\${dim}"
    for (( i=0; i<empty; i++ )); do bar="\${bar}${emptyCh}"; done
  fi
  ctx_out="\${bar}\${reset}"
`;
  }

  // Percent sub-component
  if (showPercent) {
    const pctText = '${used_int}%';
    if (textStyle === 'ultrathink') {
      code += genRainbowAppend('ctx_out', `\${used_int}%`, 'rainbow_cpct');
    } else {
      const color = COLORS[textStyle] || '';
      const colorCode = color || '${ctx_color}';
      code += `  [ -n "$ctx_out" ] && ctx_out="\${ctx_out} ${colorCode}\${used_int}%\${reset}" || ctx_out="${colorCode}\${used_int}%\${reset}"
`;
    }
  }

  // Size sub-component
  if (showSize) {
    if (textStyle === 'ultrathink') {
      code += genRainbowAppend('ctx_out', `\${ctx_used_fmt}/\${ctx_size_fmt}`, 'rainbow_csz');
    } else {
      const color = COLORS[textStyle] || '';
      const colorCode = color || '${ctx_color}';
      code += `  [ -n "$ctx_out" ] && ctx_out="\${ctx_out} ${colorCode}\${ctx_used_fmt}/\${ctx_size_fmt}\${reset}" || ctx_out="${colorCode}\${ctx_used_fmt}/\${ctx_size_fmt}\${reset}"
`;
    }
  }

  code += `  [ -n "$ctx_out" ] && parts="\${parts} \${ctx_out}"
fi
`;
  return code;
}

// Helper: generate rainbow text and append to an output var
function genRainbowAppend(outVar, textExpr, rainbowVar) {
  return `  _rt="${textExpr}"
  ${rainbowVar}=""
  for (( i=0; i<\${#_rt}; i++ )); do
    ch="\${_rt:\$i:1}"
    ci=$(( (i + offset) % 7 ))
    if [ "$use_shimmer" -eq 1 ]; then
      ${rainbowVar}="\${${rainbowVar}}\\033[38;2;\${r_shim[\$ci]}m\${ch}"
    else
      ${rainbowVar}="\${${rainbowVar}}\\033[38;2;\${r_main[\$ci]}m\${ch}"
    fi
  done
  [ -n "$${outVar}" ] && ${outVar}="\${${outVar}} \${${rainbowVar}}\${reset}" || ${outVar}="\${${rainbowVar}}\${reset}"
`;
}
