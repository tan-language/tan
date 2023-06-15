pub mod token;

use std::str::Chars;

use crate::{
    error::{Error, ErrorKind},
    range::Range,
};

use self::token::{Token, TokenKind};

// https://en.wikipedia.org/wiki/Lexical_analysis

// #TODO lex_all, lex_single
// #TODO introduce SemanticToken, with extra semantic information, _after_ parsing.
// #TODO use annotations before number literals to set the type?
// #TODO use (doc_comment ...) for doc-comments.
// #TODO support `\ ` for escaped space in symbols.
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

// #Insight
// Rust's `Peekable` iterator is not used, as multiple-lookahead is
// required to scan e.g. signed-numbers. Additionally, the 'put_back' interface
// seems more intuitive and ergonomic.

// #Insight
// The lexer does not need synchronization to recover from errors.

// #Insight
// Don't try to make the lexer just a function.

// #Insight
// Numeric tokens parsing is postponed to a later stage (parse):
//   - there is more semantic information (e.g. annotations)
//   - joint synchronization in parsing phase allows to find more errors
//   - more scalebale for number formats, allows for less lexical tokens

/// The Lexer performs the lexical analysis stage of the compilation pipeline.
/// The input text is scanned into lexemes and then evaluated into lexical tokens.
/// The tokens are associated with ranges (ranges within the input text).
pub struct Lexer<'a> {
    chars: Chars<'a>,
    start: usize,
    index: usize,
    lookahead: Vec<char>,
    errors: Vec<Error>,
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

    // #TODO try to remove this!
    fn range(&self) -> Range {
        self.start..self.index
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

    // Scans withespace, returns the number of lines scanned.
    fn scan_whitespace(&mut self) -> i64 {
        let mut lines_count = 0;

        loop {
            let Some(ch) = self.next_char() else {
                break;
            };

            if !(is_whitespace(ch) || is_eol(ch)) {
                self.put_back_char(ch);
                break;
            }

            if is_eol(ch) {
                lines_count += 1;
            }
        }

        lines_count
    }

    // Scans a line, used for line-comments.
    fn scan_line(&mut self) -> String {
        let mut line = String::from("");

        loop {
            let Some(ch) = self.next_char() else {
                break;
            };

            if is_eol(ch) {
                // #Insight we put back the trailing EOL to help with range
                // and MultiLineWhitespace computation.
                self.put_back_char(ch);
                break;
            }

            line.push(ch);
        }

        line
    }

    // #TODO support 'raw' strings, e.g. (write #raw "this is \ cool")
    /// Scans a string lexeme.
    fn scan_string(&mut self) -> Option<String> {
        let mut string = String::new();

        loop {
            let Some(ch) = self.next_char() else {
                let mut error = Error::new(ErrorKind::UnterminatedString);
                error.push_note("String is missing the closing `\"` character", Some(self.range())); // #TODO refine the text.
                self.errors.push(error);
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
                let mut error = Error::new(ErrorKind::UnterminatedString);
                error.push_note("Text string is not closed", Some(self.range())); // #TODO refine the text.
                self.errors.push(error);
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
                        let mut error = Error::new(ErrorKind::UnterminatedString);
                        error.push_note("Text string is not closed", Some(self.range())); // #TODO refine the text.
                        self.errors.push(error);
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
            let mut error = Error::new(ErrorKind::UnterminatedAnnotation);
            error.push_note("Annotation is not closed", Some(self.range())); // #TODO refine the text.
            self.errors.push(error);

            None
        }
    }

    fn scan_number(&mut self) -> String {
        let lexeme = self.scan_lexeme();

        // Ignore `_`, it is considered a number separator.
        // #Insight do _not_ consider `,` as number separator, bad idea!
        lexeme.replace('_', "")
    }

    // #TODO consider passing into array of chars or something more general.
    pub fn lex(&mut self) -> Result<Vec<Token>, Vec<Error>> {
        let mut tokens: Vec<Token> = Vec::new();

        'outer: loop {
            self.start = self.index;

            let Some(ch) = self.next_char() else {
                break 'outer;
            };

            match ch {
                '(' => {
                    tokens.push(Token::new(TokenKind::LeftParen, None, self.range()));
                }
                ')' => {
                    tokens.push(Token::new(TokenKind::RightParen, None, self.range()));
                }
                // #TODO maybe should just rewrite [..] -> (Array ..)
                '[' => {
                    tokens.push(Token::new(TokenKind::LeftBracket, None, self.range()));
                }
                ']' => {
                    tokens.push(Token::new(TokenKind::RightBracket, None, self.range()));
                }
                // #TODO maybe should just rewrite {..} -> (Dict ..)
                '{' => {
                    tokens.push(Token::new(TokenKind::LeftBrace, None, self.range()));
                }
                '}' => {
                    tokens.push(Token::new(TokenKind::RightBrace, None, self.range()));
                }
                ';' => {
                    // #Insight
                    // We only support `;` line comments and not `--` line comments
                    // The `--` line comments don't play well with the use of `-`
                    // as word separator in names.
                    self.put_back_char(ch);
                    let line = self.scan_line();
                    tokens.push(Token::new(TokenKind::Comment, Some(line), self.range()));
                }
                '\'' => {
                    tokens.push(Token::new(TokenKind::Quote, None, self.range()));
                }
                '"' => {
                    let Some(ch1) = self.next_char() else {
                        let mut error = Error::new(ErrorKind::UnterminatedString);
                        error.push_note("String is not closed", Some(self.range())); // #TODO refine the text.
                        self.errors.push(error);
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
                                        let mut error = Error::new(ErrorKind::UnterminatedString);
                                        error.push_note("Text string is not closed", Some(self.range())); // #TODO refine the text.
                                        self.errors.push(error);
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
                                tokens.push(Token::new(
                                    TokenKind::String,
                                    Some(string),
                                    self.range(),
                                ));

                                continue;
                            }
                            self.put_back_char(ch2);
                        };
                    }

                    self.put_back_char(ch1);

                    let Some(string) = self.scan_string() else {
                        break;
                    };
                    tokens.push(Token::new(TokenKind::String, Some(string), self.range()));
                }
                '-' => {
                    let Some(ch1) = self.next_char() else {
                        let mut error = Error::new(ErrorKind::UnexpectedEnd);
                        error.push_note("Text string is not closed", Some(self.range())); // #TODO refine the text.
                        self.errors.push(error);
                        break 'outer;
                    };

                    self.put_back_char(ch1);
                    self.put_back_char(ch);

                    // #insight `--` line comments no longer supported.
                    // if ch1 == '-' {
                    //     // #TODO if we switch to kebab-case, `--` comments may cause issues.
                    //     // `--` line comment
                    //     let line = self.scan_line();
                    //     tokens.push(Ranged(Token::Comment(line), self.range()));
                    // } else
                    if ch1.is_numeric() {
                        // Negative number
                        tokens.push(Token::new(
                            TokenKind::Number,
                            Some(self.scan_number()),
                            self.range(),
                        ));
                    } else {
                        // #TODO lint warning for this!
                        // Symbol starting with `-`.
                        tokens.push(Token::new(
                            TokenKind::Symbol,
                            Some(self.scan_lexeme()),
                            self.range(),
                        ));
                    }
                }
                '#' => {
                    if self.index == 1 {
                        if let Some(ch1) = self.next_char() {
                            if ch1 == '!' {
                                // Shebang line detected, skip.
                                let _ = self.scan_line();
                                // #TODO should we keep the shebang as a module annotation?
                                continue;
                            } else {
                                self.put_back_char(ch1);
                            }
                        }
                    }
                    let Some(ann) = self.scan_annotation() else {
                        break 'outer;
                    };
                    tokens.push(Token::new(TokenKind::Annotation, Some(ann), self.range()));
                }
                _ if is_whitespace(ch) => {
                    // Consume whitespace
                    if is_eol(ch) {
                        self.put_back_char(ch);
                    }

                    let lines_count = self.scan_whitespace();

                    if lines_count > 1 {
                        tokens.push(Token::new(
                            TokenKind::MultiLineWhitespace,
                            None,
                            self.range(),
                        ));
                    }
                }
                _ if ch.is_numeric() => {
                    self.put_back_char(ch);

                    tokens.push(Token::new(
                        TokenKind::Number,
                        Some(self.scan_number()),
                        self.range(),
                    ));
                }
                _ => {
                    self.put_back_char(ch);

                    tokens.push(Token::new(
                        TokenKind::Symbol,
                        Some(self.scan_lexeme()),
                        self.range(),
                    ));
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
