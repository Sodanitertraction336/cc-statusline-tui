//! Preset configuration builder.
//!
//! Provides `build_preset()` which creates a fully-populated `Config` for
//! each named preset (minimal, developer, dashboard, rainbow, crypto).

use crate::config::{
    Config, ContextSegment, CostSegment, CryptoSegment, GitSegment, ModelSegment, PathSegment,
    Segments, UsageSegment,
};

pub(super) fn build_preset(lang: &str, name: &str) -> Config {
    let mut cfg = match name {
        "minimal" => Config {
            rows: vec![
                vec!["model".into(), "cost".into(), "context".into()],
                vec![],
                vec![],
            ],
            order: vec![],
            segments: Segments {
                model: ModelSegment {
                    enabled: true,
                    style: "cyan".into(),
                    icon: "".into(),
                },
                cost: CostSegment {
                    enabled: true,
                    style: "green".into(),
                },
                usage: UsageSegment {
                    enabled: false,
                    ..Default::default()
                },
                path: PathSegment {
                    enabled: false,
                    ..Default::default()
                },
                git: GitSegment {
                    enabled: false,
                    ..Default::default()
                },
                context: ContextSegment {
                    enabled: true,
                    style: "semantic".into(),
                    bar_char: "shade".into(),
                    bar_length: 8,
                    show_bar: true,
                    show_percent: true,
                    show_size: false,
                },
                crypto: CryptoSegment {
                    enabled: false,
                    ..Default::default()
                },
                usage_7d: UsageSegment {
                    enabled: false,
                    ..Default::default()
                },
            },
            ..Default::default()
        },
        "developer" => Config {
            rows: vec![
                vec![
                    "model".into(),
                    "cost".into(),
                    "path".into(),
                    "git".into(),
                    "context".into(),
                ],
                vec![],
                vec![],
            ],
            order: vec![],
            segments: Segments {
                model: ModelSegment {
                    enabled: true,
                    style: "cyan".into(),
                    icon: "".into(),
                },
                cost: CostSegment {
                    enabled: true,
                    style: "green".into(),
                },
                usage: UsageSegment {
                    enabled: false,
                    ..Default::default()
                },
                path: PathSegment {
                    enabled: true,
                    style: "cyan".into(),
                    max_length: 20,
                },
                git: GitSegment {
                    enabled: true,
                    style: "cyan".into(),
                    show_dirty: true,
                    show_remote: true,
                },
                context: ContextSegment {
                    enabled: true,
                    style: "semantic".into(),
                    bar_char: "shade".into(),
                    bar_length: 10,
                    show_bar: true,
                    show_percent: true,
                    show_size: true,
                },
                crypto: CryptoSegment {
                    enabled: false,
                    ..Default::default()
                },
                usage_7d: UsageSegment {
                    enabled: false,
                    ..Default::default()
                },
            },
            ..Default::default()
        },
        "dashboard" => Config {
            rows: vec![
                vec!["model".into(), "cost".into(), "context".into()],
                vec!["usage".into(), "usage_7d".into()],
                vec!["crypto".into()],
            ],
            order: vec![],
            segments: Segments {
                model: ModelSegment {
                    enabled: true,
                    style: "ultrathink".into(),
                    icon: "\u{1f525}".into(),
                },
                cost: CostSegment {
                    enabled: true,
                    style: "green".into(),
                },
                usage: UsageSegment {
                    enabled: true,
                    style: "ultrathink-gradient".into(),
                    bar_char: "shade".into(),
                    bar_length: 8,
                    show_bar: true,
                    show_percent: true,
                    show_reset: true,
                    label: String::new(),
                },
                path: PathSegment {
                    enabled: false,
                    ..Default::default()
                },
                git: GitSegment {
                    enabled: false,
                    ..Default::default()
                },
                context: ContextSegment {
                    enabled: true,
                    style: "ultrathink-gradient".into(),
                    bar_char: "shade".into(),
                    bar_length: 12,
                    show_bar: true,
                    show_percent: true,
                    show_size: true,
                },
                crypto: CryptoSegment {
                    enabled: true,
                    style: "green".into(),
                    refresh_interval: 60,
                    coins: vec!["BTC".into()],
                },
                usage_7d: UsageSegment {
                    enabled: true,
                    style: "ultrathink-gradient".into(),
                    bar_char: "shade".into(),
                    bar_length: 8,
                    show_bar: true,
                    show_percent: true,
                    show_reset: true,
                    label: String::new(),
                },
            },
            ..Default::default()
        },
        "rainbow" => Config {
            rows: vec![
                vec![
                    "model".into(),
                    "cost".into(),
                    "usage".into(),
                    "context".into(),
                    "crypto".into(),
                ],
                vec![],
                vec![],
            ],
            order: vec![],
            segments: Segments {
                model: ModelSegment {
                    enabled: true,
                    style: "ultrathink".into(),
                    icon: "\u{1f525}".into(),
                },
                cost: CostSegment {
                    enabled: true,
                    style: "ultrathink".into(),
                },
                usage: UsageSegment {
                    enabled: true,
                    style: "ultrathink-gradient".into(),
                    bar_char: "full-block".into(),
                    bar_length: 8,
                    show_bar: true,
                    show_percent: true,
                    show_reset: false,
                    label: String::new(),
                },
                path: PathSegment {
                    enabled: false,
                    ..Default::default()
                },
                git: GitSegment {
                    enabled: false,
                    ..Default::default()
                },
                context: ContextSegment {
                    enabled: true,
                    style: "ultrathink-gradient".into(),
                    bar_char: "full-block".into(),
                    bar_length: 12,
                    show_bar: true,
                    show_percent: true,
                    show_size: false,
                },
                crypto: CryptoSegment {
                    enabled: true,
                    style: "ultrathink".into(),
                    refresh_interval: 60,
                    coins: vec!["BTC".into(), "ETH".into()],
                },
                usage_7d: UsageSegment {
                    enabled: false,
                    ..Default::default()
                },
            },
            ..Default::default()
        },
        "crypto" => Config {
            rows: vec![
                vec!["model".into(), "cost".into(), "context".into()],
                vec![],
                vec!["crypto".into()],
            ],
            order: vec![],
            segments: Segments {
                model: ModelSegment {
                    enabled: true,
                    style: "ultrathink".into(),
                    icon: "\u{1f525}".into(),
                },
                cost: CostSegment {
                    enabled: true,
                    style: "green".into(),
                },
                usage: UsageSegment {
                    enabled: false,
                    ..Default::default()
                },
                path: PathSegment {
                    enabled: false,
                    ..Default::default()
                },
                git: GitSegment {
                    enabled: false,
                    ..Default::default()
                },
                context: ContextSegment {
                    enabled: true,
                    style: "semantic".into(),
                    bar_char: "shade".into(),
                    bar_length: 8,
                    show_bar: true,
                    show_percent: true,
                    show_size: false,
                },
                crypto: CryptoSegment {
                    enabled: true,
                    style: "green".into(),
                    refresh_interval: 30,
                    coins: vec!["BTC".into(), "ETH".into(), "SOL".into()],
                },
                usage_7d: UsageSegment {
                    enabled: false,
                    ..Default::default()
                },
            },
            ..Default::default()
        },
        _ => Config::default(),
    };
    cfg.lang = lang.to_string();
    cfg
}
