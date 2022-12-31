pub mod error;
pub mod token;

use std::str::Chars;

use crate::range::Ranged;

use self::{error::LexicalError, token::Token};

// https://en.wikipedia.org/wiki/Lexical_analysis

// #TODO lex_all, lex_single
// #TODO introduce SemanticToken, with extra semantic information, _after_ parsing.
// #TODO use annotations before number literals to set the type?
// #TODO use (doc_comment ...) for doc-comments.
// #TODO support `\ ` for escaped space in symbols.
// #TODO can the lexer be just a function?
// #TODO implement PutBackIterator
// #TODO no need to keep iterator as state in Lexer!
// #TODO accept IntoIterator
// #TODO try to use `let mut reader = BufReader::new(source.as_bytes());` like an older version

/// Returns true if ch is considered whitespace.
/// The `,` character is considered whitespace, in the Lisp tradition.
fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace() || ch == ','
}

fn is_delimiter(ch: char) -> bool {
    ch == '(' || ch == ')'
}

fn is_eol(ch: char) -> bool {
    ch == '\n'
}

// #TODO stateful lexer vs buffer

// #Insight Rust's `Peekable` iterator is not used, as multiple-lookahead is
// required to scan e.g. signed-numbers. Additionally, the 'put_back' interface
// seems more intuitive and ergonomic.

/// The Lexer performs the lexical analysis stage of the compilation pipeline.
/// The input text is scanned into lexemes and then evaluated into lexical tokens.
/// The tokens are associated with ranges (ranges within the input text).
pub struct Lexer<'a> {
    chars: Chars<'a>,
    index: usize,
    lookahead: Vec<char>,
}

impl<'a> Lexer<'a> {
    /// Makes a new Lexer with the given input text.
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            index: 0,
            lookahead: Vec::new(),
        }
    }

    /// Returns the input text as a String.
    pub fn input(&self) -> String {
        self.chars.clone().collect()
    }

    // #TODO unit test
    // #TODO refactor
    fn next_char(&mut self) -> Option<char> {
        if let Some(char) = self.lookahead.pop() {
            self.index += 1;
            return Some(char);
        }

        if let Some(char) = self.chars.next() {
            self.index += 1;
            Some(char)
        } else {
            None
        }
    }

    fn put_back_char(&mut self, ch: char) {
        self.lookahead.push(ch);
        self.index -= 1;
    }

    // #TODO add unit tests
    // #TODO try to reuse more!
    fn scan_lexeme(&mut self) -> Ranged<String> {
        let mut text = String::new();

        let start = self.index;

        let mut char;

        loop {
            char = self.next_char();

            let Some(ch) = char  else {
                break;
            };

            // #TODO maybe whitespace does not need put_back, but need to adjust range.
            if is_whitespace(ch) || is_delimiter(ch) || is_eol(ch) {
                self.put_back_char(ch);
                break;
            }

            text.push(ch);
        }

        let range = start..self.index;

        Ranged(text, range)
    }

    // Scans a comment lexeme, since it ignores all input until EOL or EOF,
    // it is infallible.
    fn scan_comment(&mut self) -> String {
        let mut comment = String::from(";");

        loop {
            let char = self.next_char();

            let Some(ch) = char  else {
                break;
            };

            if ch == '\n' {
                break;
            }

            comment.push(ch);
        }

        comment
    }

    // #TODO support multi-line strings
    // #TODO support 'raw' strings, e.g. (write #raw "this is \ cool")
    /// Scans a string lexeme.
    fn scan_string(&mut self) -> Result<String, LexicalError> {
        let mut string = String::new();

        loop {
            let char = self.next_char();

            let Some(ch) = char  else {
                return Err(LexicalError::UnterminatedStringError);
            };

            if ch == '"' {
                break;
            }

            string.push(ch);
        }

        Ok(string)
    }

    // #TODO the lexer should keep the Number token as String.
    fn lex_number(&mut self) -> Result<Ranged<Token>, Ranged<LexicalError>> {
        let Ranged(lexeme, range) = self.scan_lexeme();

        // Ignore `_`, it is considered a number separator.
        // #Insight do _not_ consider `,` as number separator, bad idea!
        let mut lexeme = lexeme.replace('_', "");

        // #TODO support radix-8 -> no, leave for arbitrary radix.
        // #TODO support arbitrary radix https://github.com/golang/go/issues/28256
        let mut radix = 10;

        if lexeme.starts_with("0x") {
            lexeme = lexeme.replace("0x", "");
            radix = 16
        } else if lexeme.starts_with("0b") {
            lexeme = lexeme.replace("0b", "");
            radix = 2
        }

        // #TODO more detailed Number error!
        // #TODO error handling not enough, we need to add context, check error_stack

        let n = i64::from_str_radix(&lexeme, radix)
            .map_err(|err| Ranged(LexicalError::NumberError(err), range.clone()))?;

        Ok(Ranged(Token::Number(n), range))
    }

    fn lex_symbol(&mut self) -> Result<Ranged<Token>, Ranged<LexicalError>> {
        let Ranged(lexeme, range) = self.scan_lexeme();

        let token = Token::Symbol(lexeme);

        // #Insight
        // It's not worth it to have token variants for reserved words.
        // Expr variants for reserved words will be used.

        Ok(Ranged(token, range))
    }

    fn lex_annotation(&mut self) -> Result<Ranged<Token>, Ranged<LexicalError>> {
        let mut text = String::new();

        let start = self.index - 1; // adjust for leading '#'

        let mut nesting = 0;

        // #TODO only allow one level of nesting?

        let mut char;

        loop {
            char = self.next_char();

            let Some(ch) = char  else {
                break;
            };

            if ch == '(' {
                nesting += 1;
            } else if ch == ')' {
                nesting -= 1;
            } else if nesting == 0 && (is_whitespace(ch) || is_eol(ch)) {
                // #TODO maybe whitespace does not need put_back, but need to adjust range.
                self.put_back_char(ch);
                break;
            }

            text.push(ch);
        }

        let range = start..self.index;

        if nesting != 0 {
            return Err(Ranged(LexicalError::UnterminatedAnnotationError, range));
        }

        Ok(Ranged(Token::Annotation(text), range))
    }

    // #TODO consider passing into array of chars or something more general.
    pub fn lex(&mut self) -> Result<Vec<Ranged<Token>>, Ranged<LexicalError>> {
        let mut tokens: Vec<Ranged<Token>> = Vec::new();

        loop {
            let start = self.index;

            let Some(char) = self.next_char() else {
                break;
            };

            match char {
                '(' => {
                    let range = start..self.index;
                    tokens.push(Ranged(Token::LeftParen, range));
                }
                ')' => {
                    let range = start..self.index;
                    tokens.push(Ranged(Token::RightParen, range));
                }
                ';' => {
                    let comment = self.scan_comment();
                    let range = start..self.index;
                    tokens.push(Ranged(Token::Comment(comment), range));
                }
                '\'' => {
                    let range = start..self.index;
                    tokens.push(Ranged(Token::Quote, range));
                }
                '"' => {
                    let string = self.scan_string();
                    let range = start..self.index;
                    let Ok(string) = string else {
                        return Err(Ranged(string.unwrap_err(), range));
                    };
                    tokens.push(Ranged(Token::String(string), range));
                }
                '-' => {
                    // #TODO support for `--` line comments!

                    let char1 = self.next_char();

                    let Some(ch1) = char1 else {
                        let range = start..(self.index-1);
                        return Err(Ranged(LexicalError::UnexpectedEol, range));
                    };

                    if ch1.is_numeric() {
                        // Negative number
                        self.put_back_char(ch1);
                        self.put_back_char(char);
                        // #TODO handle range outside of lex_xxx
                        tokens.push(self.lex_number()?);
                    } else {
                        // #TODO lint warning for this!
                        // Symbol
                        self.put_back_char(ch1);
                        self.put_back_char(char);
                        // #TODO handle range outside of lex_xxx
                        tokens.push(self.lex_symbol()?);
                    }
                }
                '#' => {
                    // #TODO handle range outside of lex_xxx
                    tokens.push(self.lex_annotation()?);
                }
                _ if is_whitespace(char) => {
                    // Consume whitespace
                }
                _ if char.is_numeric() => {
                    self.put_back_char(char);
                    // #TODO handle range outside of lex_xxx
                    tokens.push(self.lex_number()?);
                }
                _ => {
                    self.put_back_char(char);
                    // #TODO handle range outside of lex_xxx
                    tokens.push(self.lex_symbol()?);
                }
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::Expr;

    #[test]
    fn expr_string_display() {
        let expr = Expr::String("hello".to_owned());
        assert_eq!("\"hello\"", format!("{expr}"));
    }
}
