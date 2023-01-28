pub mod token;

use std::{ops::Range, str::Chars};

use crate::{error::Error, range::Ranged};

use self::token::Token;

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
// #TODO postpone parsing of numerics to a later stage where there is more semantic information (e.g. annotations)

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

// #Insight
// Rust's `Peekable` iterator is not used, as multiple-lookahead is
// required to scan e.g. signed-numbers. Additionally, the 'put_back' interface
// seems more intuitive and ergonomic.

// #Insight
// The lexer does not need synchronization to recover from errors.

/// The Lexer performs the lexical analysis stage of the compilation pipeline.
/// The input text is scanned into lexemes and then evaluated into lexical tokens.
/// The tokens are associated with ranges (ranges within the input text).
pub struct Lexer<'a> {
    chars: Chars<'a>,
    start: usize,
    index: usize,
    lookahead: Vec<char>,
    errors: Vec<Ranged<Error>>,
}

impl<'a> Lexer<'a> {
    /// Makes a new Lexer with the given input text.
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            start: 0,
            index: 0,
            lookahead: Vec::new(),
            errors: Vec::new(),
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

    fn range(&self) -> Range<usize> {
        self.start..self.index
    }

    fn push_error(&mut self, error: Error) {
        self.errors.push(Ranged(error, self.range()));
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

    // #TODO support 'raw' strings, e.g. (write #raw "this is \ cool")
    /// Scans a string lexeme.
    fn scan_string(&mut self) -> Option<String> {
        let mut string = String::new();

        loop {
            let Some(ch) = self.next_char() else {
                self.push_error(Error::UnterminatedString);
                return None;
            };

            if ch == '"' {
                break;
            }

            string.push(ch);
        }

        Some(string)
    }

    // #TODO needs cleanup.
    // #TODO does not support leading tabs.
    // #TODO find better name, `scan_indented_string`.
    // #TODO support 'raw' strings, e.g. (write #raw "this is \ cool")
    /// Scans a multi-string 'layout'.
    fn scan_text(&mut self, indent: u64) -> Option<String> {
        let mut string = String::new();

        let mut quote_count = 0;

        loop {
            let Some(ch) = self.next_char() else {
                self.push_error(Error::UnterminatedString);
                return None;
            };

            if ch == '"' {
                quote_count += 1;

                if quote_count < 3 {
                    continue;
                } else {
                    break;
                }
            } else if is_eol(ch) {
                for _ in 0..indent {
                    let Some(ch1) = self.next_char() else {
                        self.push_error(Error::UnterminatedString);
                        return None;
                    };

                    if is_eol(ch1) {
                        self.put_back_char(ch1);
                        break;
                    }

                    if !ch1.is_whitespace() {
                        self.put_back_char(ch1);
                        break;
                    }
                }
            } else {
                // #TODO support single or double `"`.
            }

            string.push(ch);
        }

        Some(string)
    }

    fn scan_annotation(&mut self) -> Option<String> {
        let mut ann = String::new();

        let mut nesting = 0;

        // #TODO only allow one level of nesting?

        loop {
            let Some(ch) = self.next_char() else {
                break;
            };

            if ch == '(' {
                nesting += 1;
            } else if ch == ')' {
                nesting -= 1;
            } else if nesting == 0 && (is_whitespace(ch) || is_eol(ch)) {
                self.put_back_char(ch);
                break;
            }

            ann.push(ch);
        }

        if nesting == 0 {
            Some(ann)
        } else {
            self.push_error(Error::UnterminatedAnnotation);
            None
        }
    }

    // #Insight uses the prefix lex because it returns a Token.
    fn lex_number(&mut self) -> Option<Token> {
        let lexeme = self.scan_lexeme();

        // Ignore `_`, it is considered a number separator.
        // #Insight do _not_ consider `,` as number separator, bad idea!
        let mut lexeme = lexeme.replace('_', "");

        // #TODO more detailed Number error!
        // #TODO error handling not enough, we need to add context, check error_stack

        if lexeme.contains('.') {
            // #TODO support radix for non-integers?
            // #TODO find a better name for 'non-integer'.
            match lexeme.parse::<f64>().map_err(Error::MalformedFloat) {
                Ok(n) => Some(Token::Float(n)),
                Err(error) => {
                    self.push_error(error);
                    None
                }
            }
        } else {
            // #TODO support arbitrary radix https://github.com/golang/go/issues/28256
            let mut radix = 10;

            if lexeme.starts_with("0x") {
                lexeme = lexeme.replace("0x", "");
                radix = 16
            } else if lexeme.starts_with("0b") {
                lexeme = lexeme.replace("0b", "");
                radix = 2
            } else if lexeme.starts_with("0o") {
                // #Insight Octal literals are supported for historical reasons.
                lexeme = lexeme.replace("0o", "");
                radix = 8
            }

            match i64::from_str_radix(&lexeme, radix).map_err(Error::MalformedInt) {
                Ok(n) => Some(Token::Int(n)),
                Err(error) => {
                    self.push_error(error);
                    None
                }
            }
        }
    }

    // #TODO consider passing into array of chars or something more general.
    pub fn lex(&mut self) -> Result<Vec<Ranged<Token>>, Vec<Ranged<Error>>> {
        let mut tokens: Vec<Ranged<Token>> = Vec::new();

        'outer: loop {
            self.start = self.index;

            let Some(ch) = self.next_char() else {
                break 'outer;
            };

            match ch {
                '(' => {
                    tokens.push(Ranged(Token::LeftParen, self.range()));
                }
                ')' => {
                    tokens.push(Ranged(Token::RightParen, self.range()));
                }
                // #TODO maybe should just rewrite [..] -> (Array ..)
                '[' => {
                    tokens.push(Ranged(Token::LeftBracket, self.range()));
                }
                ']' => {
                    tokens.push(Ranged(Token::RightBracket, self.range()));
                }
                // #TODO maybe should just rewrite {..} -> (Dict ..)
                '{' => {
                    tokens.push(Ranged(Token::LeftBrace, self.range()));
                }
                '}' => {
                    tokens.push(Ranged(Token::RightBrace, self.range()));
                }
                // #TODO consider removing `;` comments.
                ';' => {
                    self.put_back_char(ch);
                    let line = self.scan_line();
                    tokens.push(Ranged(Token::Comment(line), self.range()));
                }
                '\'' => {
                    tokens.push(Ranged(Token::Quote, self.range()));
                }
                '"' => {
                    let Some(ch1) = self.next_char() else {
                        self.push_error(Error::UnterminatedString);
                        break 'outer;
                    };

                    // Check for `"""` triple-quote multi-line string delimiter.
                    if ch1 == '"' {
                        if let Some(ch2) = self.next_char() {
                            if ch2 == '"' {
                                // Compute the indentation.
                                let mut indent = 0;

                                loop {
                                    let Some(ch3) = self.next_char() else {
                                        self.push_error(Error::UnterminatedString);
                                        break 'outer;
                                    };

                                    if is_eol(ch3) {
                                        indent = 0;
                                        continue;
                                    } else if ch3.is_whitespace() {
                                        indent += 1;
                                    } else {
                                        self.put_back_char(ch3);
                                        break;
                                    }
                                }

                                let Some(string) = self.scan_text(indent) else {
                                    break;
                                };
                                tokens.push(Ranged(Token::String(string), self.range()));

                                continue;
                            }
                            self.put_back_char(ch2);
                        };
                    }

                    self.put_back_char(ch1);

                    let Some(string) = self.scan_string() else {
                        break;
                    };
                    tokens.push(Ranged(Token::String(string), self.range()));
                }
                '-' => {
                    let Some(ch1) = self.next_char() else {
                        self.push_error(Error::UnexpectedEnd);
                        break 'outer;
                    };

                    self.put_back_char(ch1);
                    self.put_back_char(ch);

                    if ch1 == '-' {
                        // #Warning if we switch to kebab-case, `--` comments may cause issues.
                        // `--` line comment
                        let line = self.scan_line();
                        tokens.push(Ranged(Token::Comment(line), self.range()));
                    } else if ch1.is_numeric() {
                        // Negative number

                        if let Some(token) = self.lex_number() {
                            tokens.push(Ranged(token, self.range()));
                            // Don't break, continue scanning to find more errors.
                        }
                    } else {
                        // #TODO lint warning for this!
                        // Symbol starting with `-`.

                        let sym = self.scan_lexeme();
                        tokens.push(Ranged(Token::Symbol(sym), self.range()));
                    }
                }
                '#' => {
                    let Some(ann) = self.scan_annotation() else {
                        break 'outer;
                    };
                    tokens.push(Ranged(Token::Annotation(ann), self.range()));
                }
                _ if is_whitespace(ch) => {
                    // Consume whitespace
                    continue;
                }
                _ if ch.is_numeric() => {
                    self.put_back_char(ch);

                    if let Some(token) = self.lex_number() {
                        tokens.push(Ranged(token, self.range()));
                        // Don't break, continue scanning to find more errors.
                    }
                }
                _ => {
                    self.put_back_char(ch);

                    let sym = self.scan_lexeme();
                    tokens.push(Ranged(Token::Symbol(sym), self.range()));
                }
            }
        }

        if self.errors.is_empty() {
            // #TODO
            Ok(tokens)
        } else {
            let errors = std::mem::take(&mut self.errors);
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    // #TODO!
}
