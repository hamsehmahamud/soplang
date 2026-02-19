//! Soplang error types with Somali messages.
//! Matches psrc/utils/errors.py (LexerError, format_error).
//! Phase 6: coloured error output with source snippet.

use std::fmt;

use colored::Colorize;

#[derive(Debug)]
#[allow(dead_code)] // Parser, Runtime, Type, Import used in Phase 2+
pub enum SoplangError {
    Lexer {
        msg:  String,
        line: usize,
        col:  usize,
    },
    Parser {
        msg:  String,
        line: usize,
        col:  usize,
    },
    Runtime {
        msg:  String,
        line: usize,
        col:  usize,
    },
    Type {
        msg:  String,
        line: usize,
        col:  usize,
    },
    Import {
        msg:  String,
        line: usize,
        col:  usize,
    },
}

impl fmt::Display for SoplangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SoplangError::Lexer { msg, line, col } => {
                write!(f, "Khalad lexer: {} sadar {}, goobta {}", msg, line, col)
            }
            SoplangError::Parser { msg, line, col } => {
                write!(f, "Khalad parser: {} sadar {}, goobta {}", msg, line, col)
            }
            SoplangError::Runtime { msg, line, col } => {
                write!(f, "Khalad runtime: {} sadar {}, goobta {}", msg, line, col)
            }
            SoplangError::Type { msg, line, col } => {
                write!(f, "Khalad nooc: {} sadar {}, goobta {}", msg, line, col)
            }
            SoplangError::Import { msg, line, col } => {
                write!(f, "Khalad import: {} sadar {}, goobta {}", msg, line, col)
            }
        }
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
}

/// Format error with optional source context (line + caret). Phase 6.
pub fn format_error_with_source(err: &SoplangError, source: Option<&str>) -> String {
    let (line, col) = err.location();
    let header = format!("{}", err);
    let mut out = format!("{}\n", header.red().bold());
    if let Some(s) = source {
        let lines: Vec<&str> = s.lines().collect();
        let line_1based = line.max(1);
        if line_1based > 0 && line_1based <= lines.len() {
            let line_content = lines[line_1based - 1];
            let col_1based = col.max(1);
            let pad = " ".repeat((col_1based - 1).min(line_content.len()));
            out.push_str(&format!("{:4} │   {}\n", line_1based, line_content));
            out.push_str(&format!("     │   {}\n", format!("{}^^^", pad).red()));
        }
    }
    out
}

/// Build a parser error at the given source location.
pub fn parser_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Parser {
        msg:  msg.into(),
        line,
        col,
    }
}

/// Build a runtime error at the given source location.
pub fn runtime_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Runtime {
        msg:  msg.into(),
        line,
        col,
    }
}

/// Build a type error at the given source location.
pub fn type_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Type {
        msg:  msg.into(),
        line,
        col,
    }
}
