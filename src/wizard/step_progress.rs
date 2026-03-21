//! Vertical step progress indicator for the wizard.
//!
//! Renders a @clack/prompts-style progress display with three states:
//! - Completed: green `●` with optional summary text
//! - Current: bright cyan `◆` (rendered by the prompt component, not here)
//! - Pending: dim `○`
//!
//! Split into two functions for the wizard's layout needs:
//! - `render_completed_steps(steps, idx)` -- printed above the current prompt
//! - `render_pending_footer(steps, idx)` -- appended below the prompt options
//!
//! `render_step_progress()` renders the full indicator (kept for tests).

// ── Step progress indicator ──────────────────────────────────────────

/// Metadata for a single step in the wizard progress bar.
#[derive(Clone)]
pub struct StepInfo {
    pub label: String,
    pub summary: Option<String>,
}

// ── ANSI color constants ────────────────────────────────────────────

const GREEN: &str = "\x1b[32m";
// Used by render_step_progress (test-only now) — keep for backward compat
#[allow(dead_code)]
const BRIGHT_CYAN: &str = "\x1b[1;36m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

// ── Public API ──────────────────────────────────────────────────────

/// Render a single step line with icon, color, and label.
///
/// Example output: `  ● 1/4 Select segments`
pub fn render_step_line(icon: &str, label: &str, color: &str) -> String {
    format!("  {}{} {}{}", color, icon, label, RESET)
}

/// Render the full vertical step progress indicator.
///
/// ```text
///   ● 1/4 Select segments        ← completed (dim green)
///   │   Segments: Model, Cost
///   │
///   ◆ 2/4 Configure (1/5)        ← current (bright cyan)
///   │
///   ○ 3/4 Reorder                ← pending (dim)
///   │
///   ○ 4/4 Confirm
/// ```
///
/// Note: This function is kept for backward compatibility and tests.
/// The wizard now uses `render_completed_steps` + `render_pending_footer` instead.
#[allow(dead_code)]
pub fn render_step_progress(steps: &[StepInfo], current_idx: usize) -> String {
    let total = steps.len();
    let mut lines: Vec<String> = Vec::new();

    for (i, step) in steps.iter().enumerate() {
        if i < current_idx {
            // Completed step: dim green ●
            lines.push(render_step_line(
                "\u{25CF}",
                &step.label,
                GREEN,
            ));
            // Show summary if present
            if let Some(ref summary) = step.summary {
                lines.push(format!("  {}│   {}{}", DIM, summary, RESET));
            }
            // Connector line
            lines.push(format!("  {}│{}", DIM, RESET));
        } else if i == current_idx {
            // Current step: bright cyan ◆
            lines.push(render_step_line(
                "\u{25C6}",
                &step.label,
                BRIGHT_CYAN,
            ));
            // Connector line (unless last step)
            if i < total - 1 {
                lines.push(format!("  {}│{}", DIM, RESET));
            }
        } else {
            // Pending step: dim ○
            lines.push(render_step_line(
                "\u{25CB}",
                &step.label,
                DIM,
            ));
            // Connector line (unless last step)
            if i < total - 1 {
                lines.push(format!("  {}│{}", DIM, RESET));
            }
        }
    }

    lines.join("\n")
}

/// Render only completed steps (indices < current_idx).
///
/// Returns empty string if current_idx == 0.
/// Uses `\n` line separators (printed in normal mode via `print!`).
/// Includes trailing connector line `│\n` after each completed step.
pub fn render_completed_steps(steps: &[StepInfo], current_idx: usize) -> String {
    if current_idx == 0 {
        return String::new();
    }

    let mut lines: Vec<String> = Vec::new();

    for (i, step) in steps.iter().enumerate() {
        if i >= current_idx {
            break;
        }
        // Completed step: green ●
        lines.push(render_step_line("\u{25CF}", &step.label, GREEN));
        // Show summary if present
        if let Some(ref summary) = step.summary {
            lines.push(format!("  {}│   {}{}", DIM, summary, RESET));
        }
        // Connector line
        lines.push(format!("  {}│{}", DIM, RESET));
    }

    // Join with \n and add trailing \n so print! ends on a new line
    format!("{}\n", lines.join("\n"))
}

/// Render pending steps (indices > current_idx) as footer text.
///
/// Uses `\r\n` line separators for raw-mode compatibility.
/// Returns empty string if current_idx is the last step.
///
/// Format:
/// ```text
/// \r\n  DIM│RESET\r\n  DIM○ labelRESET\r\n  DIM│RESET\r\n  DIM○ labelRESET
/// ```
/// (starts with `\r\n` connector, NO trailing `\r\n` on last line)
pub fn render_pending_footer(steps: &[StepInfo], current_idx: usize) -> String {
    let total = steps.len();
    if current_idx >= total - 1 {
        return String::new();
    }

    let mut parts: Vec<String> = Vec::new();

    for step in &steps[(current_idx + 1)..total] {
        // Connector line before each pending step
        parts.push(format!("\r\n  {}│{}", DIM, RESET));
        // Pending step: dim ○
        parts.push(format!(
            "\r\n  {}\u{25CB} {}{}",
            DIM, step.label, RESET
        ));
    }

    parts.join("")
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_steps() -> Vec<StepInfo> {
        vec![
            StepInfo {
                label: "1/4 Select segments".into(),
                summary: None,
            },
            StepInfo {
                label: "2/4 Configure".into(),
                summary: None,
            },
            StepInfo {
                label: "3/4 Reorder".into(),
                summary: None,
            },
            StepInfo {
                label: "4/4 Confirm".into(),
                summary: None,
            },
        ]
    }

    #[test]
    fn test_step_info_creation() {
        let step = StepInfo {
            label: "Select segments".into(),
            summary: Some("Model, Cost".into()),
        };
        assert_eq!(step.label, "Select segments");
        assert_eq!(step.summary.as_deref(), Some("Model, Cost"));

        let step2 = step.clone();
        assert_eq!(step2.label, "Select segments");
    }

    #[test]
    fn test_render_step_progress_first() {
        let steps = make_steps();
        let result = render_step_progress(&steps, 0);

        // First step is current (◆)
        assert!(result.contains('\u{25C6}'));
        assert!(result.contains("1/4 Select segments"));

        // Remaining steps are pending (○)
        assert!(result.contains('\u{25CB}'));
        assert!(result.contains("2/4 Configure"));
        assert!(result.contains("3/4 Reorder"));
        assert!(result.contains("4/4 Confirm"));

        // No completed steps (●)
        assert!(!result.contains('\u{25CF}'));
    }

    #[test]
    fn test_render_step_progress_middle() {
        let mut steps = make_steps();
        steps[0].summary = Some("Segments: Model, Cost".into());
        steps[1].summary = Some("Done".into());
        let result = render_step_progress(&steps, 2);

        // First two are completed (●)
        let completed_count = result.matches('\u{25CF}').count();
        assert_eq!(completed_count, 2);

        // Third is current (◆)
        assert!(result.contains('\u{25C6}'));
        assert!(result.contains("3/4 Reorder"));

        // Fourth is pending (○)
        assert!(result.contains('\u{25CB}'));
        assert!(result.contains("4/4 Confirm"));
    }

    #[test]
    fn test_render_step_progress_with_summary() {
        let mut steps = make_steps();
        steps[0].summary = Some("Segments: Model, Cost".into());
        let result = render_step_progress(&steps, 1);

        // Completed step should show summary
        assert!(result.contains("Segments: Model, Cost"));
        // First step is completed (●)
        assert!(result.contains('\u{25CF}'));
        // Second step is current (◆)
        assert!(result.contains('\u{25C6}'));
        assert!(result.contains("2/4 Configure"));
    }

    #[test]
    fn test_render_step_progress_last() {
        let mut steps = make_steps();
        steps[0].summary = Some("Done".into());
        steps[1].summary = Some("Done".into());
        steps[2].summary = Some("Done".into());
        let result = render_step_progress(&steps, 3);

        // First three are completed (●)
        let completed_count = result.matches('\u{25CF}').count();
        assert_eq!(completed_count, 3);

        // Last is current (◆)
        assert!(result.contains('\u{25C6}'));
        assert!(result.contains("4/4 Confirm"));

        // No pending steps (○)
        assert!(!result.contains('\u{25CB}'));
    }

    // ── render_completed_steps tests ──────────────────────────────────

    #[test]
    fn test_render_completed_steps_at_first() {
        let steps = make_steps();
        let result = render_completed_steps(&steps, 0);
        // No completed steps when current_idx == 0
        assert!(result.is_empty());
    }

    #[test]
    fn test_render_completed_steps_at_middle() {
        let mut steps = make_steps();
        steps[0].summary = Some("Segments: Model, Cost".into());
        let result = render_completed_steps(&steps, 2);

        // Should contain two completed steps
        let completed_count = result.matches('\u{25CF}').count();
        assert_eq!(completed_count, 2);

        // Should contain summary of first step
        assert!(result.contains("Segments: Model, Cost"));

        // Should NOT contain current or pending steps
        assert!(!result.contains('\u{25C6}'));
        assert!(!result.contains('\u{25CB}'));

        // Should end with a newline
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_render_completed_steps_at_last() {
        let mut steps = make_steps();
        steps[0].summary = Some("Done".into());
        steps[1].summary = Some("Done".into());
        steps[2].summary = Some("Done".into());
        let result = render_completed_steps(&steps, 3);

        // Three completed steps
        let completed_count = result.matches('\u{25CF}').count();
        assert_eq!(completed_count, 3);

        // Should contain connector lines
        assert!(result.contains("│"));
    }

    // ── render_pending_footer tests ──────────────────────────────────

    #[test]
    fn test_render_pending_footer_at_first() {
        let steps = make_steps();
        let result = render_pending_footer(&steps, 0);

        // Should contain three pending steps
        let pending_count = result.matches('\u{25CB}').count();
        assert_eq!(pending_count, 3);

        // Should use \r\n
        assert!(result.contains("\r\n"));

        // Should contain all pending step labels
        assert!(result.contains("2/4 Configure"));
        assert!(result.contains("3/4 Reorder"));
        assert!(result.contains("4/4 Confirm"));

        // Should start with \r\n (connector from last option)
        assert!(result.starts_with("\r\n"));

        // Should NOT end with \r\n
        assert!(!result.ends_with("\r\n"));
    }

    #[test]
    fn test_render_pending_footer_at_middle() {
        let steps = make_steps();
        let result = render_pending_footer(&steps, 2);

        // Should contain one pending step
        let pending_count = result.matches('\u{25CB}').count();
        assert_eq!(pending_count, 1);

        assert!(result.contains("4/4 Confirm"));
        assert!(!result.contains("3/4 Reorder"));
    }

    #[test]
    fn test_render_pending_footer_at_last() {
        let steps = make_steps();
        let result = render_pending_footer(&steps, 3);

        // No pending steps when at last step
        assert!(result.is_empty());
    }
}
