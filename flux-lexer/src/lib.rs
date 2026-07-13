use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Print,
    Def,
    Return,
    If,
    Elif,
    Else,
    For,
    In,
    While,
    Class,
    Init,
    Import,
    From,
    As,
    Try,
    Except,
    Finally,
    Raise,
    Async,
    Await,
    True,
    False,
    None,
    And,
    Or,
    Not,
    Range,
    Break,
    Continue,
    Pass,
    SelfKw,
    Type,
    Ask,

    // Literals
    Identifier(String),
    String(String),
    Integer(i64),
    Float(f64),

    // Operators
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    FloorDiv,
    Percent,
    StarStar,
    EqEq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Colon,
    Comma,
    Dot,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Structure
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Debug, Error)]
pub enum LexError {
    #[error("unexpected character '{ch}' at line {line}, column {column}")]
    UnexpectedCharacter { ch: char, line: usize, column: usize },

    #[error("unterminated string at line {line}, column {column}")]
    UnterminatedString { line: usize, column: usize },

    #[error("inconsistent indentation at line {line}, column {column}")]
    InconsistentIndentation { line: usize, column: usize },

    #[error("invalid number literal '{value}' at line {line}, column {column}")]
    InvalidNumber {
        value: String,
        line: usize,
        column: usize,
    },
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

struct Lexer<'a> {
    source: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    pending_indents: Vec<Token>,
    at_line_start: bool,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
            pending_indents: Vec::new(),
            at_line_start: true,
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while self.pos < self.chars.len() || !self.pending_indents.is_empty() {
            if let Some(token) = self.pending_indents.pop() {
                tokens.push(token);
                continue;
            }

            if self.at_line_start {
                self.handle_indentation(&mut tokens)?;
                // handle_indentation may have queued DEDENT tokens (e.g. when
                // the file ends inside an indented block). Loop back to the
                // top so the `pending_indents.pop()` branch above drains them
                // before we consider stopping — otherwise those DEDENTs would
                // be silently dropped when `pos` has already reached EOF.
                continue;
            }

            self.skip_whitespace();

            if self.pos >= self.chars.len() {
                break;
            }

            if self.chars[self.pos] == '\n' {
                tokens.push(self.make_token(TokenKind::Newline, "\n".to_string()));
                self.advance_char();
                self.at_line_start = true;
                continue;
            }

            if self.chars[self.pos] == '#' {
                self.skip_comment();
                continue;
            }

            let token = self.next_token()?;
            self.at_line_start = false;
            tokens.push(token);
        }

        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(self.make_token(TokenKind::Dedent, "".to_string()));
        }

        tokens.push(self.make_token(TokenKind::Eof, "".to_string()));
        Ok(tokens)
    }

fn handle_indentation(&mut self, tokens: &mut Vec<Token>) -> Result<(), LexError> {
    let mut indent = 0;
    while self.pos < self.chars.len() {
        match self.chars[self.pos] {
            ' ' => {
                indent += 1;
                self.advance_char();
            }
            '\t' => {
                indent += 4;
                self.advance_char();
            }
            '\r' => {
                self.advance_char();
                continue;
            }
            '\n' => {
                self.advance_char();
                indent = 0;
                continue;
            }
            '#' => {
                self.skip_comment();
                if self.pos < self.chars.len() && self.chars[self.pos] == '\n' {
                    self.advance_char();
                    indent = 0;
                }
                continue;
            }
            _ => break,
        }
    }

    let current = *self.indent_stack.last().unwrap();
    if indent > current {
        self.indent_stack.push(indent);
        tokens.push(self.make_token(TokenKind::Indent, "".to_string()));
    } else if indent < current {
        while self.indent_stack.len() > 1 && *self.indent_stack.last().unwrap() > indent {
            self.indent_stack.pop();
            self.pending_indents.push(self.make_token(TokenKind::Dedent, "".to_string()));
        }
        if *self.indent_stack.last().unwrap() != indent {
            return Err(LexError::InconsistentIndentation {
                line: self.line,
                column: self.column,
            });
        }
    }

    self.at_line_start = false;
    Ok(())
}

    fn next_token(&mut self) -> Result<Token, LexError> {
        let ch = self.chars[self.pos];
        let start_line = self.line;
        let start_col = self.column;

        match ch {
            '(' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::LParen, "(", start_line, start_col))
            }
            ')' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::RParen, ")", start_line, start_col))
            }
            '[' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::LBracket, "[", start_line, start_col))
            }
            ']' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::RBracket, "]", start_line, start_col))
            }
            '{' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::LBrace, "{", start_line, start_col))
            }
            '}' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::RBrace, "}", start_line, start_col))
            }
            ':' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::Colon, ":", start_line, start_col))
            }
            ',' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::Comma, ",", start_line, start_col))
            }
            '.' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::Dot, ".", start_line, start_col))
            }
            '+' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::Plus, "+", start_line, start_col))
            }
            '-' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::Minus, "-", start_line, start_col))
            }
            '*' => {
                if self.peek() == Some('*') {
                    self.advance_char();
                    self.advance_char();
                    Ok(self.token_at(TokenKind::StarStar, "**", start_line, start_col))
                } else {
                    self.advance_char();
                    Ok(self.token_at(TokenKind::Star, "*", start_line, start_col))
                }
            }
            '/' => {
                if self.peek() == Some('/') {
                    self.advance_char();
                    self.advance_char();
                    Ok(self.token_at(TokenKind::FloorDiv, "//", start_line, start_col))
                } else {
                    self.advance_char();
                    Ok(self.token_at(TokenKind::Slash, "/", start_line, start_col))
                }
            }
            '%' => {
                self.advance_char();
                Ok(self.token_at(TokenKind::Percent, "%", start_line, start_col))
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance_char();
                    self.advance_char();
                    Ok(self.token_at(TokenKind::EqEq, "==", start_line, start_col))
                } else {
                    self.advance_char();
                    Ok(self.token_at(TokenKind::Assign, "=", start_line, start_col))
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance_char();
                    self.advance_char();
                    Ok(self.token_at(TokenKind::NotEq, "!=", start_line, start_col))
                } else {
                    self.advance_char();
                    Err(LexError::UnexpectedCharacter {
                        ch: '!',
                        line: start_line,
                        column: start_col,
                    })
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance_char();
                    self.advance_char();
                    Ok(self.token_at(TokenKind::LtEq, "<=", start_line, start_col))
                } else {
                    self.advance_char();
                    Ok(self.token_at(TokenKind::Lt, "<", start_line, start_col))
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance_char();
                    self.advance_char();
                    Ok(self.token_at(TokenKind::GtEq, ">=", start_line, start_col))
                } else {
                    self.advance_char();
                    Ok(self.token_at(TokenKind::Gt, ">", start_line, start_col))
                }
            }
            '"' | '\'' => self.string_literal(ch, start_line, start_col),
            c if c.is_ascii_digit() => self.number_literal(start_line, start_col),
            c if c.is_ascii_alphabetic() || c == '_' => self.identifier_or_keyword(start_line, start_col),
            _ => Err(LexError::UnexpectedCharacter {
                ch,
                line: start_line,
                column: start_col,
            }),
        }
    }

    fn string_literal(&mut self, quote: char, start_line: usize, start_col: usize) -> Result<Token, LexError> {
        self.advance_char();
        let mut value = String::new();

        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch == quote {
                self.advance_char();
                return Ok(self.token_at(TokenKind::String(value.clone()), &format!("{quote}{value}{quote}"), start_line, start_col));
            }
            if ch == '\\' {
                self.advance_char();
                if self.pos >= self.chars.len() {
                    break;
                }
                let escaped = match self.chars[self.pos] {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    other => other,
                };
                value.push(escaped);
                self.advance_char();
                continue;
            }
            value.push(ch);
            self.advance_char();
        }

        Err(LexError::UnterminatedString {
            line: start_line,
            column: start_col,
        })
    }

    fn number_literal(&mut self, start_line: usize, start_col: usize) -> Result<Token, LexError> {
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.advance_char();
        }

        let mut is_float = false;
        if self.pos < self.chars.len() && self.chars[self.pos] == '.' {
            if self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                is_float = true;
                self.advance_char();
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                    self.advance_char();
                }
            }
        }

        let lexeme: String = self.chars[start..self.pos].iter().collect();
        if is_float {
            let value: f64 = lexeme.parse().map_err(|_| LexError::InvalidNumber {
                value: lexeme.clone(),
                line: start_line,
                column: start_col,
            })?;
            Ok(self.token_at(TokenKind::Float(value), &lexeme, start_line, start_col))
        } else {
            let value: i64 = lexeme.parse().map_err(|_| LexError::InvalidNumber {
                value: lexeme.clone(),
                line: start_line,
                column: start_col,
            })?;
            Ok(self.token_at(TokenKind::Integer(value), &lexeme, start_line, start_col))
        }
    }

    fn identifier_or_keyword(&mut self, start_line: usize, start_col: usize) -> Result<Token, LexError> {
        let start = self.pos;
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance_char();
            } else {
                break;
            }
        }

        let lexeme: String = self.chars[start..self.pos].iter().collect();
        let upper = lexeme.to_uppercase();
        let kind = match upper.as_str() {
            "PRINT" => TokenKind::Print,
            "DEF" => TokenKind::Def,
            "RETURN" => TokenKind::Return,
            "IF" => TokenKind::If,
            "ELIF" => TokenKind::Elif,
            "ELSE" => TokenKind::Else,
            "FOR" => TokenKind::For,
            "IN" => TokenKind::In,
            "WHILE" => TokenKind::While,
            "CLASS" => TokenKind::Class,
            "INIT" => TokenKind::Init,
            "IMPORT" => TokenKind::Import,
            "FROM" => TokenKind::From,
            "AS" => TokenKind::As,
            "TRY" => TokenKind::Try,
            "EXCEPT" => TokenKind::Except,
            "FINALLY" => TokenKind::Finally,
            "RAISE" => TokenKind::Raise,
            "ASYNC" => TokenKind::Async,
            "AWAIT" => TokenKind::Await,
            "TRUE" => TokenKind::True,
            "FALSE" => TokenKind::False,
            "NONE" => TokenKind::None,
            "AND" => TokenKind::And,
            "OR" => TokenKind::Or,
            "NOT" => TokenKind::Not,
            "RANGE" => TokenKind::Range,
            "BREAK" => TokenKind::Break,
            "CONTINUE" => TokenKind::Continue,
            "PASS" => TokenKind::Pass,
            "SELF" => TokenKind::SelfKw,
            "TYPE" => TokenKind::Type,
            "ASK" => TokenKind::Ask,
            _ => TokenKind::Identifier(lexeme.clone()),
        };

        Ok(self.token_at(kind, &lexeme, start_line, start_col))
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                ' ' | '\r' | '\t' => self.advance_char(),
                _ => break,
            }
        }
    }

    fn skip_comment(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
            self.advance_char();
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos + 1 < self.chars.len() {
            Some(self.chars[self.pos + 1])
        } else {
            None
        }
    }

    fn advance_char(&mut self) {
        if self.pos < self.chars.len() {
            if self.chars[self.pos] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
    }

    fn make_token(&self, kind: TokenKind, lexeme: String) -> Token {
        Token {
            kind,
            lexeme,
            line: self.line,
            column: self.column,
        }
    }

    fn token_at(&self, kind: TokenKind, lexeme: &str, line: usize, column: usize) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            line,
            column,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_hello_world() {
        let tokens = lex(r#"PRINT "Hello World""#).unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Print));
        assert!(matches!(tokens[1].kind, TokenKind::String(_)));
    }

    #[test]
    fn lexes_indentation() {
        let source = "IF TRUE:\n    PRINT 1\n";
        let tokens = lex(source).unwrap();
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Indent)));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Dedent)));
    }

    /// A blank CRLF line ("\r\n" with no leading spaces) inside an
    /// indented block used to be misread as a dedent all the way back to
    /// column 0, closing the block early. Two statements at the same
    /// indent level separated by a blank line must stay in the same
    /// block: exactly one Indent/Dedent pair for the whole block, not one
    /// per statement.
    #[test]
    fn blank_crlf_line_does_not_dedent() {
        let source = "IF TRUE:\r\n    PRINT 1\r\n\r\n    PRINT 2\r\n";
        let tokens = lex(source).unwrap();
        let indents = tokens.iter().filter(|t| matches!(t.kind, TokenKind::Indent)).count();
        let dedents = tokens.iter().filter(|t| matches!(t.kind, TokenKind::Dedent)).count();
        assert_eq!(indents, 1);
        assert_eq!(dedents, 1);
    }
}
