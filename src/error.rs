//! Soplang error types with Somali messages.
//! Phase 1: structured errors with codes, spans, and hints.

use std::fmt;
use std::path::PathBuf;

use colored::Colorize;

// ---------------------------------------------------------------------------
// Error codes (machine-readable, documented)
// ---------------------------------------------------------------------------

/// Error code catalog. Use in error messages for tests and documentation.
pub mod codes {
    // Lexer (E001–E009)
    pub const E001_UNEXPECTED_CHAR: &str = "E001";
    pub const E002_UNTERMINATED_STRING: &str = "E002";
    pub const E003_UNTERMINATED_COMMENT: &str = "E003";

    // Parser (E010–E019)
    pub const E010_UNEXPECTED_TOKEN: &str = "E010";
    pub const E011_INVALID_NUMBER: &str = "E011";
    pub const E012_EXPECTED_NAME: &str = "E012";

    // Type / Semantic (E020–E029)
    pub const E020_REDECLARED: &str = "E020";
    pub const E021_UNDECLARED: &str = "E021";
    pub const E022_TYPE_MISMATCH: &str = "E022";

    // Runtime (E030–E039)
    pub const E030_DIVISION_BY_ZERO: &str = "E030";
    pub const E031_INDEX_OUT_OF_BOUNDS: &str = "E031";
    pub const E032_BREAK_OUTSIDE_LOOP: &str = "E032";
    pub const E033_NOT_A_FUNCTION: &str = "E033";
    pub const E034_UNKNOWN_METHOD: &str = "E034";
}

/// Optional metadata for an error (code, span end, hint, file).
#[derive(Debug, Clone, Default)]
pub struct ErrorMeta {
    pub code:   Option<&'static str>,
    pub end_line: Option<usize>,
    pub end_col: Option<usize>,
    pub hint:   Option<String>,
    pub file:   Option<PathBuf>,
}

impl ErrorMeta {
    pub fn with_code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }
    pub fn with_span(mut self, end_line: usize, end_col: usize) -> Self {
        self.end_line = Some(end_line);
        self.end_col = Some(end_col);
        self
    }
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }
}

// ---------------------------------------------------------------------------
// Main error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum SoplangError {
    Lexer {
        msg:  String,
        line: usize,
        col:  usize,
        meta: ErrorMeta,
    },
    Parser {
        msg:  String,
        line: usize,
        col:  usize,
        meta: ErrorMeta,
    },
    Runtime {
        msg:  String,
        line: usize,
        col:  usize,
        meta: ErrorMeta,
    },
    Type {
        msg:  String,
        line: usize,
        col:  usize,
        meta: ErrorMeta,
    },
    Import {
        msg:  String,
        line: usize,
        col:  usize,
        meta: ErrorMeta,
    },
}

impl fmt::Display for SoplangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, msg, line, col, code) = match self {
            SoplangError::Lexer { msg, line, col, meta } => ("lexer", msg, line, col, meta.code),
            SoplangError::Parser { msg, line, col, meta } => ("parser", msg, line, col, meta.code),
            SoplangError::Runtime { msg, line, col, meta } => ("runtime", msg, line, col, meta.code),
            SoplangError::Type { msg, line, col, meta } => ("nooc", msg, line, col, meta.code),
            SoplangError::Import { msg, line, col, meta } => ("import", msg, line, col, meta.code),
        };
        let code_str = code.map(|c| format!(" [{}]", c)).unwrap_or_default();
        write!(f, "Khalad {}: {}{} sadar {}, goobta {}", kind, msg, code_str, line, col)
    }
}

impl std::error::Error for SoplangError {}

impl SoplangError {
    /// Line and column for source snippet (1-based for display).
    pub fn location(&self) -> (usize, usize) {
        match self {
            SoplangError::Lexer { line, col, .. }
            | SoplangError::Parser { line, col, .. }
            | SoplangError::Runtime { line, col, .. }
            | SoplangError::Type { line, col, .. }
            | SoplangError::Import { line, col, .. } => (*line, *col),
        }
    }

    pub fn code(&self) -> Option<&'static str> {
        match self {
            SoplangError::Lexer { meta, .. }
            | SoplangError::Parser { meta, .. }
            | SoplangError::Runtime { meta, .. }
            | SoplangError::Type { meta, .. }
            | SoplangError::Import { meta, .. } => meta.code,
        }
    }

    pub fn hint(&self) -> Option<&str> {
        match self {
            SoplangError::Lexer { meta, .. }
            | SoplangError::Parser { meta, .. }
            | SoplangError::Runtime { meta, .. }
            | SoplangError::Type { meta, .. }
            | SoplangError::Import { meta, .. } => meta.hint.as_deref(),
        }
    }

    pub fn meta(&self) -> &ErrorMeta {
        match self {
            SoplangError::Lexer { meta, .. }
            | SoplangError::Parser { meta, .. }
            | SoplangError::Runtime { meta, .. }
            | SoplangError::Type { meta, .. }
            | SoplangError::Import { meta, .. } => meta,
        }
    }
}

// ---------------------------------------------------------------------------
// Formatting with source context
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Constructors (backward-compatible + extended)
// ---------------------------------------------------------------------------

/// Build a lexer error.
pub fn lexer_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Lexer {
        msg:  msg.into(),
        line,
        col,
        meta: ErrorMeta::default(),
    }
}

/// Build a lexer error with optional metadata.
pub fn lexer_error_ex(msg: impl Into<String>, line: usize, col: usize, meta: ErrorMeta) -> SoplangError {
    SoplangError::Lexer {
        msg:  msg.into(),
        line,
        col,
        meta,
    }
}

/// Build a parser error.
pub fn parser_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Parser {
        msg:  msg.into(),
        line,
        col,
        meta: ErrorMeta::default(),
    }
}

/// Build a parser error with optional metadata.
pub fn parser_error_ex(msg: impl Into<String>, line: usize, col: usize, meta: ErrorMeta) -> SoplangError {
    SoplangError::Parser {
        msg:  msg.into(),
        line,
        col,
        meta,
    }
}

/// Build a runtime error.
pub fn runtime_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Runtime {
        msg:  msg.into(),
        line,
        col,
        meta: ErrorMeta::default(),
    }
}

/// Build a runtime error with optional metadata.
pub fn runtime_error_ex(msg: impl Into<String>, line: usize, col: usize, meta: ErrorMeta) -> SoplangError {
    SoplangError::Runtime {
        msg:  msg.into(),
        line,
        col,
        meta,
    }
}

/// Build a type error.
pub fn type_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Type {
        msg:  msg.into(),
        line,
        col,
        meta: ErrorMeta::default(),
    }
}

/// Build a type error with optional metadata.
pub fn type_error_ex(msg: impl Into<String>, line: usize, col: usize, meta: ErrorMeta) -> SoplangError {
    SoplangError::Type {
        msg:  msg.into(),
        line,
        col,
        meta,
    }
}
