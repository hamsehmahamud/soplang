//! Format errors with source context (line + caret, hint).

use colored::Colorize;

use super::SoplangError;

/// Format error with optional source context (line + caret). Shows code and hint when present.
pub fn format_error_with_source(err: &SoplangError, source: Option<&str>) -> String {
    let (line, col) = err.location();
    let meta = err.meta();
    let header = format!("{}", err);
    let mut out = format!("{}\n", header.red().bold());
    if let Some(s) = source {
        let lines: Vec<&str> = s.lines().collect();
        let line_1based = line.max(1);
        if line_1based > 0 && line_1based <= lines.len() {
            let line_content = lines[line_1based - 1];
            let col_1based = col.max(1);
            let (pad_len, underline_len) = if let (Some(el), Some(ec)) = (meta.end_line, meta.end_col) {
                let end_1based = el.max(1);
                if end_1based == line_1based && ec >= col_1based {
                    let len = (ec - col_1based + 1).min(line_content.len().saturating_sub(col_1based - 1));
                    (col_1based - 1, len)
                } else {
                    (col_1based - 1, 1)
                }
            } else {
                (col_1based - 1, 1)
            };
            let pad = " ".repeat(pad_len.min(line_content.len()));
            let underline = "^".repeat(underline_len.max(1));
            out.push_str(&format!("{:4} │   {}\n", line_1based, line_content));
            out.push_str(&format!("     │   {}\n", format!("{}{}", pad, underline).red()));
        }
    }
    if let Some(h) = meta.hint.as_deref() {
        out.push_str(&format!("{} {}\n", "     └─".dimmed(), h.dimmed()));
    }
    out
}
