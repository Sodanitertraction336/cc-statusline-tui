use serde::Deserialize;
use std::io::Read;

// ─── Stdin JSON structs ──────────────────────────────────────────────

#[derive(Deserialize, Default, Debug)]
pub struct StdinInput {
    #[serde(default)]
    pub model: StdinModel,
    #[serde(default)]
    pub workspace: StdinWorkspace,
    #[serde(default)]
    pub context_window: StdinContext,
    #[serde(default)]
    pub cost: StdinCost,
}

#[derive(Deserialize, Default, Debug)]
pub struct StdinModel {
    #[serde(default)]
    pub id: String,
}

#[derive(Deserialize, Default, Debug)]
pub struct StdinWorkspace {
    #[serde(default)]
    pub current_dir: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
pub struct StdinContext {
    #[serde(default)]
    pub context_window_size: Option<u64>,
    #[serde(default)]
    pub used_percentage: Option<f64>,
}

#[derive(Deserialize, Default, Debug)]
pub struct StdinCost {
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
}

// ─── read_stdin ──────────────────────────────────────────────────────

pub fn read_stdin() -> StdinInput {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap_or_default();
    serde_json::from_str(&buf).unwrap_or_default()
}

// ─── Formatting helpers ──────────────────────────────────────────────

/// Capitalize the first character of a string, leaving the rest unchanged.
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let upper: String = c.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

/// Format a model id into a human-friendly short name.
///
/// - Strip `claude-` prefix
/// - Strip anything in `[...]` (e.g. `[1m]`)
/// - Pattern: `name-major-minor...` → `NameMajor.Minor`
///   e.g. `opus-4-6` → `Opus4.6`, `haiku-4-5-20251001` → `Haiku4.5`
/// - If the id doesn't match the expected pattern, return it as-is.
pub fn format_model(id: &str) -> String {
    if id.is_empty() {
        return String::new();
    }

    // Strip bracket suffix like [1m]
    let clean = if let Some(bracket_pos) = id.find('[') {
        &id[..bracket_pos]
    } else {
        id
    };

    // Strip `claude-` prefix
    let without_prefix = clean.strip_prefix("claude-").unwrap_or(clean);

    // Split into parts by `-`
    let parts: Vec<&str> = without_prefix.splitn(3, '-').collect();

    if parts.len() < 3 {
        // Not enough parts to extract name-major-minor, return cleaned id
        // but still capitalize if we stripped a prefix
        if clean != id || without_prefix != clean {
            return capitalize(without_prefix);
        }
        return without_prefix.to_string();
    }

    let name = parts[0];
    let major = parts[1];
    let minor_raw = parts[2];

    // Extract leading digits from minor (e.g. "5-20251001" → "5", "6" → "6")
    let minor: String = minor_raw.chars().take_while(|c| c.is_ascii_digit()).collect();

    if major.is_empty() || minor.is_empty() {
        return capitalize(without_prefix);
    }

    format!("{}{}.{}", capitalize(name), major, minor)
}

/// Format a path for display, replacing the home directory with `~`.
/// If the result exceeds `max_length`, show only the last path component.
pub fn format_path(cwd: &str, home: &str, max_length: usize) -> String {
    let replaced = if !home.is_empty() && cwd.starts_with(home) {
        format!("~{}", &cwd[home.len()..])
    } else {
        cwd.to_string()
    };

    if replaced.len() <= max_length {
        replaced
    } else {
        // Return the last path component (directory name)
        match cwd.rsplit('/').next() {
            Some(last) if !last.is_empty() => last.to_string(),
            _ => replaced,
        }
    }
}

/// Format a number with K/M suffixes.
///
/// - >= 1,000,000: `"1M"` or `"1.5M"`
/// - >= 1,000: `"1K"` or `"1.5K"`
/// - else: plain number
/// - If value is exact (no fractional part), omit `.0`
pub fn format_size(n: u64) -> String {
    if n >= 1_000_000 {
        let val = n as f64 / 1_000_000.0;
        if (val.fract().abs()) < f64::EPSILON {
            format!("{}M", val as u64)
        } else {
            // Format with one decimal, strip trailing zeros
            let s = format!("{:.1}M", val);
            s.replace(".0M", "M")
        }
    } else if n >= 1_000 {
        let val = n as f64 / 1_000.0;
        if (val.fract().abs()) < f64::EPSILON {
            format!("{}K", val as u64)
        } else {
            let s = format!("{:.1}K", val);
            s.replace(".0K", "K")
        }
    } else {
        n.to_string()
    }
}

// ─── Stub run() — Task 6 will implement ──────────────────────────────

pub fn run() {}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_model_opus() {
        assert_eq!(format_model("claude-opus-4-6"), "Opus4.6");
    }

    #[test]
    fn test_format_model_sonnet() {
        assert_eq!(format_model("claude-sonnet-4-6"), "Sonnet4.6");
    }

    #[test]
    fn test_format_model_haiku() {
        assert_eq!(format_model("claude-haiku-4-5-20251001"), "Haiku4.5");
    }

    #[test]
    fn test_format_model_with_bracket() {
        assert_eq!(format_model("claude-opus-4-6[1m]"), "Opus4.6");
    }

    #[test]
    fn test_format_model_unknown() {
        assert_eq!(format_model("unknown"), "unknown");
    }

    #[test]
    fn test_format_path_with_home() {
        assert_eq!(
            format_path("/Users/loki/Desktop/web3", "/Users/loki", 20),
            "~/Desktop/web3"
        );
    }

    #[test]
    fn test_format_path_too_long() {
        assert_eq!(
            format_path(
                "/Users/loki/Desktop/very-long-project-name",
                "/Users/loki",
                15
            ),
            "very-long-project-name"
        );
    }

    #[test]
    fn test_format_path_short() {
        assert_eq!(format_path("/tmp", "/Users/loki", 15), "/tmp");
    }

    #[test]
    fn test_format_size_million() {
        assert_eq!(format_size(1_000_000), "1M");
    }

    #[test]
    fn test_format_size_half_million() {
        assert_eq!(format_size(500_000), "500K");
    }

    #[test]
    fn test_format_size_exact_k() {
        assert_eq!(format_size(2_000), "2K");
    }

    #[test]
    fn test_format_size_fractional() {
        assert_eq!(format_size(1_500), "1.5K");
    }

    #[test]
    fn test_format_size_small() {
        assert_eq!(format_size(500), "500");
    }

    #[test]
    fn test_stdin_deserialize() {
        let json = r#"{
            "model": { "id": "claude-opus-4-6" },
            "workspace": { "cwd": "/Users/loki/project" },
            "context_window": { "context_window_size": 200000, "used_percentage": 0.42 },
            "cost": { "total_cost_usd": 1.23 }
        }"#;
        let input: StdinInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.model.id, "claude-opus-4-6");
        assert_eq!(input.workspace.cwd.as_deref(), Some("/Users/loki/project"));
        assert_eq!(input.context_window.context_window_size, Some(200000));
        assert!((input.context_window.used_percentage.unwrap() - 0.42).abs() < f64::EPSILON);
        assert!((input.cost.total_cost_usd.unwrap() - 1.23).abs() < f64::EPSILON);
    }

    #[test]
    fn test_stdin_empty() {
        let input: StdinInput = serde_json::from_str("{}").unwrap_or_default();
        assert_eq!(input.model.id, "");
        assert!(input.workspace.cwd.is_none());
        assert!(input.context_window.context_window_size.is_none());
        assert!(input.cost.total_cost_usd.is_none());
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("opus"), "Opus");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("A"), "A");
        assert_eq!(capitalize("hello world"), "Hello world");
    }
}
