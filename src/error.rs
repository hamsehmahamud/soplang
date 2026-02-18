//! Soplang error types with Somali messages.
//! Matches psrc/utils/errors.py (LexerError, format_error).

use std::fmt;

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

/// Build a parser error at the given source location.
pub fn parser_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Parser {
        msg:  msg.into(),
        line,
        col,
    }
}
