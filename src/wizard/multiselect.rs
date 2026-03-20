use super::terminal::{self, Key};
use std::collections::HashSet;
use std::io::{Write, stdout};

// ── Result & Option types ──────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum MultiselectResult {
    Selected(Vec<String>), // Enter or Right arrow
    Back,                  // Left arrow
    Cancelled,             // Esc or Ctrl+C
}

pub struct MultiselectOption {
    pub value: String,
    pub label: String,
    pub hint: Option<String>,
}

// ── ANSI helpers ───────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const BAR: &str = "│";
const CORNER_TL: &str = "┌";
const CORNER_BL: &str = "└";

// ── Drawing ────────────────────────────────────────────────────────────────

/// Render the multiselect UI into a byte buffer.
///
/// Layout (clack-style):
/// ```text
///   ┌ Select segments to enable
///   │  › ◼ Model       Opus4.6         ← cursor here, checked
///   │    ◼ Cost        $0.42           ← checked
///   │    ◻ Usage       ████░░░░ 25%    ← unchecked
///   │    ◼ Context     ██████░░░░ 60%  ← checked
///   └
/// ```
pub fn draw_multiselect(
    message: &str,
    options: &[MultiselectOption],
    cursor: usize,
    selected: &HashSet<String>,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    // Top: ┌ message
    let _ = write!(buf, "  {CYAN}{CORNER_TL}{RESET} {message}\r\n");

    for (i, opt) in options.iter().enumerate() {
        let cursor_indicator = if i == cursor { "›" } else { " " };
        let check_marker = if selected.contains(&opt.value) {
            "◼"
        } else {
            "◻"
        };
        let hint_str = opt
            .hint
            .as_deref()
            .map_or(String::new(), |h| format!("  {h}"));

        if i == cursor {
            let _ = write!(
                buf,
                "  {CYAN}{BAR}{RESET}  {cursor_indicator} \x1b[1m{check_marker} {}{hint_str}{RESET}\r\n",
                opt.label
            );
        } else {
            let _ = write!(
                buf,
                "  {CYAN}{BAR}{RESET}  {cursor_indicator} {DIM}{check_marker} {}{hint_str}{RESET}\r\n",
                opt.label
            );
        }
    }

    // Bottom: └
    let _ = write!(buf, "  {CYAN}{CORNER_BL}{RESET}");

    buf
}

/// Erase previous render and redraw.
fn redraw(
    message: &str,
    options: &[MultiselectOption],
    cursor: usize,
    selected: &HashSet<String>,
) {
    let mut out = stdout();
    // Move cursor up by (options.len() + 2) lines to cover header + options + footer
    let lines_up = options.len() as u16 + 2;
    let _ = write!(out, "\x1b[{lines_up}A\r");
    // Clear from cursor to end of screen
    let _ = write!(out, "\x1b[J");
    let rendered = draw_multiselect(message, options, cursor, selected);
    let _ = out.write_all(&rendered);
    let _ = out.flush();
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Multi-select with live preview.
///
/// `on_change` is called whenever the selected set changes (Space toggle).
/// Returns [`MultiselectResult`].
pub fn multiselect(
    message: &str,
    options: &[MultiselectOption],
    initial_values: &[String],
    required: bool,
    on_change: &mut dyn FnMut(&[String]),
) -> MultiselectResult {
    if options.is_empty() {
        return MultiselectResult::Cancelled;
    }

    // 1. Initialize cursor and selected set from initial_values
    let mut idx: usize = 0;
    let mut selected: HashSet<String> = initial_values.iter().cloned().collect();

    // 2. Initial render
    let mut out = stdout();
    let rendered = draw_multiselect(message, options, idx, &selected);
    let _ = out.write_all(&rendered);
    let _ = out.flush();

    // 3. Enter raw mode + hide cursor
    terminal::enable_raw_mode();
    terminal::hide_cursor();

    // 4. Key loop
    let result = loop {
        match terminal::read_key() {
            Key::Up => {
                if idx > 0 {
                    idx -= 1;
                    redraw(message, options, idx, &selected);
                }
            }
            Key::Down => {
                if idx < options.len() - 1 {
                    idx += 1;
                    redraw(message, options, idx, &selected);
                }
            }
            Key::Space => {
                let value = &options[idx].value;
                if selected.contains(value) {
                    selected.remove(value);
                } else {
                    selected.insert(value.clone());
                }
                let selected_vec: Vec<String> = options
                    .iter()
                    .filter(|o| selected.contains(&o.value))
                    .map(|o| o.value.clone())
                    .collect();
                on_change(&selected_vec);
                redraw(message, options, idx, &selected);
            }
            Key::Enter | Key::Right => {
                if required && selected.is_empty() {
                    // Don't accept empty selection when required
                    continue;
                }
                let selected_vec: Vec<String> = options
                    .iter()
                    .filter(|o| selected.contains(&o.value))
                    .map(|o| o.value.clone())
                    .collect();
                break MultiselectResult::Selected(selected_vec);
            }
            Key::Left => {
                break MultiselectResult::Back;
            }
            Key::Escape | Key::CtrlC => {
                break MultiselectResult::Cancelled;
            }
            _ => {}
        }
    };

    // 5. Restore terminal state
    terminal::show_cursor();
    terminal::disable_raw_mode();

    // Final newline so the caller starts on a clean line
    let _ = writeln!(stdout());

    result
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiselect_result_variants() {
        let sel = MultiselectResult::Selected(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            sel,
            MultiselectResult::Selected(vec!["a".to_string(), "b".to_string()])
        );
        assert_ne!(sel, MultiselectResult::Back);
        assert_ne!(sel, MultiselectResult::Cancelled);

        // Back and Cancelled are distinct
        assert_eq!(MultiselectResult::Back, MultiselectResult::Back);
        assert_eq!(MultiselectResult::Cancelled, MultiselectResult::Cancelled);
        assert_ne!(MultiselectResult::Back, MultiselectResult::Cancelled);

        // Empty selected vec
        let empty = MultiselectResult::Selected(vec![]);
        assert_eq!(empty, MultiselectResult::Selected(vec![]));
    }

    #[test]
    fn test_multiselect_option_creation() {
        let opt = MultiselectOption {
            value: "model".to_string(),
            label: "Model".to_string(),
            hint: None,
        };
        assert_eq!(opt.value, "model");
        assert_eq!(opt.label, "Model");
        assert!(opt.hint.is_none());

        let opt_hint = MultiselectOption {
            value: "cost".to_string(),
            label: "Cost".to_string(),
            hint: Some("$0.42".to_string()),
        };
        assert_eq!(opt_hint.hint.as_deref(), Some("$0.42"));
    }

    #[test]
    fn test_draw_multiselect_checked() {
        let options = vec![
            MultiselectOption {
                value: "a".into(),
                label: "Alpha".into(),
                hint: None,
            },
            MultiselectOption {
                value: "b".into(),
                label: "Beta".into(),
                hint: Some("recommended".into()),
            },
            MultiselectOption {
                value: "c".into(),
                label: "Gamma".into(),
                hint: None,
            },
        ];

        let mut selected = HashSet::new();
        selected.insert("a".to_string());
        selected.insert("c".to_string());

        let buf = draw_multiselect("Pick items", &options, 0, &selected);
        let output = String::from_utf8(buf).unwrap();

        // Header
        assert!(output.contains("Pick items"));

        // Checked items show ◼
        assert!(output.contains("◼ Alpha"));
        assert!(output.contains("◼ Gamma"));

        // Unchecked item shows ◻
        assert!(output.contains("◻ Beta"));

        // Hint appears
        assert!(output.contains("recommended"));

        // Box-drawing characters
        assert!(output.contains("┌"));
        assert!(output.contains("│"));
        assert!(output.contains("└"));
    }

    #[test]
    fn test_draw_multiselect_cursor() {
        let options = vec![
            MultiselectOption {
                value: "a".into(),
                label: "Alpha".into(),
                hint: None,
            },
            MultiselectOption {
                value: "b".into(),
                label: "Beta".into(),
                hint: None,
            },
            MultiselectOption {
                value: "c".into(),
                label: "Gamma".into(),
                hint: None,
            },
        ];

        let selected = HashSet::new();

        // Cursor at index 1
        let buf = draw_multiselect("Test", &options, 1, &selected);
        let output = String::from_utf8(buf).unwrap();

        // Split into lines to check cursor position
        let lines: Vec<&str> = output.split("\r\n").collect();

        // lines[0] = header, lines[1] = Alpha (no cursor), lines[2] = Beta (cursor), lines[3] = Gamma (no cursor)
        // Alpha line should NOT have cursor indicator ›
        assert!(
            !lines[1].contains('›'),
            "Alpha line should not have cursor indicator ›"
        );
        // Beta line should have cursor indicator › (with ANSI codes in between)
        assert!(
            lines[2].contains('›') && lines[2].contains("◻ Beta"),
            "Beta line should have cursor indicator › and ◻ Beta"
        );
        // Gamma line should NOT have cursor indicator ›
        assert!(
            !lines[3].contains('›'),
            "Gamma line should not have cursor indicator ›"
        );
    }

    #[test]
    fn test_multiselect_empty_returns_cancelled() {
        let options: Vec<MultiselectOption> = vec![];
        let result = multiselect("Empty", &options, &[], false, &mut |_| {});
        assert_eq!(result, MultiselectResult::Cancelled);
    }
}
