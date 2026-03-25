//! Segment configuration flows.
//!
//! Each `configure_*` function implements a multi-step sub-wizard for one
//! segment type (model, cost, usage, path, git, context, crypto). The main
//! dispatcher `configure_segment` routes by key, and
//! `configure_segment_with_row` wraps it with row-assignment logic.

use crate::config::{Config, CRYPTO_LIST};
use crate::i18n::t;

use super::helpers::{
    default_row_for_segment, find_segment_row, remove_segment_from_rows, seg_label,
    set_seg_enabled,
};
use super::options::{
    bar_char_options, bar_length_options, bar_style_options, icon_options, max_length_options,
    refresh_interval_options, style_options,
};
use super::{multiselect, preview, select, step_progress};
use super::{show_screen, StepResult, PREVIEW_ROW};

pub(super) fn configure_segment_with_row(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
    key: &str,
) {
    // Save state for rollback
    let orig_segments = config.segments.clone();
    let orig_rows = config.rows.clone();

    loop {
        let step_label = seg_label(key).to_string();

        // Temporarily add segment to its default row so preview can show it
        if find_segment_row(&config.rows, key).is_none() {
            let default_idx: usize = match default_row_for_segment(key) {
                "2" => 1,
                "3" => 2,
                _ => 0,
            };
            if config.rows.len() > default_idx {
                config.rows[default_idx].push(key.to_string());
            }
        }
        set_seg_enabled(config, key, true);

        // Phase 1: Configure segment options (style, etc.)
        let result = configure_segment(config, steps, key, &step_label);
        match result {
            StepResult::Next => {
                // Phase 2: Select which row
                match select_row_for_segment(config, steps, key) {
                    StepResult::Next => {
                        // Success: enable segment
                        set_seg_enabled(config, key, true);
                        return;
                    }
                    StepResult::Back => {
                        // Back from row select -> re-enter configure
                        continue;
                    }
                    StepResult::Cancelled => {
                        config.segments = orig_segments;
                        config.rows = orig_rows;
                        return;
                    }
                }
            }
            StepResult::Back | StepResult::Cancelled => {
                // Back/cancel from first config sub-step -> discard, return to menu
                config.segments = orig_segments;
                config.rows = orig_rows;
                return;
            }
        }
    }
}

fn select_row_for_segment(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
    key: &str,
) -> StepResult {
    let step_label = seg_label(key).to_string();
    show_screen(config, steps, 0, &step_label);

    let current_row = find_segment_row(&config.rows, key);
    let default = current_row
        .map(|r| match r {
            0 => "1",
            1 => "2",
            2 => "3",
            _ => "1",
        })
        .unwrap_or_else(|| default_row_for_segment(key));

    let opts = vec![
        select::SelectOption {
            value: "1".into(),
            label: t("row.1").into(),
            hint: None,
        },
        select::SelectOption {
            value: "2".into(),
            label: t("row.2").into(),
            hint: None,
        },
        select::SelectOption {
            value: "3".into(),
            label: t("row.3").into(),
            hint: None,
        },
    ];

    let prompt = format!("{} \u{2014} {}", seg_label(key), t("prompt.selectRow"));
    let config_ptr = config as *mut Config;
    let footer = step_progress::render_pending_footer(steps, 0);

    let result = select::select(
        &prompt,
        &opts,
        Some(default),
        &mut |v: &str| {
            let cfg = unsafe { &mut *config_ptr };
            remove_segment_from_rows(&mut cfg.rows, key);
            if let Ok(r) = v.parse::<usize>() {
                let idx = r.saturating_sub(1).min(2);
                if cfg.rows.len() > idx {
                    cfg.rows[idx].push(key.to_string());
                }
            }
            set_seg_enabled(cfg, key, true);
            preview::update_preview_in_place(cfg, PREVIEW_ROW);
        },
        Some(&footer),
    );

    match result {
        select::SelectResult::Selected(v) => {
            let row_idx = v.parse::<usize>().unwrap_or(1).saturating_sub(1).min(2);
            remove_segment_from_rows(&mut config.rows, key);
            if config.rows.len() > row_idx {
                config.rows[row_idx].push(key.to_string());
            }
            StepResult::Next
        }
        select::SelectResult::Back => StepResult::Back,
        select::SelectResult::Cancelled => StepResult::Cancelled,
    }
}

fn configure_segment(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
    key: &str,
    step_label: &str,
) -> StepResult {
    match key {
        "model" => configure_model(config, steps, step_label),
        "cost" => configure_cost(config, steps, step_label),
        "usage" => configure_usage(config, steps, step_label),
        "usage_7d" => {
            // Reuse configure_usage by temporarily swapping usage_7d into the usage slot
            std::mem::swap(&mut config.segments.usage, &mut config.segments.usage_7d);
            let result = configure_usage(config, steps, step_label);
            std::mem::swap(&mut config.segments.usage, &mut config.segments.usage_7d);
            result
        }
        "path" => configure_path(config, steps, step_label),
        "git" => configure_git(config, steps, step_label),
        "context" => configure_context(config, steps, step_label),
        "crypto" => configure_crypto(config, steps, step_label),
        _ => StepResult::Next,
    }
}

// ── Model configuration ─────────────────────────────────────────────────

fn configure_model(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
    step_label: &str,
) -> StepResult {
    // Sub-step 0: style
    // Sub-step 1: icon
    let mut sub: usize = 0;
    loop {
        match sub {
            0 => {
                let orig = config.segments.model.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("model"), t("prompt.style"));
                let opts = style_options();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&config.segments.model.style),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.model.style = v.to_string();
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        config.segments.model.style = v;
                        sub = 1;
                    }
                    select::SelectResult::Back => {
                        config.segments.model = orig;
                        return StepResult::Back;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.model = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            1 => {
                let orig = config.segments.model.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — Icon", seg_label("model"));
                let opts = icon_options();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&config.segments.model.icon),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.model.icon = v.to_string();
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        config.segments.model.icon = v;
                        return StepResult::Next;
                    }
                    select::SelectResult::Back => {
                        config.segments.model = orig;
                        sub = 0;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.model = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            _ => return StepResult::Next,
        }
    }
}

// ── Cost configuration ──────────────────────────────────────────────────

fn configure_cost(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
    step_label: &str,
) -> StepResult {
    let orig = config.segments.cost.clone();
    show_screen(config, steps, 0, step_label);
    let prompt = format!("{} — {}", seg_label("cost"), t("prompt.style"));
    let opts = style_options();
    let config_ptr = config as *mut Config;
    let footer = step_progress::render_pending_footer(steps, 0);
    let result = select::select(
        &prompt,
        &opts,
        Some(&config.segments.cost.style),
        &mut |v: &str| {
            let cfg = unsafe { &mut *config_ptr };
            cfg.segments.cost.style = v.to_string();
            preview::update_preview_in_place(cfg, PREVIEW_ROW);
        },
        Some(&footer),
    );
    match result {
        select::SelectResult::Selected(v) => {
            config.segments.cost.style = v;
            StepResult::Next
        }
        select::SelectResult::Back => {
            config.segments.cost = orig;
            StepResult::Back
        }
        select::SelectResult::Cancelled => {
            config.segments.cost = orig;
            StepResult::Cancelled
        }
    }
}

// ── Usage configuration ─────────────────────────────────────────────────

fn configure_usage(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
    step_label: &str,
) -> StepResult {
    // Sub-steps: 0=parts, 1=style, 2=barChar, 3=barLength, 4=refresh
    let mut sub: usize = 0;
    loop {
        match sub {
            0 => {
                // Parts multiselect
                let orig = config.segments.usage.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("usage"), t("prompt.showParts"));
                let opts = vec![
                    multiselect::MultiselectOption {
                        value: "bar".into(),
                        label: t("part.bar").into(),
                        hint: None,
                    },
                    multiselect::MultiselectOption {
                        value: "percent".into(),
                        label: t("part.percent").into(),
                        hint: None,
                    },
                    multiselect::MultiselectOption {
                        value: "reset".into(),
                        label: t("part.reset").into(),
                        hint: None,
                    },
                ];
                let mut initial = Vec::new();
                if config.segments.usage.show_bar {
                    initial.push("bar".to_string());
                }
                if config.segments.usage.show_percent {
                    initial.push("percent".to_string());
                }
                if config.segments.usage.show_reset {
                    initial.push("reset".to_string());
                }
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = multiselect::multiselect(
                    &prompt,
                    &opts,
                    &initial,
                    true,
                    &mut |selected: &[String]| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.usage.show_bar = selected.contains(&"bar".to_string());
                        cfg.segments.usage.show_percent =
                            selected.contains(&"percent".to_string());
                        cfg.segments.usage.show_reset = selected.contains(&"reset".to_string());
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    multiselect::MultiselectResult::Selected(selected) => {
                        config.segments.usage.show_bar = selected.contains(&"bar".to_string());
                        config.segments.usage.show_percent =
                            selected.contains(&"percent".to_string());
                        config.segments.usage.show_reset =
                            selected.contains(&"reset".to_string());
                        sub = 1;
                    }
                    multiselect::MultiselectResult::Back => {
                        config.segments.usage = orig;
                        return StepResult::Back;
                    }
                    multiselect::MultiselectResult::Cancelled => {
                        config.segments.usage = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            1 => {
                let orig = config.segments.usage.clone();
                // Bar style (only if bar is shown)
                if config.segments.usage.show_bar {
                    show_screen(config, steps, 0, step_label);
                    let prompt = format!("{} — {}", seg_label("usage"), t("prompt.barStyle"));
                    let opts = bar_style_options();
                    let config_ptr = config as *mut Config;
                    let footer = step_progress::render_pending_footer(steps, 0);
                    let result = select::select(
                        &prompt,
                        &opts,
                        Some(&config.segments.usage.style),
                        &mut |v: &str| {
                            let cfg = unsafe { &mut *config_ptr };
                            cfg.segments.usage.style = v.to_string();
                            preview::update_preview_in_place(cfg, PREVIEW_ROW);
                        },
                        Some(&footer),
                    );
                    match result {
                        select::SelectResult::Selected(v) => {
                            config.segments.usage.style = v;
                            sub = 2;
                        }
                        select::SelectResult::Back => {
                            config.segments.usage = orig;
                            sub = 0;
                        }
                        select::SelectResult::Cancelled => {
                            config.segments.usage = orig;
                            return StepResult::Cancelled;
                        }
                    }
                } else {
                    // Text style (no bar shown)
                    show_screen(config, steps, 0, step_label);
                    let prompt = format!("{} — {}", seg_label("usage"), t("prompt.textStyle"));
                    let opts = style_options();
                    let config_ptr = config as *mut Config;
                    let footer = step_progress::render_pending_footer(steps, 0);
                    let result = select::select(
                        &prompt,
                        &opts,
                        Some(&config.segments.usage.style),
                        &mut |v: &str| {
                            let cfg = unsafe { &mut *config_ptr };
                            cfg.segments.usage.style = v.to_string();
                            preview::update_preview_in_place(cfg, PREVIEW_ROW);
                        },
                        Some(&footer),
                    );
                    match result {
                        select::SelectResult::Selected(v) => {
                            config.segments.usage.style = v;
                            // No bar -> done
                            return StepResult::Next;
                        }
                        select::SelectResult::Back => {
                            config.segments.usage = orig;
                            sub = 0;
                        }
                        select::SelectResult::Cancelled => {
                            config.segments.usage = orig;
                            return StepResult::Cancelled;
                        }
                    }
                }
            }
            2 => {
                // Bar char
                let orig = config.segments.usage.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("usage"), t("prompt.barChar"));
                let opts = bar_char_options();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&config.segments.usage.bar_char),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.usage.bar_char = v.to_string();
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        config.segments.usage.bar_char = v;
                        sub = 3;
                    }
                    select::SelectResult::Back => {
                        config.segments.usage = orig;
                        sub = 1;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.usage = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            3 => {
                // Bar length
                let orig = config.segments.usage.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("usage"), t("prompt.barLength"));
                let opts = bar_length_options();
                let current = config.segments.usage.bar_length.to_string();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&current),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        if let Ok(n) = v.parse::<u32>() {
                            cfg.segments.usage.bar_length = n;
                            preview::update_preview_in_place(cfg, PREVIEW_ROW);
                        }
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        if let Ok(n) = v.parse::<u32>() {
                            config.segments.usage.bar_length = n;
                        }
                        return StepResult::Next;
                    }
                    select::SelectResult::Back => {
                        config.segments.usage = orig;
                        sub = 2;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.usage = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            _ => return StepResult::Next,
        }
    }
}

// ── Path configuration ──────────────────────────────────────────────────

fn configure_path(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
    step_label: &str,
) -> StepResult {
    // Sub-steps: 0=style, 1=maxLength
    let mut sub: usize = 0;
    loop {
        match sub {
            0 => {
                let orig = config.segments.path.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("path"), t("prompt.style"));
                let opts = style_options();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&config.segments.path.style),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.path.style = v.to_string();
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        config.segments.path.style = v;
                        sub = 1;
                    }
                    select::SelectResult::Back => {
                        config.segments.path = orig;
                        return StepResult::Back;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.path = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            1 => {
                let orig = config.segments.path.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("path"), t("prompt.maxLength"));
                let opts = max_length_options();
                let current = config.segments.path.max_length.to_string();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&current),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        if let Ok(n) = v.parse::<u32>() {
                            cfg.segments.path.max_length = n;
                            preview::update_preview_in_place(cfg, PREVIEW_ROW);
                        }
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        if let Ok(n) = v.parse::<u32>() {
                            config.segments.path.max_length = n;
                        }
                        return StepResult::Next;
                    }
                    select::SelectResult::Back => {
                        config.segments.path = orig;
                        sub = 0;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.path = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            _ => return StepResult::Next,
        }
    }
}

// ── Git configuration ───────────────────────────────────────────────────

fn configure_git(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
    step_label: &str,
) -> StepResult {
    // Sub-steps: 0=parts, 1=style
    let mut sub: usize = 0;
    loop {
        match sub {
            0 => {
                let orig = config.segments.git.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("git"), t("prompt.showParts"));
                let opts = vec![
                    multiselect::MultiselectOption {
                        value: "dirty".into(),
                        label: t("part.dirty").into(),
                        hint: Some("*".into()),
                    },
                    multiselect::MultiselectOption {
                        value: "remote".into(),
                        label: t("part.remote").into(),
                        hint: Some("\u{2191}2\u{2193}1".into()),
                    },
                ];
                let mut initial = Vec::new();
                if config.segments.git.show_dirty {
                    initial.push("dirty".to_string());
                }
                if config.segments.git.show_remote {
                    initial.push("remote".to_string());
                }
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = multiselect::multiselect(
                    &prompt,
                    &opts,
                    &initial,
                    false,
                    &mut |selected: &[String]| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.git.show_dirty = selected.contains(&"dirty".to_string());
                        cfg.segments.git.show_remote = selected.contains(&"remote".to_string());
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    multiselect::MultiselectResult::Selected(selected) => {
                        config.segments.git.show_dirty =
                            selected.contains(&"dirty".to_string());
                        config.segments.git.show_remote =
                            selected.contains(&"remote".to_string());
                        sub = 1;
                    }
                    multiselect::MultiselectResult::Back => {
                        config.segments.git = orig;
                        return StepResult::Back;
                    }
                    multiselect::MultiselectResult::Cancelled => {
                        config.segments.git = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            1 => {
                let orig = config.segments.git.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("git"), t("prompt.style"));
                let opts = style_options();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&config.segments.git.style),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.git.style = v.to_string();
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        config.segments.git.style = v;
                        return StepResult::Next;
                    }
                    select::SelectResult::Back => {
                        config.segments.git = orig;
                        sub = 0;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.git = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            _ => return StepResult::Next,
        }
    }
}

// ── Context configuration ───────────────────────────────────────────────

fn configure_context(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
    step_label: &str,
) -> StepResult {
    // Sub-steps: 0=parts, 1=style, 2=barChar, 3=barLength
    let mut sub: usize = 0;
    loop {
        match sub {
            0 => {
                let orig = config.segments.context.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("context"), t("prompt.showParts"));
                let opts = vec![
                    multiselect::MultiselectOption {
                        value: "bar".into(),
                        label: t("part.bar").into(),
                        hint: None,
                    },
                    multiselect::MultiselectOption {
                        value: "percent".into(),
                        label: t("part.percent").into(),
                        hint: None,
                    },
                    multiselect::MultiselectOption {
                        value: "size".into(),
                        label: t("part.size").into(),
                        hint: Some("600K/1M".into()),
                    },
                ];
                let mut initial = Vec::new();
                if config.segments.context.show_bar {
                    initial.push("bar".to_string());
                }
                if config.segments.context.show_percent {
                    initial.push("percent".to_string());
                }
                if config.segments.context.show_size {
                    initial.push("size".to_string());
                }
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = multiselect::multiselect(
                    &prompt,
                    &opts,
                    &initial,
                    true,
                    &mut |selected: &[String]| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.context.show_bar = selected.contains(&"bar".to_string());
                        cfg.segments.context.show_percent =
                            selected.contains(&"percent".to_string());
                        cfg.segments.context.show_size = selected.contains(&"size".to_string());
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    multiselect::MultiselectResult::Selected(selected) => {
                        config.segments.context.show_bar =
                            selected.contains(&"bar".to_string());
                        config.segments.context.show_percent =
                            selected.contains(&"percent".to_string());
                        config.segments.context.show_size =
                            selected.contains(&"size".to_string());
                        sub = 1;
                    }
                    multiselect::MultiselectResult::Back => {
                        config.segments.context = orig;
                        return StepResult::Back;
                    }
                    multiselect::MultiselectResult::Cancelled => {
                        config.segments.context = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            1 => {
                let orig = config.segments.context.clone();
                if config.segments.context.show_bar {
                    show_screen(config, steps, 0, step_label);
                    let prompt =
                        format!("{} — {}", seg_label("context"), t("prompt.barStyle"));
                    let opts = bar_style_options();
                    let config_ptr = config as *mut Config;
                    let footer = step_progress::render_pending_footer(steps, 0);
                    let result = select::select(
                        &prompt,
                        &opts,
                        Some(&config.segments.context.style),
                        &mut |v: &str| {
                            let cfg = unsafe { &mut *config_ptr };
                            cfg.segments.context.style = v.to_string();
                            preview::update_preview_in_place(cfg, PREVIEW_ROW);
                        },
                        Some(&footer),
                    );
                    match result {
                        select::SelectResult::Selected(v) => {
                            config.segments.context.style = v;
                            sub = 2;
                        }
                        select::SelectResult::Back => {
                            config.segments.context = orig;
                            sub = 0;
                        }
                        select::SelectResult::Cancelled => {
                            config.segments.context = orig;
                            return StepResult::Cancelled;
                        }
                    }
                } else {
                    show_screen(config, steps, 0, step_label);
                    let prompt =
                        format!("{} — {}", seg_label("context"), t("prompt.textStyle"));
                    let opts = style_options();
                    let config_ptr = config as *mut Config;
                    let footer = step_progress::render_pending_footer(steps, 0);
                    let result = select::select(
                        &prompt,
                        &opts,
                        Some(&config.segments.context.style),
                        &mut |v: &str| {
                            let cfg = unsafe { &mut *config_ptr };
                            cfg.segments.context.style = v.to_string();
                            preview::update_preview_in_place(cfg, PREVIEW_ROW);
                        },
                        Some(&footer),
                    );
                    match result {
                        select::SelectResult::Selected(v) => {
                            config.segments.context.style = v;
                            return StepResult::Next;
                        }
                        select::SelectResult::Back => {
                            config.segments.context = orig;
                            sub = 0;
                        }
                        select::SelectResult::Cancelled => {
                            config.segments.context = orig;
                            return StepResult::Cancelled;
                        }
                    }
                }
            }
            2 => {
                let orig = config.segments.context.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("context"), t("prompt.barChar"));
                let opts = bar_char_options();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&config.segments.context.bar_char),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.context.bar_char = v.to_string();
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        config.segments.context.bar_char = v;
                        sub = 3;
                    }
                    select::SelectResult::Back => {
                        config.segments.context = orig;
                        sub = 1;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.context = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            3 => {
                let orig = config.segments.context.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("context"), t("prompt.barLength"));
                let opts = bar_length_options();
                let current = config.segments.context.bar_length.to_string();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&current),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        if let Ok(n) = v.parse::<u32>() {
                            cfg.segments.context.bar_length = n;
                            preview::update_preview_in_place(cfg, PREVIEW_ROW);
                        }
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        if let Ok(n) = v.parse::<u32>() {
                            config.segments.context.bar_length = n;
                        }
                        return StepResult::Next;
                    }
                    select::SelectResult::Back => {
                        config.segments.context = orig;
                        sub = 2;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.context = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            _ => return StepResult::Next,
        }
    }
}

// ── Crypto configuration ────────────────────────────────────────────────

fn configure_crypto(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
    step_label: &str,
) -> StepResult {
    // Sub-steps: 0=coins, 1=style, 2=refresh
    let mut sub: usize = 0;
    loop {
        match sub {
            0 => {
                let orig = config.segments.crypto.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("crypto"), t("prompt.selectCoins"));
                let opts: Vec<multiselect::MultiselectOption> = CRYPTO_LIST
                    .iter()
                    .map(|(sym, name, _pair)| multiselect::MultiselectOption {
                        value: sym.to_string(),
                        label: format!("{} {}", sym, name),
                        hint: None,
                    })
                    .collect();
                let initial = config.segments.crypto.coins.clone();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = multiselect::multiselect(
                    &prompt,
                    &opts,
                    &initial,
                    true,
                    &mut |selected: &[String]| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.crypto.coins = selected.to_vec();
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    multiselect::MultiselectResult::Selected(selected) => {
                        config.segments.crypto.coins = selected;
                        sub = 1;
                    }
                    multiselect::MultiselectResult::Back => {
                        config.segments.crypto = orig;
                        return StepResult::Back;
                    }
                    multiselect::MultiselectResult::Cancelled => {
                        config.segments.crypto = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            1 => {
                let orig = config.segments.crypto.clone();
                show_screen(config, steps, 0, step_label);
                let prompt = format!("{} — {}", seg_label("crypto"), t("prompt.style"));
                let opts = style_options();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&config.segments.crypto.style),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        cfg.segments.crypto.style = v.to_string();
                        preview::update_preview_in_place(cfg, PREVIEW_ROW);
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        config.segments.crypto.style = v;
                        sub = 2;
                    }
                    select::SelectResult::Back => {
                        config.segments.crypto = orig;
                        sub = 0;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.crypto = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            2 => {
                let orig = config.segments.crypto.clone();
                show_screen(config, steps, 0, step_label);
                let prompt =
                    format!("{} — {}", seg_label("crypto"), t("prompt.refreshInterval"));
                let opts = refresh_interval_options();
                let current = config.segments.crypto.refresh_interval.to_string();
                let config_ptr = config as *mut Config;
                let footer = step_progress::render_pending_footer(steps, 0);
                let result = select::select(
                    &prompt,
                    &opts,
                    Some(&current),
                    &mut |v: &str| {
                        let cfg = unsafe { &mut *config_ptr };
                        if let Ok(n) = v.parse::<u64>() {
                            cfg.segments.crypto.refresh_interval = n;
                            preview::update_preview_in_place(cfg, PREVIEW_ROW);
                        }
                    },
                    Some(&footer),
                );
                match result {
                    select::SelectResult::Selected(v) => {
                        if let Ok(n) = v.parse::<u64>() {
                            config.segments.crypto.refresh_interval = n;
                        }
                        return StepResult::Next;
                    }
                    select::SelectResult::Back => {
                        config.segments.crypto = orig;
                        sub = 1;
                    }
                    select::SelectResult::Cancelled => {
                        config.segments.crypto = orig;
                        return StepResult::Cancelled;
                    }
                }
            }
            _ => return StepResult::Next,
        }
    }
}
