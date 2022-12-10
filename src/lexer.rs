use crate::types::{Span, Spanned};
use std::error::Error;
use std::fmt;
use std::{num::ParseIntError, str::Chars};

// https://en.wikipedia.org/wiki/Lexical_analysis

// #TODO lex_all, lex_single
// #TODO use peekable iterator instead of put_back/lookahead.

// There is no need for an EOF Token. The end of the Token list marks the end
// of the input.

/// A lexical Token gives semantic meaning to a Lexeme.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    Number(i64),
    String(String),
    Symbol(String),
    If,
    Using,
    // True,
    // False,
    Comment(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Token::*;
        f.write_str(
            (match self {
                LParen => "(".to_string(),
                RParen => ")".to_string(),
                Number(n) => format!("{}", n),
                String(s) => s.to_string(),
                Symbol(s) => s.to_string(),
                If => "if".to_string(),
                Using => "using".to_string(),
                Comment(s) => s.to_string(),
            })
            .as_str(),
        )
    }
}

// #TODO better name or extract.
#[derive(Debug)]
pub enum LexicalError {
    NumberError(ParseIntError, Span),
    UnterminatedStringError(Span),
}

impl Error for LexicalError {}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Returns true if ch is considered whitespace.
/// The `,` character is considered whitespace, in the Lisp tradition.
fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace() || ch == ','
}

fn is_delimiter(ch: char) -> bool {
    ch == '(' || ch == ')'
}

// #TODO stateful lexer vs buffer

pub struct Lexer<'a> {
    chars: Chars<'a>,
    index: usize,
    lookahead: Option<char>, // #TODO find better name!
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            index: 0,
            lookahead: None,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.index += 1;

        if let Some(ch) = self.lookahead {
            self.lookahead = None;
            return Some(ch);
        }

        self.chars.next()
    }

    fn put_back_char(&mut self, ch: char) {
        self.lookahead = Some(ch);
        self.index -= 1;
    }

    // Span is a right-open range.
    fn span(&self, start: usize) -> Span {
        Span {
            start: start - 1,
            end: self.index,
        }
    }

    pub fn lex_string(&mut self) -> Result<Spanned<Token>, LexicalError> {
        let start = self.index;
        let mut text = String::new();

        let mut char = self.next_char();

        while let Some(ch) = char {
            if ch == '"' {
                break;
            }

            text.push(ch);

            char = self.next_char();
        }

        if char != Some('"') {
            return Err(LexicalError::UnterminatedStringError(self.span(start)));
        }

        Ok((Token::String(text), self.span(start)))
    }

    // #TODO consider passing into array of chars or something more general.
    pub fn lex(&mut self) -> Result<Vec<Spanned<Token>>, LexicalError> {
        let mut tokens: Vec<Spanned<Token>> = Vec::new();

        let mut char = self.next_char();

        while let Some(ch) = char {
            match ch {
                '(' => tokens.push((Token::LParen, self.span(self.index))),
                ')' => tokens.push((Token::RParen, self.span(self.index))),
                ';' => {
                    let start = self.index;
                    let mut text = String::new();

                    while let Some(ch) = char {
                        if ch == '\n' {
                            break;
                        }

                        text.push(ch);

                        char = self.next_char();
                    }

                    let mut span = self.span(start);
                    // Adjust for the trailing '\n'.
                    span.end -= 1;
                    tokens.push((Token::Comment(text), span));
                }
                '"' => {
                    tokens.push(self.lex_string()?);
                }
                _ if is_whitespace(ch) => {
                    // Consume whitespace
                }
                _ => {
                    // #TODO check for reserved names: if, for, using, pragma

                    let start = self.index;
                    let mut text = String::new();

                    let is_number = ch.is_numeric();

                    while let Some(ch) = char {
                        if is_whitespace(ch) || is_delimiter(ch) {
                            self.put_back_char(ch);
                            break;
                        }

                        text.push(ch);

                        char = self.next_char();
                    }

                    let token = if is_number {
                        // #TODO error handling not enough, we need to add context, check error_stack
                        let n: i64 = text
                            .parse()
                            .map_err(|err| LexicalError::NumberError(err, self.span(start)))?;

                        // #TODO support 0b01111 binary numbers
                        // #TODO support 0xaf001 hex numbers

                        Token::Number(n)
                    } else {
                        match text.as_str() {
                            "if" => Token::If,
                            "using" => Token::Using,
                            _ => Token::Symbol(text),
                        }
                    };

                    tokens.push((token, self.span(start)));
                }
            }

            char = self.next_char();
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_should_handle_an_empty_string() {
        let source = "";
        let tokens = Lexer::new(source).lex();

        let tokens = tokens.unwrap();

        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn lex_should_tokenize() {
        let source = "((+ 1   25 399)  )";
        let tokens = Lexer::new(source).lex();

        let tokens = tokens.unwrap();

        dbg!(&tokens);

        assert_eq!(tokens.len(), 8);

        // #TODO add specific assertions.
    }
}
