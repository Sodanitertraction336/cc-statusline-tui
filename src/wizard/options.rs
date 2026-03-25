//! Option builders for wizard select prompts.
//!
//! Each function returns a `Vec<SelectOption>` representing the available
//! choices for a particular configuration field (style, bar character, etc.).

use super::select;
use crate::i18n::t;

pub(super) fn style_options() -> Vec<select::SelectOption> {
    vec![
        select::SelectOption {
            value: "ultrathink".into(),
            label: t("style.rainbow").into(),
            hint: None,
        },
        select::SelectOption {
            value: "cyan".into(),
            label: t("style.cyan").into(),
            hint: None,
        },
        select::SelectOption {
            value: "green".into(),
            label: t("style.green").into(),
            hint: None,
        },
        select::SelectOption {
            value: "blue".into(),
            label: t("style.blue").into(),
            hint: None,
        },
        select::SelectOption {
            value: "yellow".into(),
            label: t("style.yellow").into(),
            hint: None,
        },
        select::SelectOption {
            value: "magenta".into(),
            label: t("style.magenta").into(),
            hint: None,
        },
        select::SelectOption {
            value: "red".into(),
            label: t("style.red").into(),
            hint: None,
        },
        select::SelectOption {
            value: "white".into(),
            label: t("style.white").into(),
            hint: None,
        },
        select::SelectOption {
            value: "orange".into(),
            label: t("style.orange").into(),
            hint: None,
        },
        select::SelectOption {
            value: "pink".into(),
            label: t("style.pink").into(),
            hint: None,
        },
        select::SelectOption {
            value: "purple".into(),
            label: t("style.purple").into(),
            hint: None,
        },
    ]
}

pub(super) fn bar_style_options() -> Vec<select::SelectOption> {
    vec![
        select::SelectOption {
            value: "ultrathink-gradient".into(),
            label: t("barStyle.gradient").into(),
            hint: None,
        },
        select::SelectOption {
            value: "semantic".into(),
            label: t("barStyle.trafficLight").into(),
            hint: Some(t("barStyle.trafficLightHint").into()),
        },
        select::SelectOption {
            value: "cyan".into(),
            label: t("style.cyan").into(),
            hint: None,
        },
        select::SelectOption {
            value: "green".into(),
            label: t("style.green").into(),
            hint: None,
        },
        select::SelectOption {
            value: "blue".into(),
            label: t("style.blue").into(),
            hint: None,
        },
        select::SelectOption {
            value: "yellow".into(),
            label: t("style.yellow").into(),
            hint: None,
        },
        select::SelectOption {
            value: "magenta".into(),
            label: t("style.magenta").into(),
            hint: None,
        },
        select::SelectOption {
            value: "orange".into(),
            label: t("style.orange").into(),
            hint: None,
        },
        select::SelectOption {
            value: "pink".into(),
            label: t("style.pink").into(),
            hint: None,
        },
        select::SelectOption {
            value: "purple".into(),
            label: t("style.purple").into(),
            hint: None,
        },
    ]
}

pub(super) fn bar_char_options() -> Vec<select::SelectOption> {
    vec![
        select::SelectOption {
            value: "shade".into(),
            label: t("char.shade").into(),
            hint: None,
        },
        select::SelectOption {
            value: "full-block".into(),
            label: t("char.fullBlock").into(),
            hint: None,
        },
        select::SelectOption {
            value: "rectangle".into(),
            label: t("char.rectangle").into(),
            hint: None,
        },
    ]
}

pub(super) fn bar_length_options() -> Vec<select::SelectOption> {
    vec![
        select::SelectOption {
            value: "6".into(),
            label: format!("6 {}", t("unit.chars")),
            hint: None,
        },
        select::SelectOption {
            value: "8".into(),
            label: format!("8 {}", t("unit.chars")),
            hint: None,
        },
        select::SelectOption {
            value: "10".into(),
            label: format!("10 {}", t("unit.chars")),
            hint: None,
        },
        select::SelectOption {
            value: "12".into(),
            label: format!("12 {}", t("unit.chars")),
            hint: None,
        },
        select::SelectOption {
            value: "16".into(),
            label: format!("16 {}", t("unit.chars")),
            hint: None,
        },
    ]
}

pub(super) fn refresh_interval_options() -> Vec<select::SelectOption> {
    vec![
        select::SelectOption {
            value: "30".into(),
            label: format!("30{}", t("unit.seconds")),
            hint: None,
        },
        select::SelectOption {
            value: "60".into(),
            label: format!("60{}", t("unit.seconds")),
            hint: None,
        },
        select::SelectOption {
            value: "120".into(),
            label: format!("120{}", t("unit.seconds")),
            hint: None,
        },
        select::SelectOption {
            value: "180".into(),
            label: format!("180{}", t("unit.seconds")),
            hint: None,
        },
        select::SelectOption {
            value: "240".into(),
            label: format!("240{}", t("unit.seconds")),
            hint: None,
        },
        select::SelectOption {
            value: "300".into(),
            label: format!("300{}", t("unit.seconds")),
            hint: None,
        },
    ]
}

pub(super) fn max_length_options() -> Vec<select::SelectOption> {
    vec![
        select::SelectOption {
            value: "10".into(),
            label: format!("10 {}", t("unit.chars")),
            hint: None,
        },
        select::SelectOption {
            value: "15".into(),
            label: format!("15 {}", t("unit.chars")),
            hint: None,
        },
        select::SelectOption {
            value: "20".into(),
            label: format!("20 {}", t("unit.chars")),
            hint: None,
        },
        select::SelectOption {
            value: "30".into(),
            label: format!("30 {}", t("unit.chars")),
            hint: None,
        },
    ]
}

pub(super) fn icon_options() -> Vec<select::SelectOption> {
    vec![
        select::SelectOption {
            value: "\u{1f525}".into(),
            label: "\u{1f525} Fire".into(),
            hint: None,
        },
        select::SelectOption {
            value: "\u{1f98a}".into(),
            label: "\u{1f98a} Fox".into(),
            hint: None,
        },
        select::SelectOption {
            value: "\u{1f422}".into(),
            label: "\u{1f422} Turtle".into(),
            hint: None,
        },
        select::SelectOption {
            value: "\u{1f430}".into(),
            label: "\u{1f430} Rabbit".into(),
            hint: None,
        },
        select::SelectOption {
            value: "".into(),
            label: "(none)".into(),
            hint: None,
        },
    ]
}
