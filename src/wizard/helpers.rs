//! Utility functions shared across wizard sub-modules.
//!
//! Provides segment label/hint lookups, row manipulation helpers, and
//! enabled-flag setters used by the configuration and menu steps.

use crate::config::Config;
use crate::i18n::t;

/// Returns the i18n label for a segment key.
pub(super) fn seg_label(key: &str) -> &'static str {
    match key {
        "model" => t("seg.model"),
        "cost" => t("seg.cost"),
        "usage" => t("seg.usage"),
        "usage_7d" => t("seg.usage7d"),
        "path" => t("seg.path"),
        "git" => t("seg.git"),
        "context" => t("seg.context"),
        "crypto" => t("seg.crypto"),
        _ => "?",
    }
}

/// Returns the sample hint for a segment (used in the segment multiselect).
pub(super) fn seg_hint(key: &str) -> &'static str {
    match key {
        "model" => "\u{1f525}Opus4.6",
        "cost" => "$0.42",
        "usage" => "5h: 25%",
        "usage_7d" => "7d: 15%",
        "path" => "~/Desktop",
        "git" => "main*",
        "context" => "60% 600K/1M",
        "crypto" => "BTC:$73748",
        _ => "",
    }
}

pub(super) fn set_seg_enabled(config: &mut Config, key: &str, enabled: bool) {
    match key {
        "model" => config.segments.model.enabled = enabled,
        "cost" => config.segments.cost.enabled = enabled,
        "usage" => config.segments.usage.enabled = enabled,
        "usage_7d" => config.segments.usage_7d.enabled = enabled,
        "path" => config.segments.path.enabled = enabled,
        "git" => config.segments.git.enabled = enabled,
        "context" => config.segments.context.enabled = enabled,
        "crypto" => config.segments.crypto.enabled = enabled,
        _ => {}
    }
}

/// Find which row (0-indexed) a segment is in, or None if not assigned.
pub(super) fn find_segment_row(rows: &[Vec<String>], key: &str) -> Option<usize> {
    rows.iter().position(|row| row.iter().any(|k| k == key))
}

/// Remove a segment from all rows.
pub(super) fn remove_segment_from_rows(rows: &mut [Vec<String>], key: &str) {
    for row in rows.iter_mut() {
        row.retain(|k| k != key);
    }
}

/// Default row (1-indexed as string) for each segment type.
pub(super) fn default_row_for_segment(key: &str) -> &'static str {
    match key {
        "model" | "cost" | "path" | "git" | "context" => "1",
        "usage" | "usage_7d" => "2",
        "crypto" => "3",
        _ => "1",
    }
}

/// Helper: translate row index (0-based) to display name.
pub(super) fn t_row(row_idx: usize) -> &'static str {
    match row_idx {
        0 => t("row.1"),
        1 => t("row.2"),
        2 => t("row.3"),
        _ => "?",
    }
}
