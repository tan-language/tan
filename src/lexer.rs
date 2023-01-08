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
    ch == '(' || ch == ')' || ch == '[' || ch == ']' || ch == '{' || ch == '}'
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
        if let Some(ch) = self.lookahead.pop() {
            self.index += 1;
            return Some(ch);
        }

        if let Some(ch) = self.chars.next() {
            self.index += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn put_back_char(&mut self, ch: char) {
        self.lookahead.push(ch);
        self.index -= 1;
    }

    // #TODO implement scanners with macro or a common function.
    // #TODO two functions scan_lexeme, scan_delimited.

    // #TODO add unit tests
    // #TODO try to reuse more!
    fn scan_lexeme(&mut self) -> String {
        let mut text = String::new();

        loop {
            let Some(ch) = self.next_char() else {
                break;
            };

            // #TODO maybe whitespace does not need put_back, but need to adjust range.
            if is_whitespace(ch) || is_delimiter(ch) || is_eol(ch) {
                self.put_back_char(ch);
                break;
            }

            text.push(ch);
        }

        text
    }

    // Scans a line.
    fn scan_line(&mut self) -> String {
        let mut comment = String::from("");

        loop {
            let Some(ch) = self.next_char() else {
                break;
            };

            if is_eol(ch) {
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
            let Some(ch) = self.next_char() else {
                return Err(LexicalError::UnterminatedString);
            };

            if ch == '"' {
                break;
            }

            string.push(ch);
        }

        Ok(string)
    }

    fn scan_annotation(&mut self) -> Result<String, LexicalError> {
        let mut ann = String::new();

        let mut nesting = 0;

        // #TODO only allow one level of nesting?
        // #TODO should probably skip the annotation 'parsing'.

        loop {
            let Some(ch) = self.next_char() else {
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

            ann.push(ch);
        }

        if nesting != 0 {
            return Err(LexicalError::UnterminatedAnnotation);
        }

        Ok(ann)
    }

    fn lex_number(&mut self) -> Result<Token, LexicalError> {
        let lexeme = self.scan_lexeme();

        // Ignore `_`, it is considered a number separator.
        // #Insight do _not_ consider `,` as number separator, bad idea!
        let mut lexeme = lexeme.replace('_', "");

        // #TODO more detailed Number error!
        // #TODO error handling not enough, we need to add context, check error_stack

        let token = if lexeme.contains('.') {
            // #TODO support radix for non-integers?
            // #TODO find a better name for 'non-integer'.
            let n = lexeme.parse().map_err(LexicalError::MalformedFloat)?;
            Token::Float(n)
        } else {
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

            let n = i64::from_str_radix(&lexeme, radix).map_err(LexicalError::MalformedInt)?;
            Token::Int(n)
        };

        Ok(token)
    }

    // #TODO consider passing into array of chars or something more general.
    pub fn lex(&mut self) -> Result<Vec<Ranged<Token>>, Ranged<LexicalError>> {
        let mut tokens: Vec<Ranged<Token>> = Vec::new();

        loop {
            let start = self.index;

            let Some(ch) = self.next_char() else {
                break;
            };

            match ch {
                '(' => {
                    let range = start..self.index;
                    tokens.push(Ranged(Token::LeftParen, range));
                }
                ')' => {
                    let range = start..self.index;
                    tokens.push(Ranged(Token::RightParen, range));
                }
                // #TODO maybe should just rewrite [..] -> (Array ..)
                '[' => {
                    let range = start..self.index;
                    tokens.push(Ranged(Token::LeftBracket, range));
                }
                ']' => {
                    let range = start..self.index;
                    tokens.push(Ranged(Token::RightBracket, range));
                }
                // #TODO maybe should just rewrite {..} -> (Dict ..)
                '{' => {
                    let range = start..self.index;
                    tokens.push(Ranged(Token::LeftBrace, range));
                }
                '}' => {
                    let range = start..self.index;
                    tokens.push(Ranged(Token::RightBrace, range));
                }
                // #TODO consider removing `;` comments.
                ';' => {
                    self.put_back_char(ch);
                    let line = self.scan_line();
                    let range = start..self.index;
                    tokens.push(Ranged(Token::Comment(line), range));
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
                    let Some(ch1) = self.next_char() else {
                        let range = start..(self.index-1);
                        return Err(Ranged(LexicalError::UnexpectedEol, range));
                    };

                    self.put_back_char(ch1);
                    self.put_back_char(ch);

                    if ch1 == '-' {
                        // #Warning if we switch to kebab-case, `--` comments may cause issues.
                        // `--` line comment
                        let line = self.scan_line();
                        let range = start..self.index;
                        tokens.push(Ranged(Token::Comment(line), range));
                    } else if ch1.is_numeric() {
                        // Negative number

                        let token = self.lex_number();
                        let range = start..self.index;
                        let Ok(token) = token else {
                            return Err(Ranged(token.unwrap_err(), range));
                        };
                        tokens.push(Ranged(token, range));
                    } else {
                        // #TODO lint warning for this!
                        // Symbol starting with `-`.

                        let sym = self.scan_lexeme();
                        let range = start..self.index;
                        tokens.push(Ranged(Token::Symbol(sym), range));
                    }
                }
                '#' => {
                    let ann = self.scan_annotation();
                    let range = start..self.index;
                    let Ok(ann) = ann else {
                        return Err(Ranged(ann.unwrap_err(), range));
                    };
                    tokens.push(Ranged(Token::Annotation(ann), range));
                }
                _ if is_whitespace(ch) => {
                    // Consume whitespace
                }
                _ if ch.is_numeric() => {
                    self.put_back_char(ch);

                    let token = self.lex_number();
                    let range = start..self.index;
                    let Ok(token) = token else {
                        return Err(Ranged(token.unwrap_err(), range));
                    };
                    tokens.push(Ranged(token, range));
                }
                _ => {
                    self.put_back_char(ch);

                    let sym = self.scan_lexeme();
                    let range = start..self.index;
                    tokens.push(Ranged(Token::Symbol(sym), range));
                }
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    // #TODO!
}
