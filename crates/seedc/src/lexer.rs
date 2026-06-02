//! Production‑grade hand‑written lexer for AGENT‑SEED v15.2.
//!
//! Uses a `Peekable<Chars>` iterator and tracks byte positions for
//! `miette::SourceSpan`.  The entry point is `tokenize(source)`.

use crate::token::{keyword_from_str, Token, TokenKind};
use crate::LexError;
use miette::SourceSpan;
use std::iter::Peekable;
use std::str::Chars;

// ── Entry point ──

/// Tokenize a complete source string.  Returns a `Vec<Token>` or a `LexError`
/// describing the first invalid character.
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token()?;
        let is_eof = tok.kind == TokenKind::Eof;
        tokens.push(tok);
        if is_eof {
            break;
        }
    }
    Ok(tokens)
}

// ── Lexer ──

struct Lexer<'a> {
    /// The complete source text (for slicing lexemes).
    source: &'a str,
    /// Remaining source characters iterator.
    chars: Peekable<Chars<'a>>,
    /// Current byte offset from the start of the source.
    pos: usize,
    /// The original source length (for EOF span).
    source_len: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            pos: 0,
            source_len: source.len(),
        }
    }

    // ── Low‑level helpers ──

    /// Return the current byte offset as a `SourceSpan` start.
    fn span_start(&self) -> usize {
        self.pos
    }

    /// Build a `SourceSpan` from a previously recorded start to current `self.pos`.
    fn span(&self, start: usize) -> SourceSpan {
        (start..self.pos).into()
    }

    /// Peek at the next character without consuming it.
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    /// Consume and return the next character.
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Consume characters while `predicate` holds.
    fn take_while(&mut self, pred: impl Fn(char) -> bool) {
        while self.peek().is_some_and(|&c| pred(c)) {
            self.advance();
        }
    }

    /// Skip whitespace and comments.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('/') => {
                    let start = self.pos;
                    self.advance();
                    match self.peek() {
                        Some('/') => {
                            self.advance();
                            self.take_while(|c| c != '\n');
                        }
                        Some('*') => {
                            self.advance();
                            loop {
                                match self.advance() {
                                    None => break, // unterminated — tracked later
                                    Some('*') if self.peek() == Some(&'/') => {
                                        self.advance();
                                        break;
                                    }
                                    _ => continue,
                                }
                            }
                        }
                        _ => {
                            // Not a comment — it's a division operator.
                            // We must NOT consume the leading '/'; unwind.
                            self.pos = start;
                            // Reset chars from the source at the current position
                            let remaining = &self.source[self.pos..];
                            self.chars = remaining.chars().peekable();
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    // ── Token constructors ──

    fn make_token(&self, kind: TokenKind, start: usize) -> Token {
        let end = self.pos;
        let text = if start < self.source.len() && end <= self.source.len() {
            &self.source[start..end]
        } else {
            ""
        };
        Token::new(kind, text, start, end)
    }

    // ── Core tokenizer ──

    fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_trivia();
        let start = self.pos;
        let c = match self.advance() {
            Some(ch) => ch,
            None => {
                return Ok(Token::new(
                    TokenKind::Eof,
                    "",
                    self.source_len,
                    self.source_len,
                ))
            }
        };

        let kind = match c {
            // ── Whitespace (already skipped) ──
            _ if c.is_whitespace() => unreachable!(),

            // ── Identifiers and keywords ──
            'a'..='z' | 'A'..='Z' | '_' => {
                self.take_while(|ch| ch.is_alphanumeric() || ch == '_');
                let text = &self.source[start..self.pos];
                let kind = keyword_from_str(text).unwrap_or(TokenKind::Ident);
                return Ok(Token::new(kind, text, start, self.pos));
            }

            // ── Numbers ──
            '0'..='9' => {
                self.take_while(|ch| ch.is_ascii_digit() || ch == '_');
                let mut is_float = false;
                if self.peek() == Some(&'.') {
                    // Look ahead to distinguish `1.` (float) from `1..` (range)
                    is_float = true;
                    self.advance();
                    self.take_while(|ch| ch.is_ascii_digit() || ch == '_');
                }
                if is_float {
                    TokenKind::FloatLiteral
                } else {
                    TokenKind::IntLiteral
                }
            }

            // ── String literals ──
            '"' => {
                let content_start = self.pos; // after opening quote
                self.take_while(|ch| ch != '"');
                if self.peek().is_none() {
                    return Err(LexError {
                        ch: '"',
                        span: (start..self.pos).into(),
                    });
                }
                let content_end = self.pos;
                self.advance(); // consume closing '"'
                let text = &self.source[content_start..content_end];
                return Ok(Token::new(TokenKind::StringLiteral, text, start, self.pos));
            }

            // ── Character literal ──
            '\'' => {
                self.advance(); // the character
                if self.peek() == Some(&'\'') {
                    self.advance();
                }
                TokenKind::CharLiteral
            }

            // ── Single‑character operators & delimiters ──
            '+' => self.check_compound('=', TokenKind::PlusEq, TokenKind::Plus),
            '-' => self.check_compound('>', TokenKind::Arrow, TokenKind::Minus),
            '*' => self.check_compound('=', TokenKind::StarEq, TokenKind::Star),
            '%' => self.check_compound('=', TokenKind::PercentEq, TokenKind::Percent),
            '!' => self.check_compound('=', TokenKind::NotEq, TokenKind::Not),
            '=' => self.check_compound('=', TokenKind::EqEq, TokenKind::Eq),
            '<' => self.check_two('<', '=', TokenKind::Shl, TokenKind::LtEq, TokenKind::Lt),
            '>' => self.check_two('>', '=', TokenKind::Shr, TokenKind::GtEq, TokenKind::Gt),
            '&' => self.check_compound('&', TokenKind::AndAnd, TokenKind::And),
            // ── The fixed '|' arm ──
            '|' => {
                if self.peek() == Some(&'|') {
                    self.advance();
                    TokenKind::OrOr
                } else if self.peek() == Some(&'>') {
                    self.advance();
                    if self.peek() == Some(&'>') {
                        self.advance();
                        TokenKind::PipeGtGt
                    } else {
                        TokenKind::PipeGt
                    }
                } else {
                    TokenKind::Pipe
                }
            }
            '^' => self.check_compound('=', TokenKind::CaretEq, TokenKind::Caret),
            '~' => TokenKind::Tilde,
            '?' => TokenKind::Question,
            '@' => TokenKind::At,
            '#' => TokenKind::Hash,
            '$' => TokenKind::Dollar,
            '\\' => TokenKind::Backslash,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            '.' => self.check_two(
                '.',
                '=',
                TokenKind::DotDot,
                TokenKind::DotDotEq,
                TokenKind::Dot,
            ),
            '/' => self.check_compound('=', TokenKind::SlashEq, TokenKind::Slash),
            ':' => self.check_compound(':', TokenKind::ColonColon, TokenKind::Colon),

            // ── Unknown ──
            _ => {
                return Err(LexError {
                    ch: c,
                    span: (start..self.pos).into(),
                });
            }
        };

        Ok(self.make_token(kind, start))
    }

    // ── Helper: check for two‑character operators ──
    fn check_compound(
        &mut self,
        second: char,
        compound: TokenKind,
        single: TokenKind,
    ) -> TokenKind {
        if self.peek() == Some(&second) {
            self.advance();
            compound
        } else {
            single
        }
    }

    fn check_two(
        &mut self,
        first: char,
        second: char,
        both: TokenKind,
        first_only: TokenKind,
        neither: TokenKind,
    ) -> TokenKind {
        if self.peek() == Some(&first) {
            self.advance();
            if self.peek() == Some(&second) {
                self.advance();
                both
            } else {
                first_only
            }
        } else if self.peek() == Some(&second) {
            self.advance();
            first_only // only the second matched
        } else {
            neither
        }
    }
}

// ── Tests ──
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let tokens = tokenize("").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    #[test]
    fn test_keywords() {
        let tokens = tokenize("agent fn let return").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::KwAgent);
        assert_eq!(tokens[1].kind, TokenKind::KwFn);
        assert_eq!(tokens[2].kind, TokenKind::KwLet);
        assert_eq!(tokens[3].kind, TokenKind::KwReturn);
    }

    #[test]
    fn test_identifiers() {
        let tokens = tokenize("my_agent hello123 _start").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[2].kind, TokenKind::Ident);
    }

    #[test]
    fn test_numbers() {
        let tokens = tokenize("42 3.14 0xFF").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[2].kind, TokenKind::IntLiteral);
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("+ - * / == != <= >= && || |> |>>").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::EqEq);
        assert_eq!(tokens[5].kind, TokenKind::NotEq);
        assert_eq!(tokens[6].kind, TokenKind::LtEq);
        assert_eq!(tokens[7].kind, TokenKind::GtEq);
        assert_eq!(tokens[8].kind, TokenKind::AndAnd);
        assert_eq!(tokens[9].kind, TokenKind::OrOr);
        assert_eq!(tokens[10].kind, TokenKind::PipeGt);
        assert_eq!(tokens[11].kind, TokenKind::PipeGtGt);
    }

    #[test]
    fn test_comment_skip() {
        let tokens = tokenize("// comment\nlet x = 42").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::KwLet);
        assert_eq!(tokens[3].kind, TokenKind::IntLiteral);
    }

    #[test]
    fn test_block_comment() {
        let tokens = tokenize("let /* comment */ x").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::KwLet);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
    }

    #[test]
    fn test_brckt() {
        let tokens = tokenize("(){}[]").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::LParen);
        assert_eq!(tokens[1].kind, TokenKind::RParen);
        assert_eq!(tokens[2].kind, TokenKind::LBrace);
        assert_eq!(tokens[3].kind, TokenKind::RBrace);
        assert_eq!(tokens[4].kind, TokenKind::LBracket);
        assert_eq!(tokens[5].kind, TokenKind::RBracket);
    }
}
