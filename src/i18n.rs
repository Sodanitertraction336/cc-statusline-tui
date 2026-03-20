use std::sync::atomic::{AtomicU8, Ordering};

/// Supported language codes.
pub const SUPPORTED_LANGS: &[&str] = &["en", "zh"];

/// 0 = en, 1 = zh
static LANG: AtomicU8 = AtomicU8::new(0);

/// Set the current language. Accepts "en" or "zh"; defaults to en for unknown values.
pub fn set_lang(lang: &str) {
    let code = match lang {
        "zh" => 1,
        _ => 0,
    };
    LANG.store(code, Ordering::Relaxed);
}

/// Get the current language code ("en" or "zh").
pub fn get_lang() -> &'static str {
    match LANG.load(Ordering::Relaxed) {
        1 => "zh",
        _ => "en",
    }
}

/// Look up a static translation by key.
/// Returns the key itself if not found.
pub fn t(key: &str) -> &'static str {
    let is_zh = LANG.load(Ordering::Relaxed) == 1;
    if is_zh {
        t_zh(key)
    } else {
        t_en(key)
    }
}

/// Look up a dynamic translation by key, replacing positional placeholders with `args`.
///
/// Placeholders are `{0}`, `{1}`, etc., corresponding to the index in `args`.
/// Returns the key itself (no substitution) if the key is not found.
pub fn tf(key: &str, args: &[&str]) -> String {
    let is_zh = LANG.load(Ordering::Relaxed) == 1;
    let template = if is_zh { tf_zh(key) } else { tf_en(key) };
    match template {
        Some(tmpl) => substitute(tmpl, args),
        None => key.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn substitute(template: &str, args: &[&str]) -> String {
    let mut result = template.to_string();
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{}}}", i);
        result = result.replace(&placeholder, arg);
    }
    result
}

// ---------------------------------------------------------------------------
// English — static keys
// ---------------------------------------------------------------------------

fn t_en(key: &str) -> &'static str {
    match key {
        // Language selector (always bilingual)
        "lang.prompt" => "Language / 语言",

        // Steps
        "step.start" => "Start",
        "step.segments" => "1/4 Segments",
        "step.reorder" => "3/4 Reorder",
        "step.confirm" => "4/4 Confirm",

        // Mode selection
        "mode.prompt" => "Config mode",
        "mode.defaults" => "Use defaults",
        "mode.defaultsHint" => "Save directly, just press Enter",
        "mode.custom" => "Customize",
        "mode.customHint" => "Adjust styles and options step by step",

        // Segments
        "seg.model" => "Model",
        "seg.cost" => "Cost",
        "seg.usage" => "5h Limit",
        "seg.path" => "Path",
        "seg.git" => "Git",
        "seg.context" => "Context",
        "seg.crypto" => "Crypto",

        // Sub-parts
        "part.bar" => "Progress bar",
        "part.percent" => "Percentage",
        "part.size" => "Capacity",
        "part.reset" => "Reset timer",
        "part.dirty" => "Dirty marker",
        "part.remote" => "Remote diff",
        "part.branch" => "branch",

        // Prompts
        "prompt.selectSegments" => "Select segments to enable",
        "prompt.showParts" => "Show which parts",
        "prompt.barStyle" => "Bar style",
        "prompt.barChar" => "Bar character",
        "prompt.barLength" => "Bar length",
        "prompt.textStyle" => "Text style",
        "prompt.style" => "Style",
        "prompt.selectCoins" => "Select coins (max 3)",
        "prompt.refreshInterval" => "Refresh interval",
        "prompt.maxLength" => "Max length (shows dir name if exceeded)",
        "prompt.saveDefaults" => "Save and apply defaults?",
        "prompt.save" => "Save and apply?",

        // Styles
        "style.rainbow" => "Ultrathink (Rainbow)",
        "style.cyan" => "Cyan",
        "style.green" => "Green",
        "style.blue" => "Blue",
        "style.yellow" => "Yellow",
        "style.magenta" => "Magenta",
        "style.red" => "Red",
        "style.white" => "White",
        "style.orange" => "Orange",
        "style.pink" => "Pink",
        "style.purple" => "Purple",

        // Bar styles
        "barStyle.gradient" => "Ultrathink Gradient (Rainbow)",
        "barStyle.trafficLight" => "Traffic light (G→Y→R)",

        // Bar characters
        "char.shade" => "▓░ Shade",
        "char.fullBlock" => "█░ Full block",
        "char.rectangle" => "▬ Rectangle",

        // Units
        "unit.chars" => "chars",
        "unit.seconds" => "s",

        // Messages
        "msg.preview" => "Preview:",
        "msg.cancelled" => "Cancelled, config not saved",
        "msg.cancelledShort" => "Cancelled",
        "msg.saving" => "Saving config...",
        "msg.saved" => "Config saved",
        "msg.saveFailed" => "Save failed",
        "msg.restart" => "Restart Claude Code or wait for next refresh",
        "msg.unknownError" => "Unknown error",
        "msg.noClaudeCode" => "Claude Code not detected (~/.claude does not exist)",
        "msg.installClaudeCode" => "Please install: https://claude.ai/code",

        // Fallback: return the key itself
        _ => key_to_static(key),
    }
}

// ---------------------------------------------------------------------------
// Chinese — static keys
// ---------------------------------------------------------------------------

fn t_zh(key: &str) -> &'static str {
    match key {
        "lang.prompt" => "Language / 语言",

        "step.start" => "开始",
        "step.segments" => "1/4 选择段落",
        "step.reorder" => "3/4 排列顺序",
        "step.confirm" => "4/4 确认",

        "mode.prompt" => "配置模式",
        "mode.defaults" => "使用默认配置",
        "mode.defaultsHint" => "直接保存，按回车即可",
        "mode.custom" => "自定义配置",
        "mode.customHint" => "逐项调整样式和选项",

        "seg.model" => "模型",
        "seg.cost" => "费用",
        "seg.usage" => "5h限额",
        "seg.path" => "目录",
        "seg.git" => "Git状态",
        "seg.context" => "上下文",
        "seg.crypto" => "加密货币",

        "part.bar" => "进度条",
        "part.percent" => "百分比",
        "part.size" => "容量",
        "part.reset" => "重置倒计时",
        "part.dirty" => "脏状态标记",
        "part.remote" => "远程差异",
        "part.branch" => "分支",

        "prompt.selectSegments" => "选择要启用的段落",
        "prompt.showParts" => "显示哪些部分",
        "prompt.barStyle" => "进度条样式",
        "prompt.barChar" => "进度条字符",
        "prompt.barLength" => "进度条长度",
        "prompt.textStyle" => "文字样式",
        "prompt.style" => "样式",
        "prompt.selectCoins" => "选择币种 (最多3个)",
        "prompt.refreshInterval" => "刷新间隔",
        "prompt.maxLength" => "最大长度 (超出只显示目录名)",
        "prompt.saveDefaults" => "保存并应用默认配置？",
        "prompt.save" => "保存并应用？",

        "style.rainbow" => "Ultrathink (彩虹)",
        "style.cyan" => "Cyan",
        "style.green" => "Green",
        "style.blue" => "Blue",
        "style.yellow" => "Yellow",
        "style.magenta" => "Magenta",
        "style.red" => "Red",
        "style.white" => "White",
        "style.orange" => "Orange",
        "style.pink" => "Pink",
        "style.purple" => "Purple",

        "barStyle.gradient" => "Ultrathink 渐变 (彩虹)",
        "barStyle.trafficLight" => "红绿灯 (绿→黄→红)",

        "char.shade" => "▓░ 阴影",
        "char.fullBlock" => "█░ 全块",
        "char.rectangle" => "▬ 水平矩形",

        "unit.chars" => "字符",
        "unit.seconds" => "秒",

        "msg.preview" => "预览:",
        "msg.cancelled" => "已取消，配置未保存",
        "msg.cancelledShort" => "已取消",
        "msg.saving" => "保存配置...",
        "msg.saved" => "配置已保存",
        "msg.saveFailed" => "保存失败",
        "msg.restart" => "重启 Claude Code 或等待下次刷新即可生效",
        "msg.unknownError" => "未知错误",
        "msg.noClaudeCode" => "未检测到 Claude Code (~/.claude 不存在)",
        "msg.installClaudeCode" => "请先安装: https://claude.ai/code",

        _ => key_to_static(key),
    }
}

// ---------------------------------------------------------------------------
// English — dynamic (template) keys
// ---------------------------------------------------------------------------

fn tf_en(key: &str) -> Option<&'static str> {
    match key {
        "step.configSegment" => Some("2/4 Configure ({0}/{1})"),
        "step.reorderN" => Some("3/4 Reorder ({0}/{1})"),
        "msg.missingDeps" => Some("Missing dependencies: {0}"),
        "msg.installDeps" => Some("Please install: brew install {0}"),
        "prompt.currentOrder" => Some("Current order: {0}, adjust?"),
        "prompt.pickN" => Some("Position {0}"),
        "done.segments" => Some("Segments: {0}"),
        "done.coins" => Some("Coins: {0}"),
        "done.refresh" => Some("Refresh: {0}s"),
        "done.length" => Some("Length: {0}"),
        "done.text" => Some("Text: {0}"),
        "done.order" => Some("Order: {0}"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Chinese — dynamic (template) keys
// ---------------------------------------------------------------------------

fn tf_zh(key: &str) -> Option<&'static str> {
    match key {
        "step.configSegment" => Some("2/4 配置段落 ({0}/{1})"),
        "step.reorderN" => Some("3/4 排列顺序 ({0}/{1})"),
        "msg.missingDeps" => Some("缺少必要依赖: {0}"),
        "msg.installDeps" => Some("请先安装: brew install {0}"),
        "prompt.currentOrder" => Some("当前顺序: {0}，要调整吗？"),
        "prompt.pickN" => Some("第 {0} 个段落"),
        "done.segments" => Some("段落: {0}"),
        "done.coins" => Some("币种={0}"),
        "done.refresh" => Some("刷新={0}s"),
        "done.length" => Some("长度={0}"),
        "done.text" => Some("文字={0}"),
        "done.order" => Some("顺序: {0}"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Leak the key string to produce a &'static str fallback.
// This is intentional: unknown keys are rare, and the leaked memory is tiny.
// ---------------------------------------------------------------------------

fn key_to_static(key: &str) -> &'static str {
    Box::leak(key.to_string().into_boxed_str())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_lang_is_en() {
        set_lang("en");
        assert_eq!(get_lang(), "en");
    }

    #[test]
    fn test_set_lang_zh() {
        set_lang("zh");
        assert_eq!(get_lang(), "zh");
        set_lang("en");
    }

    #[test]
    fn test_set_lang_unknown_falls_back_to_en() {
        set_lang("fr");
        assert_eq!(get_lang(), "en");
        set_lang("en");
    }

    #[test]
    fn test_t_en() {
        set_lang("en");
        assert_eq!(t("step.start"), "Start");
        assert_eq!(t("seg.model"), "Model");
        assert_eq!(t("msg.saved"), "Config saved");
    }

    #[test]
    fn test_t_zh() {
        set_lang("zh");
        assert_eq!(t("step.start"), "开始");
        assert_eq!(t("seg.model"), "模型");
        set_lang("en");
    }

    #[test]
    fn test_t_unknown_key_returns_key() {
        set_lang("en");
        assert_eq!(t("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_tf_en() {
        set_lang("en");
        assert_eq!(tf("step.configSegment", &["2", "4"]), "2/4 Configure (2/4)");
        assert_eq!(tf("done.refresh", &["30"]), "Refresh: 30s");
        assert_eq!(
            tf("msg.missingDeps", &["jq perl"]),
            "Missing dependencies: jq perl"
        );
    }

    #[test]
    fn test_tf_zh() {
        set_lang("zh");
        assert_eq!(
            tf("step.configSegment", &["2", "4"]),
            "2/4 配置段落 (2/4)"
        );
        assert_eq!(tf("done.refresh", &["30"]), "刷新=30s");
        set_lang("en");
    }

    #[test]
    fn test_tf_unknown_key_returns_key() {
        set_lang("en");
        assert_eq!(tf("nonexistent.key", &["arg"]), "nonexistent.key");
    }

    #[test]
    fn test_supported_langs() {
        assert_eq!(SUPPORTED_LANGS, &["en", "zh"]);
    }

    #[test]
    fn test_lang_prompt_same_in_both() {
        set_lang("en");
        let en = t("lang.prompt");
        set_lang("zh");
        let zh = t("lang.prompt");
        assert_eq!(en, zh);
        set_lang("en");
    }

    #[test]
    fn test_style_names_same_in_both() {
        for name in &[
            "style.cyan",
            "style.green",
            "style.blue",
            "style.yellow",
            "style.magenta",
            "style.red",
            "style.white",
            "style.orange",
            "style.pink",
            "style.purple",
        ] {
            set_lang("en");
            let en = t(name);
            set_lang("zh");
            let zh = t(name);
            assert_eq!(en, zh, "style key '{}' should be identical in en and zh", name);
        }
        set_lang("en");
    }
}
