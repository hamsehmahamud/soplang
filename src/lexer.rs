//! Character-by-character lexer for Soplang source.
//! Matches psrc/core/lexer.py behaviour.

use std::iter::Peekable;
use std::str::Chars;

use crate::error::{codes, lexer_error_ex, ErrorMeta, SoplangError};
use crate::token::{Token, TokenType};

pub struct Lexer<'a> {
    #[allow(dead_code)] // kept for future error reporting (source lines)
    source:  &'a str,
    chars:   Peekable<Chars<'a>>,
    line:    usize,
    col:     usize,
    current: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars().peekable();
        let current = chars.next();
        Self {
            source,
            chars,
            line: 1,
            col: 1,
            current,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, SoplangError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenType::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, SoplangError> {
        while let Some(c) = self.current {
            if c.is_whitespace() {
                self.skip_whitespace();
                continue;
            }

            // Comments
            if c == '/' {
                if self.skip_comment()? {
                    continue;
                }
            }

            let line = self.line;
            let col = self.col;

            // Identifier or keyword (letter or _ first)
            if c.is_alphabetic() || c == '_' {
                return Ok(self.read_identifier(line, col));
            }

            // Number
            if c.is_ascii_digit() {
                return Ok(self.read_number(line, col));
            }

            // String
            if c == '"' || c == '\'' {
                return self.read_string(c, line, col);
            }

            // Two-char operators (must be before single-char)
            if c == '=' {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token::new(TokenType::EqEq, "==", line, col));
                }
                return Ok(Token::new(TokenType::Assign, "=", line, col));
            }
            if c == '>' {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token::new(TokenType::GreaterEq, ">=", line, col));
                }
                return Ok(Token::new(TokenType::Greater, ">", line, col));
            }
            if c == '<' {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token::new(TokenType::LessEq, "<=", line, col));
                }
                return Ok(Token::new(TokenType::Less, "<", line, col));
            }
            if c == '!' {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token::new(TokenType::NotEq, "!=", line, col));
                }
                return Ok(Token::new(TokenType::Not, "!", line, col));
            }
            if c == '&' {
                self.advance();
                if self.current == Some('&') {
                    self.advance();
                    return Ok(Token::new(TokenType::And, "&&", line, col));
                }
                return Err(lexer_error_ex(
                    format!("Xaraf aan la filayn: {}", c),
                    line,
                    col,
                    ErrorMeta::default().with_code(codes::E001_UNEXPECTED_CHAR),
                ));
            }
            if c == '|' {
                self.advance();
                if self.current == Some('|') {
                    self.advance();
                    return Ok(Token::new(TokenType::Or, "||", line, col));
                }
                return Err(lexer_error_ex(
                    format!("Xaraf aan la filayn: {}", c),
                    line,
                    col,
                    ErrorMeta::default().with_code(codes::E001_UNEXPECTED_CHAR),
                ));
            }

            // Single-char tokens
            let (kind, lexeme) = match c {
                '+' => (TokenType::Plus, "+"),
                '-' => (TokenType::Minus, "-"),
                '*' => (TokenType::Star, "*"),
                '/' => (TokenType::Slash, "/"),
                '%' => (TokenType::Modulo, "%"),
                '(' => (TokenType::LParen, "("),
                ')' => (TokenType::RParen, ")"),
                '{' => (TokenType::LBrace, "{"),
                '}' => (TokenType::RBrace, "}"),
                '[' => (TokenType::LBracket, "["),
                ']' => (TokenType::RBracket, "]"),
                ',' => (TokenType::Comma, ","),
                ':' => (TokenType::Colon, ":"),
                ';' => (TokenType::Semicolon, ";"),
                '.' => (TokenType::Dot, "."),
                _ => {
                    return Err(lexer_error_ex(
                        format!("Xaraf aan la filayn: {}", c),
                        line,
                        col,
                        ErrorMeta::default().with_code(codes::E001_UNEXPECTED_CHAR),
                    ));
                }
            };
            self.advance();
            return Ok(Token::new(kind, lexeme, line, col));
        }

        Ok(Token::new(TokenType::Eof, "", self.line, self.col))
    }

    fn advance(&mut self) {
        if self.current == Some('\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        self.current = self.chars.next();
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn skip_whitespace(&mut self) {
        while self.current.map_or(false, |c| c.is_whitespace()) {
            self.advance();
        }
    }

    /// Returns true if a comment was skipped.
    fn skip_comment(&mut self) -> Result<bool, SoplangError> {
        if self.peek() != Some('/') && self.peek() != Some('*') {
            return Ok(false);
        }
        if self.peek() == Some('/') {
            // Line comment
            self.advance(); // /
            self.advance(); // /
            while self.current.is_some() && self.current != Some('\n') {
                self.advance();
            }
            if self.current == Some('\n') {
                self.advance();
            }
            return Ok(true);
        }
        // Block comment /* ... */
        self.advance(); // /
        self.advance(); // *
        while self.current.is_some() {
            if self.current == Some('*') && self.peek() == Some('/') {
                self.advance(); // *
                self.advance(); // /
                return Ok(true);
            }
            self.advance();
        }
        Err(lexer_error_ex(
            "Faallo aan la dhammaystirin",
            self.line,
            self.col,
            ErrorMeta::default().with_code(codes::E003_UNTERMINATED_COMMENT),
        ))
    }

    fn read_identifier(&mut self, start_line: usize, start_col: usize) -> Token {
        let mut s = String::new();
        while self
            .current
            .map_or(false, |c| c.is_alphanumeric() || c == '_')
        {
            s.push(self.current.unwrap());
            self.advance();
        }
        let kind = keyword(&s).unwrap_or(TokenType::Identifier);
        Token::new(kind, s, start_line, start_col)
    }

    fn read_number(&mut self, start_line: usize, start_col: usize) -> Token {
        let mut s = String::new();
        while self
            .current
            .map_or(false, |c| c.is_ascii_digit() || c == '.')
        {
            s.push(self.current.unwrap());
            self.advance();
        }
        Token::new(TokenType::Number, s, start_line, start_col)
    }

    fn read_string(
        &mut self,
        quote: char,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, SoplangError> {
        self.advance(); // opening quote
        let mut s = String::new();
        while self.current.is_some() && self.current != Some(quote) {
            match self.current {
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
                None => break,
            }
        }
        if self.current == Some(quote) {
            self.advance(); // closing quote
            Ok(Token::new(TokenType::String, s, start_line, start_col))
        } else {
            Err(lexer_error_ex(
                "Qoraal aan la dhammaystirin",
                self.line,
                self.col,
                ErrorMeta::default().with_code(codes::E002_UNTERMINATED_STRING),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_hello() {
        let source = r#"qor("Salaan, Adduunka!")"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 5); // qor, (, string, ), eof
        assert_eq!(tokens[0].kind, TokenType::Qor);
        assert_eq!(tokens[1].kind, TokenType::LParen);
        assert_eq!(tokens[2].kind, TokenType::String);
        assert_eq!(tokens[2].lexeme, "Salaan, Adduunka!");
        assert_eq!(tokens[3].kind, TokenType::RParen);
        assert_eq!(tokens[4].kind, TokenType::Eof);
    }

    #[test]
    fn tokenize_keywords_and_operators() {
        let source = "door x = 1 + 2";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenType::Door);
        assert_eq!(tokens[1].kind, TokenType::Identifier);
        assert_eq!(tokens[2].kind, TokenType::Assign);
        assert_eq!(tokens[3].kind, TokenType::Number);
        assert_eq!(tokens[4].kind, TokenType::Plus);
        assert_eq!(tokens[5].kind, TokenType::Number);
    }

    #[test]
    fn tokenize_comments() {
        let source = "// skip\nqor(1)";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenType::Qor);
        assert_eq!(tokens[1].kind, TokenType::LParen);
        assert_eq!(tokens[2].kind, TokenType::Number);
        assert_eq!(tokens[2].lexeme, "1");
    }

    #[test]
    fn unterminated_string_error() {
        let source = r#"qor("unclosed"#;
        let mut lexer = Lexer::new(source);
        let res = lexer.tokenize();
        assert!(res.is_err());
    }
}

fn keyword(s: &str) -> Option<TokenType> {
    match s {
        "door" => Some(TokenType::Door),
        "madoor" => Some(TokenType::Madoor),
        "hawl" => Some(TokenType::Hawl),
        "celi" => Some(TokenType::Celi),
        "qor" => Some(TokenType::Qor),
        "gelin" => Some(TokenType::Gelin),
        "haddii" => Some(TokenType::Haddii),
        "haddii_kale" => Some(TokenType::HaddiiKale),
        "ugudambeyn" => Some(TokenType::Ugudambeyn),
        "dooro" => Some(TokenType::Dooro),
        "xaalad" => Some(TokenType::Xaalad),
        "kuceli" => Some(TokenType::Kuceli),
        "intay" => Some(TokenType::Intay),
        "jooji" => Some(TokenType::Jooji),
        "soco" => Some(TokenType::Soco),
        "isku_day" => Some(TokenType::IskuDay),
        "qabo" => Some(TokenType::Qabo),
        "ka_keen" => Some(TokenType::KaKeen),
        "fasalka" => Some(TokenType::Fasalka),
        "ka_dhaxal" => Some(TokenType::KaDhaxal),
        "cusub" => Some(TokenType::Cusub),
        "nafta" => Some(TokenType::Nafta),
        "abn" => Some(TokenType::Abn),
        "jajab" => Some(TokenType::Jajab),
        "qoraal" => Some(TokenType::Qoraal),
        "bool" => Some(TokenType::Bool),
        "teed" => Some(TokenType::Teed),
        "walax" => Some(TokenType::Walax),
        "run" => Some(TokenType::True),
        "been" => Some(TokenType::False),
        "null" => Some(TokenType::Null),
        _ => None,
    }
}
