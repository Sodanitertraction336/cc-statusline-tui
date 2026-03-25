//! Interactive TUI wizard -- 3-step configuration flow.
//!
//! Launched when the binary is run without arguments. Guides the user through:
//! 1. Module menu (pick a module, configure it, assign to a row -- repeat)
//! 2. Reorder segments within each row
//! 3. Confirm and save
//!
//! Uses snapshot-based back navigation: each step's config is saved before
//! entry, restored on Back. The UI follows a @clack/prompts-style layout
//! with a step progress indicator (completed/current/pending).
//!
//! Submodules provide reusable TUI components:
//! - `select` / `multiselect` / `confirm` -- input prompts
//! - `terminal` -- crossterm abstraction (raw mode, cursor, key reading)
//! - `spinner` -- braille loading animation
//! - `preview` -- live statusline preview using sample data
//! - `step_progress` -- vertical step indicator
//!
//! Extracted sub-modules:
//! - `helpers` -- segment label/hint lookups and row manipulation
//! - `options` -- `SelectOption` builders for style/bar/icon pickers
//! - `presets` -- named preset configurations
//! - `configure` -- per-segment configuration flows

pub mod confirm;
mod configure;
mod helpers;
pub mod multiselect;
mod options;
pub mod preview;
mod presets;
pub mod select;
pub mod spinner;
pub mod step_progress;
pub mod terminal;

use crate::config::Config;
use crate::i18n::{self, t, tf, SUPPORTED_LANGS};

use helpers::{find_segment_row, seg_hint, seg_label, t_row};
use presets::build_preset;

pub(super) const PREVIEW_ROW: u16 = 3;

/// All seven segment keys in canonical order.
pub(super) const ALL_SEGMENTS: &[&str] = &[
    "model", "cost", "usage", "usage_7d", "path", "git", "context", "crypto",
];

// ── Step result ─────────────────────────────────────────────────────────────

pub(super) enum StepResult {
    Next,
    Back,
    Cancelled,
}

// ── Public entry point ──────────────────────────────────────────────────────

pub fn run() {
    // Install panic hook to restore terminal state on crash
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Show);
        default_hook(info);
    }));

    // Preflight check
    if !dirs::home_dir()
        .map(|h| h.join(".claude").exists())
        .unwrap_or(false)
    {
        eprintln!("{}", t("msg.noClaudeCode"));
        eprintln!("{}", t("msg.installClaudeCode"));
        std::process::exit(1);
    }

    // Load existing config (if any) and prepare default config
    let has_existing = crate::config::config_path().exists();
    let existing_config = crate::config::load_config();
    let mut config = Config::default();

    // Step 0: Language selection (always shown, previous choice as default)
    let default_lang = if !existing_config.lang.is_empty()
        && SUPPORTED_LANGS.contains(&existing_config.lang.as_str())
    {
        existing_config.lang.as_str()
    } else {
        "en"
    };
    let lang_opts = vec![
        select::SelectOption {
            value: "en".into(),
            label: "English".into(),
            hint: None,
        },
        select::SelectOption {
            value: "zh".into(),
            label: "\u{4e2d}\u{6587}".into(),
            hint: None,
        },
        select::SelectOption {
            value: "ja".into(),
            label: "\u{65e5}\u{672c}\u{8a9e}".into(),
            hint: None,
        },
        select::SelectOption {
            value: "ko".into(),
            label: "\u{d55c}\u{ad6d}\u{c5b4}".into(),
            hint: None,
        },
        select::SelectOption {
            value: "es".into(),
            label: "Espa\u{f1}ol".into(),
            hint: None,
        },
        select::SelectOption {
            value: "pt".into(),
            label: "Portugu\u{ea}s".into(),
            hint: None,
        },
        select::SelectOption {
            value: "ru".into(),
            label: "\u{420}\u{443}\u{441}\u{441}\u{43a}\u{438}\u{439}".into(),
            hint: None,
        },
    ];
    match select::select(
        "Language / \u{8bed}\u{8a00}",
        &lang_opts,
        Some(default_lang),
        &mut |_| {},
        None,
    ) {
        select::SelectResult::Selected(v) => {
            config.lang = v.clone();
            i18n::set_lang(&v);
        }
        _ => {
            std::process::exit(0);
        }
    }

    // Step 0b: Mode selection — presets + existing + custom, flat list
    // Preview updates live as user navigates between options
    loop {
        // Show preview with the initially selected preset (Developer)
        let initial_preview = build_preset(&config.lang, "developer");
        show_header(&initial_preview, t("step.start"));

        // Build flat option list: 5 presets [+ existing] + custom
        let mut mode_opts = vec![
            select::SelectOption {
                value: "minimal".into(),
                label: t("preset.minimal").into(),
                hint: Some(t("preset.minimalHint").into()),
            },
            select::SelectOption {
                value: "developer".into(),
                label: t("preset.developer").into(),
                hint: Some(t("preset.developerHint").into()),
            },
            select::SelectOption {
                value: "dashboard".into(),
                label: t("preset.dashboard").into(),
                hint: Some(t("preset.dashboardHint").into()),
            },
            select::SelectOption {
                value: "rainbow".into(),
                label: t("preset.rainbow").into(),
                hint: Some(t("preset.rainbowHint").into()),
            },
            select::SelectOption {
                value: "crypto".into(),
                label: t("preset.crypto").into(),
                hint: Some(t("preset.cryptoHint").into()),
            },
        ];
        if has_existing {
            mode_opts.push(select::SelectOption {
                value: "existing".into(),
                label: t("mode.existing").into(),
                hint: Some(t("mode.existingHint").into()),
            });
        }
        mode_opts.push(select::SelectOption {
            value: "custom".into(),
            label: t("mode.custom").into(),
            hint: Some(t("mode.customHint").into()),
        });

        let existing_ref = &existing_config;
        let lang = config.lang.clone();
        match select::select(
            t("mode.prompt"),
            &mode_opts,
            Some("developer"),
            &mut |v: &str| match v {
                "existing" => {
                    preview::update_preview_in_place(existing_ref, PREVIEW_ROW);
                }
                "custom" => {
                    let default_cfg = Config {
                        lang: lang.clone(),
                        ..Config::default()
                    };
                    preview::update_preview_in_place(&default_cfg, PREVIEW_ROW);
                }
                preset_name => {
                    let preview_cfg = build_preset(&lang, preset_name);
                    preview::update_preview_in_place(&preview_cfg, PREVIEW_ROW);
                }
            },
            None,
        ) {
            select::SelectResult::Selected(ref v) if v == "existing" => {
                show_header(&existing_config, t("step.confirm"));
                match confirm::confirm(t("prompt.saveExisting"), true, None) {
                    confirm::ConfirmResult::Yes => {
                        do_save(&existing_config);
                        return;
                    }
                    confirm::ConfirmResult::Cancelled => std::process::exit(0),
                    _ => continue,
                }
            }
            select::SelectResult::Selected(ref v) if v == "custom" => {
                config.rows = vec![vec![], vec![], vec![]]; // Initialize 3 empty rows
                break; // custom -> 3-step wizard
            }
            select::SelectResult::Selected(preset_name) => {
                // Preset selected — confirm and save
                let preset_cfg = build_preset(&config.lang, &preset_name);
                show_header(&preset_cfg, t("step.confirm"));
                match confirm::confirm(t("prompt.save"), true, None) {
                    confirm::ConfirmResult::Yes => {
                        do_save(&preset_cfg);
                        return;
                    }
                    confirm::ConfirmResult::Cancelled => std::process::exit(0),
                    _ => continue,
                }
            }
            _ => std::process::exit(0),
        }
    }

    // Define the 3 major steps
    let mut steps = vec![
        step_progress::StepInfo {
            label: t("step.modules").to_string(),
            summary: None,
        },
        step_progress::StepInfo {
            label: t("step.reorder").to_string(),
            summary: None,
        },
        step_progress::StepInfo {
            label: t("step.confirm").to_string(),
            summary: None,
        },
    ];

    let mut current_step: usize = 0;
    let mut snapshots: Vec<Config> = Vec::new();

    loop {
        // Save snapshot before step
        if snapshots.len() <= current_step {
            snapshots.push(config.clone());
        }

        let result = match current_step {
            0 => step_module_menu(&mut config, &mut steps),
            1 => step_reorder(&mut config, &mut steps),
            2 => step_confirm(&config, &steps),
            _ => break,
        };

        match result {
            StepResult::Next => {
                current_step += 1;
                // Auto-skip reorder step if no row has >1 segment
                if current_step == 1 && !config.rows.iter().any(|r| r.len() > 1) {
                    steps[1].summary = Some("\u{2714}".to_string());
                    current_step = 2;
                }
                if current_step > 2 {
                    do_save(&config);
                    return;
                }
            }
            StepResult::Back => {
                current_step = current_step.saturating_sub(1);
                // Auto-skip reorder step backward if nothing to reorder
                if current_step == 1 && !config.rows.iter().any(|r| r.len() > 1) {
                    current_step = 0;
                }
                if let Some(snapshot) = snapshots.get(current_step) {
                    config = snapshot.clone();
                }
                snapshots.truncate(current_step);
                steps[current_step].summary = None;
            }
            StepResult::Cancelled => {
                eprintln!("{}", t("msg.cancelled"));
                std::process::exit(0);
            }
        }
    }
}

// ── UI helpers ──────────────────────────────────────────────────────────────

fn show_header(config: &Config, step_label: &str) {
    terminal::clear_screen();
    println!();
    println!(
        "  \x1b[1mClaude Statusline Configurator\x1b[0m \x1b[2m\u{2014} {}\x1b[0m",
        step_label
    );
    println!("  \x1b[2m{}\x1b[0m", "\u{2500}".repeat(56));

    // Multi-line preview
    let label = t("msg.preview");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let rows = config.effective_rows();
    let mut has_content = false;

    for (i, row_keys) in rows.iter().enumerate() {
        let parts: Vec<String> = row_keys
            .iter()
            .filter_map(|key| preview::render_sample_segment(key, config, now))
            .collect();
        if !parts.is_empty() {
            if i == 0 {
                println!("  \x1b[2m{}\x1b[0m {}", label, parts.join(" "));
            } else {
                let padding = " ".repeat(label.len());
                println!("  \x1b[2m{}\x1b[0m {}", padding, parts.join(" "));
            }
            has_content = true;
        }
    }

    if !has_content {
        println!("  \x1b[2m{}\x1b[0m", label);
    }

    println!("  \x1b[2m{}\x1b[0m", "\u{2500}".repeat(56));
}

fn show_screen(
    config: &Config,
    steps: &[step_progress::StepInfo],
    current_step: usize,
    step_label: &str,
) {
    show_header(config, step_label);
    let completed = step_progress::render_completed_steps(steps, current_step);
    if !completed.is_empty() {
        print!("{}", completed);
    }
}

fn do_save(config: &Config) {
    let sp = spinner::Spinner::start(t("msg.saving"));
    match crate::install::save_and_apply(config) {
        Ok(()) => {
            sp.stop(t("msg.saved"));
            println!("\n  {}", t("msg.restart"));
        }
        Err(e) => {
            sp.stop(t("msg.saveFailed"));
            eprintln!("  {}", e);
        }
    }
}

// ── Step 1: Module menu ───────────────────────────────────────────────────

fn step_module_menu(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
) -> StepResult {
    // Ensure rows initialized
    if config.rows.len() < 3 {
        config.rows.resize(3, vec![]);
    }

    loop {
        show_screen(config, steps, 0, t("step.modules"));

        // Build menu options
        let mut opts: Vec<select::SelectOption> = Vec::new();
        for &key in ALL_SEGMENTS {
            let row = find_segment_row(&config.rows, key);
            let label = if let Some(r) = row {
                format!(
                    "\x1b[32m\u{2713}\x1b[0m {} ({}) \u{2192} {}",
                    seg_label(key),
                    seg_hint(key),
                    t_row(r)
                )
            } else {
                format!("  {} ({})", seg_label(key), seg_hint(key))
            };
            opts.push(select::SelectOption {
                value: key.into(),
                label,
                hint: None,
            });
        }
        opts.push(select::SelectOption {
            value: "__done__".into(),
            label: t("action.done").into(),
            hint: None,
        });

        let footer = step_progress::render_pending_footer(steps, 0);
        let result = select::select(
            t("prompt.selectModule"),
            &opts,
            None,
            &mut |_: &str| {},
            Some(&footer),
        );

        match result {
            select::SelectResult::Selected(ref v) if v == "__done__" => {
                let has_any = config.rows.iter().any(|r| !r.is_empty());
                if !has_any {
                    // Can't proceed without at least one configured module
                    continue;
                }
                // Build summary
                let configured: Vec<&str> = ALL_SEGMENTS
                    .iter()
                    .filter(|&&k| find_segment_row(&config.rows, k).is_some())
                    .map(|&k| seg_label(k))
                    .collect();
                steps[0].summary = Some(configured.join(", "));
                return StepResult::Next;
            }
            select::SelectResult::Selected(key) => {
                // Enter configure flow for this segment
                configure::configure_segment_with_row(config, steps, &key);
            }
            select::SelectResult::Back => return StepResult::Back,
            select::SelectResult::Cancelled => return StepResult::Cancelled,
        }
    }
}

// ── Step 2: Reorder within rows ─────────────────────────────────────────

fn step_reorder(
    config: &mut Config,
    steps: &mut [step_progress::StepInfo],
) -> StepResult {
    // Find rows that need reordering (>1 segment)
    let rows_to_reorder: Vec<usize> = config
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.len() > 1)
        .map(|(i, _)| i)
        .collect();

    if rows_to_reorder.is_empty() {
        steps[1].summary = Some("\u{2714}".to_string());
        return StepResult::Next;
    }

    let mut idx = 0;
    while idx < rows_to_reorder.len() {
        let r = rows_to_reorder[idx];
        let labels: Vec<&str> = config.rows[r].iter().map(|k| seg_label(k)).collect();

        let prompt = format!(
            "{} {}: {} \u{2014} {}",
            t("row.label"),
            r + 1,
            labels.join(", "),
            t("prompt.reorder"),
        );

        show_screen(config, steps, 1, t("step.reorder"));
        let footer = step_progress::render_pending_footer(steps, 1);
        let result = confirm::confirm(&prompt, false, Some(&footer));

        match result {
            confirm::ConfirmResult::No => {
                idx += 1;
            }
            confirm::ConfirmResult::Yes => {
                let keys = config.rows[r].clone();
                let total = keys.len();
                let mut new_order: Vec<String> = Vec::new();
                let mut remaining = keys.clone();
                let mut pos: usize = 0;

                while pos < total {
                    let pick_label = tf(
                        "step.reorderN",
                        &[&(pos + 1).to_string(), &total.to_string()],
                    );
                    show_screen(config, steps, 1, &pick_label);

                    let prompt = tf("prompt.pickN", &[&(pos + 1).to_string()]);
                    let opts: Vec<select::SelectOption> = remaining
                        .iter()
                        .map(|k| select::SelectOption {
                            value: k.clone(),
                            label: seg_label(k).into(),
                            hint: None,
                        })
                        .collect();

                    let footer = step_progress::render_pending_footer(steps, 1);
                    let result =
                        select::select(&prompt, &opts, None, &mut |_| {}, Some(&footer));

                    match result {
                        select::SelectResult::Selected(v) => {
                            new_order.push(v.clone());
                            remaining.retain(|k| k != &v);
                            pos += 1;
                        }
                        select::SelectResult::Back => {
                            if pos == 0 {
                                // Back from first pick of this row
                                if idx == 0 {
                                    return StepResult::Back;
                                }
                                idx -= 1;
                                break;
                            }
                            pos -= 1;
                            let undone = new_order.pop().unwrap();
                            remaining.insert(0, undone);
                            remaining.sort_by_key(|k| {
                                keys.iter()
                                    .position(|orig| orig == k)
                                    .unwrap_or(usize::MAX)
                            });
                        }
                        select::SelectResult::Cancelled => return StepResult::Cancelled,
                    }
                }

                if new_order.len() == total {
                    config.rows[r] = new_order;
                    idx += 1;
                }
            }
            confirm::ConfirmResult::Back => {
                if idx == 0 {
                    return StepResult::Back;
                }
                idx -= 1;
            }
            confirm::ConfirmResult::Cancelled => return StepResult::Cancelled,
        }
    }

    steps[1].summary = Some("\u{2714}".to_string());
    StepResult::Next
}

// ── Step 3: Confirm ─────────────────────────────────────────────────────

fn step_confirm(
    config: &Config,
    steps: &[step_progress::StepInfo],
) -> StepResult {
    show_screen(config, steps, 2, t("step.confirm"));

    let footer = step_progress::render_pending_footer(steps, 2);
    let result = confirm::confirm(t("prompt.save"), true, Some(&footer));

    match result {
        confirm::ConfirmResult::Yes => StepResult::Next,
        confirm::ConfirmResult::No | confirm::ConfirmResult::Back => StepResult::Back,
        confirm::ConfirmResult::Cancelled => StepResult::Cancelled,
    }
}
