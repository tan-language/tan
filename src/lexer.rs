pub mod comment;
pub mod token;

use std::str::Chars;

use crate::{
    error::{Error, ErrorVariant},
    range::{Position, Range},
    util::is_range_literal,
};

use self::{
    comment::CommentKind,
    token::{Token, TokenKind},
};

// https://en.wikipedia.org/wiki/Lexical_analysis

// #todo lex_all, lex_single
// #todo introduce SemanticToken, with extra semantic information, _after_ parsing.
// #todo use annotations before number literals to set the type?
// #todo use (doc-comment ...) for doc-comments.
// #todo no need to keep iterator as state in Lexer!
// #todo accept IntoIterator
// #todo try to use `let mut reader = BufReader::new(source.as_bytes());` like an older version

/// Returns true if ch is considered whitespace.
fn is_whitespace(ch: char) -> bool {
    // #insight
    // do _not_ consider `,` as a separator, it can be useful in names,
    // for example in *.css.tan files.
    ch.is_whitespace() // || ch == ','
}

fn is_delimiter(ch: char) -> bool {
    ch == '(' || ch == ')' || ch == '[' || ch == ']' || ch == '{' || ch == '}'
}

fn is_eol(ch: char) -> bool {
    ch == '\n'
}

// #todo stateful lexer vs buffer

// #insight
// Rust's `Peekable` iterator is not used, as multiple-lookahead is
// required to scan e.g. signed-numbers. Additionally, the 'put_back' interface
// seems more intuitive and ergonomic.

// #insight
// The lexer does not need synchronization to recover from errors.

// #insight
// Don't try to make the lexer just a function.

// #insight
// Numeric tokens parsing is postponed to a later stage (parse):
//   - there is more semantic information (e.g. annotations)
//   - joint synchronization in parsing phase allows to find more errors
//   - more scalebale for number formats, allows for less lexical tokens

/// The Lexer performs the lexical analysis stage of the compilation pipeline.
/// The input text is scanned into lexemes and then evaluated into lexical tokens.
/// The tokens are associated with ranges (ranges within the input text).
pub struct Lexer<'a> {
    chars: Chars<'a>,
    current_position: Position,
    start_position: Position,
    lookahead: Vec<char>,
    errors: Vec<Error>,
}

impl<'a> Lexer<'a> {
    /// Makes a new Lexer with the given input text.
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            current_position: Position::default(),
            start_position: Position::default(),
            lookahead: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn with_position(self, position: Position) -> Self {
        Self {
            current_position: position,
            start_position: position,
            ..self
        }
    }

    /// Returns the input text as a String.
    pub fn input(&self) -> String {
        self.chars.clone().collect()
    }

    // #todo unit test
    // #todo refactor
    fn next_char(&mut self) -> Option<char> {
        if let Some(ch) = self.lookahead.pop() {
            self.current_position.index += 1;
            self.current_position.col += 1;
            return Some(ch);
        }

        if let Some(ch) = self.chars.next() {
            self.current_position.index += 1;
            self.current_position.col += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn put_back_char(&mut self, ch: char) {
        self.lookahead.push(ch);
        self.current_position.index -= 1;
        self.current_position.col -= 1;
    }

    // #todo try to remove this!
    fn current_range(&self) -> Range {
        self.start_position..self.current_position
    }

    // #todo implement scanners with macro or a common function.
    // #todo two functions scan_lexeme, scan_delimited.

    fn scan_chars(&mut self, count: usize) -> Option<String> {
        let mut chars = String::new();
        for _ in 0..count {
            let ch = self.next_char()?;
            chars.push(ch);
        }
        Some(chars)
    }

    // #todo add unit tests
    // #todo try to reuse more!
    fn scan_lexeme(&mut self) -> String {
        let mut text = String::new();

        loop {
            let Some(ch) = self.next_char() else {
                break;
            };

            // #todo maybe whitespace does not need put_back, but need to adjust range.
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

            // #insight
            // is_whitespace returns true for EOL.

            if !is_whitespace(ch) {
                self.put_back_char(ch);
                break;
            }

            if is_eol(ch) {
                // #todo extract position method!
                self.current_position.line += 1;
                self.current_position.col = 0;
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
                // #insight we put back the trailing EOL to help with range
                // and MultiLineWhitespace computation.
                self.put_back_char(ch);
                break;
            }

            line.push(ch);
        }

        line
    }

    // #todo support 'raw' strings, e.g. (write #raw "this is \ cool")
    /// Scans a string lexeme.
    fn scan_string(&mut self) -> Option<String> {
        let mut string = String::new();

        let mut is_escaping = false;

        loop {
            let Some(ch) = self.next_char() else {
                let mut error = Error::new(ErrorVariant::UnterminatedString);
                error.push_note(
                    "string is missing the closing `\"` character",
                    Some(self.current_range()),
                ); // #todo refine the text.
                self.errors.push(error);
                return None;
            };

            if is_escaping {
                // #todo support escaping more than one char
                // #todo add additional escape sequences.
                // #todo support \xHH
                // #todo support \u{HHHH}: Represents a 16-bit Unicode code point (where HHHH is up to four hex digits)
                // #todo support \U{HHHHHHHH}: Represents a 32-bit Unicode code point (where HHHHHHHH is up to eight hex digits)
                match ch {
                    '\\' | '"' => string.push(ch),
                    'n' => string.push('\n'),
                    't' => string.push('\t'),
                    'x' => {
                        // #todo streamline this or extract
                        // an escaped byte, in hexadecimal notation, read two
                        // characters.
                        let Some(chars) = self.scan_chars(2) else {
                            let mut error = Error::new(ErrorVariant::MalformedEscapeCode);
                            error.push_note(
                                "the \\x escape code requires two characters",
                                Some(self.current_range()),
                            ); // #todo refine the text.
                            self.errors.push(error);
                            return None;
                        };
                        let code_point = u32::from_str_radix(&chars, 16);
                        let Ok(code_point) = u32::from_str_radix(&chars, 16) else {
                            let mut error = Error::new(ErrorVariant::MalformedEscapeCode);
                            error.push_note(
                                &format!(
                                    "invalid \\x escape code point: {}",
                                    code_point.unwrap_err()
                                ),
                                Some(self.current_range()),
                            ); // #todo refine the text.
                            self.errors.push(error);
                            return None;
                        };
                        let Some(ch) = char::from_u32(code_point) else {
                            let mut error = Error::new(ErrorVariant::MalformedEscapeCode);
                            error.push_note(
                                "invalid \\x escape code point",
                                Some(self.current_range()),
                            ); // #todo refine the text.
                            self.errors.push(error);
                            return None;
                        };
                        string.push(ch);
                    }
                    _ => string.push_str(&format!("\\{ch}")), //#todo what to do here?
                }

                is_escaping = false;
                continue;
            }

            if ch == '\\' {
                is_escaping = true;
                continue;
            }

            if ch == '"' {
                break;
            }

            string.push(ch);
        }

        Some(string)
    }

    // #todo needs cleanup.
    // #todo does not support leading tabs.
    // #todo find better name, `scan_indented_string`.
    // #todo support 'raw' strings, e.g. (write #raw "this is \ cool")
    /// Scans a multi-string 'layout'.
    fn scan_text(&mut self, indent: u64) -> Option<String> {
        let mut string = String::new();

        let mut quote_count = 0;

        loop {
            let Some(ch) = self.next_char() else {
                let mut error = Error::new(ErrorVariant::UnterminatedString);
                error.push_note("Text string is not closed", Some(self.current_range())); // #todo refine the text.
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
            } else {
                if quote_count > 0 {
                    // Insert to the string 'non-text-delimiter' double quotes.

                    for _ in 0..quote_count {
                        string.push('"');
                    }

                    quote_count = 0;
                }

                if is_eol(ch) {
                    for _ in 0..indent {
                        let Some(ch1) = self.next_char() else {
                            let mut error = Error::new(ErrorVariant::UnterminatedString);
                            error
                                .push_note("Text string is not closed", Some(self.current_range())); // #todo refine the text.
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
                }
            }

            string.push(ch);
        }

        Some(string)
    }

    fn scan_annotation(&mut self) -> Option<String> {
        let mut ann = String::new();

        let mut paren_nesting = 0;
        let mut curly_nesting = 0;

        // #todo only allow one level of nesting?

        loop {
            let Some(ch) = self.next_char() else {
                break;
            };

            if ch == '(' {
                paren_nesting += 1;
            } else if ch == ')' {
                paren_nesting -= 1;
            } else if ch == '{' {
                curly_nesting += 1;
            } else if ch == '}' {
                curly_nesting -= 1;
            } else if paren_nesting == 0 && curly_nesting == 0 && (is_whitespace(ch) || is_eol(ch))
            {
                self.put_back_char(ch);
                break;
            }

            ann.push(ch);
        }

        if paren_nesting != 0 {
            let mut error = Error::new(ErrorVariant::UnterminatedAnnotation);
            error.push_note(
                "annotation is not closed, missing closing `)`",
                Some(self.current_range()),
            ); // #todo refine the text.
            self.errors.push(error);
            None
        } else if curly_nesting != 0 {
            let mut error = Error::new(ErrorVariant::UnterminatedAnnotation);
            error.push_note(
                "annotation is not closed, missing closing `}`",
                Some(self.current_range()),
            ); // #todo refine the text.
            self.errors.push(error);
            None
        } else {
            Some(ann)
        }
    }

    fn scan_number(&mut self) -> String {
        let lexeme = self.scan_lexeme();

        // Ignore `_`, it is considered a number separator.
        // #insight do _not_ consider `,` as number separator, bad idea!
        lexeme.replace('_', "")
    }

    fn lex_number_or_range(&mut self) -> Token {
        let lexeme = self.scan_number();

        if is_range_literal(&lexeme) {
            Token::symbol(lexeme, self.current_range())
        } else {
            Token::number(lexeme, self.current_range())
        }
    }

    // #todo consider passing into array of chars or something more general.
    pub fn lex(&mut self) -> Result<Vec<Token>, Vec<Error>> {
        let mut tokens: Vec<Token> = Vec::new();

        let mut previous_token_line = 0;

        'outer: loop {
            self.start_position = self.current_position;

            let Some(ch) = self.next_char() else {
                break 'outer;
            };

            match ch {
                '(' => {
                    tokens.push(Token::new(TokenKind::LeftParen, self.current_range()));
                }
                ')' => {
                    tokens.push(Token::new(TokenKind::RightParen, self.current_range()));
                }
                // #todo maybe should just rewrite [..] -> (Array ..)
                '[' => {
                    tokens.push(Token::new(TokenKind::LeftBracket, self.current_range()));
                }
                ']' => {
                    tokens.push(Token::new(TokenKind::RightBracket, self.current_range()));
                }
                // #todo maybe should just rewrite {..} -> (Map ..)
                '{' => {
                    tokens.push(Token::new(TokenKind::LeftBrace, self.current_range()));
                }
                '}' => {
                    tokens.push(Token::new(TokenKind::RightBrace, self.current_range()));
                }
                ';' => {
                    // #insight
                    // We only support `;` line comments and not `--` line comments
                    // The `--` line comments don't play well with the use of `-`
                    // as word separator in names.
                    self.put_back_char(ch);
                    let lexeme = self.scan_line();

                    // #todo temp solution.
                    let comment_kind = if self.current_position.line == previous_token_line {
                        CommentKind::Inline
                    } else {
                        CommentKind::Line
                    };

                    tokens.push(Token::comment(lexeme, self.current_range(), comment_kind));
                }
                '\'' => {
                    // #todo consider `:`
                    tokens.push(Token::new(TokenKind::Quote, self.current_range()));
                }
                '$' => {
                    // #insight unquoting is interpolation.
                    tokens.push(Token::new(TokenKind::Unquote, self.current_range()));
                }
                '"' => {
                    let Some(ch1) = self.next_char() else {
                        let mut error = Error::new(ErrorVariant::UnterminatedString);
                        error.push_note("String is not closed", Some(self.current_range())); // #todo refine the text.
                        self.errors.push(error);
                        break 'outer;
                    };

                    // #todo consider using ``` instead of """, like commonmark.
                    // #todo what is a good name? Text? IndentedString?
                    // Check for `"""` triple-quote multi-line string delimiter.
                    if ch1 == '"' {
                        if let Some(ch2) = self.next_char() {
                            if ch2 == '"' {
                                // Compute the indentation.
                                let mut indent = 0;

                                loop {
                                    let Some(ch3) = self.next_char() else {
                                        let mut error =
                                            Error::new(ErrorVariant::UnterminatedString);
                                        error.push_note(
                                            "Text string is not closed",
                                            Some(self.current_range()),
                                        ); // #todo refine the text.
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

                                let Some(lexeme) = self.scan_text(indent) else {
                                    break;
                                };
                                // #todo
                                // custom token needed or some kind of annotation so that tools like
                                // the formatter can apply custom formatting? or the formatter can just check
                                // if the range starts with '"""'?
                                tokens.push(Token::string(lexeme, self.current_range()));

                                continue;
                            }
                            self.put_back_char(ch2);
                        };
                    }

                    self.put_back_char(ch1);

                    let Some(lexeme) = self.scan_string() else {
                        break;
                    };
                    tokens.push(Token::string(lexeme, self.current_range()));
                }
                '-' => {
                    let Some(ch1) = self.next_char() else {
                        let mut error = Error::new(ErrorVariant::UnexpectedEnd);
                        error.push_note("Text string is not closed", Some(self.current_range())); // #todo refine the text.
                        self.errors.push(error);
                        break 'outer;
                    };

                    self.put_back_char(ch1);
                    self.put_back_char(ch);

                    // #insight `--` line comments no longer supported.
                    // if ch1 == '-' {
                    //     // #todo if we switch to kebab-case, `--` comments may cause issues.
                    //     // `--` line comment
                    //     let line = self.scan_line();
                    //     tokens.push(Ranged(Token::Comment(line), self.range()));
                    // } else
                    if ch1.is_numeric() {
                        // Negative number or range
                        tokens.push(self.lex_number_or_range());
                    } else {
                        // #todo lint warning for this!
                        // Symbol starting with `-`.
                        let lexeme = self.scan_lexeme();
                        tokens.push(Token::symbol(lexeme, self.current_range()));
                    }
                }
                '#' => {
                    if self.current_position.index == 1 {
                        if let Some(ch1) = self.next_char() {
                            if ch1 == '!' {
                                // Shebang line detected, skip.
                                let _ = self.scan_line();
                                // #todo should we keep the shebang as a module annotation?
                                continue;
                            } else {
                                self.put_back_char(ch1);
                            }
                        }
                    }
                    let Some(lexeme) = self.scan_annotation() else {
                        break 'outer;
                    };
                    tokens.push(Token::annotation(lexeme, self.current_range()));
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
                            self.current_range(),
                        ));
                    }
                }
                _ if ch.is_numeric() => {
                    self.put_back_char(ch);
                    tokens.push(self.lex_number_or_range());
                }
                _ => {
                    self.put_back_char(ch);
                    let lexeme = self.scan_lexeme();
                    tokens.push(Token::symbol(lexeme, self.current_range()));
                }
            }

            previous_token_line = self.current_position.line;
        }

        if self.errors.is_empty() {
            // #todo
            Ok(tokens)
        } else {
            let errors = std::mem::take(&mut self.errors);
            Err(errors)
        }
    }
}
