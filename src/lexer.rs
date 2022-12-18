use std::str::Chars;

use crate::span::{Span, Spanned};

use self::{error::LexicalError, token::Token};

pub mod error;
pub mod token;

// https://en.wikipedia.org/wiki/Lexical_analysis

// #TODO lex_all, lex_single
// #TODO use peekable iterator instead of put_back/lookahead.
// #TODO introduce SemanticToken, with extra semantic information, _after_ parsing.
// #TODO parse signed numbers
// #TODO use annotations before number literals to set the type?
// #TODO use (doc_comment ...) for doc-comments.
// #TODO support `\ ` for escaped space in symbols.

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

/// The Lexer performs the lexical analysis stage of the compilation pipeline.
/// The input text is scanned into lexemes and then evaluated into lexical tokens.
/// The tokens are associated with spans (ranges within the input text).
pub struct Lexer<'a> {
    chars: Chars<'a>,
    index: usize,
    // #TODO use stack to support 'unlimited' lookahead?
    lookahead: Option<char>, // #TODO find better name!
}

impl<'a> Lexer<'a> {
    /// Makes a new Lexer with the given input text.
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            index: 0,
            lookahead: None,
        }
    }

    /// Returns the input text as a String.
    pub fn input(&self) -> String {
        self.chars.clone().collect()
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

    // Span is a right-open range, i.e. [start, end)
    fn span(&self, start: usize) -> Span {
        Span {
            start: start - 1,
            end: self.index,
        }
    }

    // #TODO add unit tests
    // #TODO try to reuse in more lexers!
    fn scan_lexeme(&mut self) -> Spanned<String> {
        let mut char = self.next_char();

        let start = self.index;
        let mut text = String::new();

        while let Some(ch) = char {
            // #TODO maybe whitespace does not need put_back, but need to adjust span.
            if is_whitespace(ch) || is_delimiter(ch) || is_eol(ch) {
                self.put_back_char(ch);
                break;
            }

            text.push(ch);

            char = self.next_char();
        }

        let span = self.span(start);

        Spanned::new(text, span)
    }

    fn lex_comment(&mut self) -> Result<Spanned<Token>, Spanned<LexicalError>> {
        let start = self.index;
        let mut text = String::from(";");

        let mut char = self.next_char();

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

        Ok(Spanned::new(Token::Comment(text), span))
    }

    fn lex_number(&mut self) -> Result<Spanned<Token>, Spanned<LexicalError>> {
        let Spanned {
            value: lexeme,
            span,
        } = self.scan_lexeme();

        // Ignore `_`, it is considered a number separator.
        // #Insight fo _not_ consider `,` as number separator, bad idea!
        let mut lexeme = lexeme.replace('_', "");

        // #TODO support radix-8
        // #TODO support arbitrary radix
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
            .map_err(|err| Spanned::new(LexicalError::NumberError(err), span.clone()))?;

        Ok(Spanned::new(Token::Number(n), span))
    }

    fn lex_symbol(&mut self) -> Result<Spanned<Token>, Spanned<LexicalError>> {
        let Spanned {
            value: lexeme,
            span,
        } = self.scan_lexeme();

        let token = match lexeme.as_str() {
            "if" => Token::If,
            "using" => Token::Using,
            _ => Token::Symbol(lexeme),
        };

        Ok(Spanned::new(token, span))
    }

    // #TODO support multi-line strings
    fn lex_string(&mut self) -> Result<Spanned<Token>, Spanned<LexicalError>> {
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

        let mut span = self.span(start);

        if char != Some('"') {
            span.end -= 1;
            return Err(Spanned::new(LexicalError::UnterminatedStringError, span));
        }

        Ok(Spanned::new(Token::String(text), span))
    }

    fn lex_annotation(&mut self) -> Result<Spanned<Token>, Spanned<LexicalError>> {
        let mut char = self.next_char();

        let start = self.index;
        let mut text = String::new();

        let mut nesting = 0;

        // #TODO only allow one level of nesting?

        while let Some(ch) = char {
            if ch == '(' {
                nesting += 1;
            } else if ch == ')' {
                nesting -= 1;
            } else if nesting == 0 && (is_whitespace(ch) || is_eol(ch)) {
                // #TODO maybe whitespace does not need put_back, but need to adjust span.
                self.put_back_char(ch);
                break;
            }

            text.push(ch);

            char = self.next_char();
        }

        let mut span = self.span(start);

        if nesting != 0 {
            span.start -= 1;
            span.end -= 1;
            return Err(Spanned::new(
                LexicalError::UnterminatedAnnotationError,
                span,
            ));
        }

        Ok(Spanned::new(Token::Annotation(text), span))
    }

    // #TODO consider passing into array of chars or something more general.
    pub fn lex(&mut self) -> Result<Vec<Spanned<Token>>, Spanned<LexicalError>> {
        let mut tokens: Vec<Spanned<Token>> = Vec::new();

        let mut char;

        loop {
            char = self.next_char();

            let Some(ch) = char  else {
                break;
            };

            match ch {
                '(' => {
                    tokens.push(Spanned::new(Token::LeftParen, self.span(self.index)));
                }
                ')' => {
                    tokens.push(Spanned::new(Token::RightParen, self.span(self.index)));
                }
                ';' => {
                    tokens.push(self.lex_comment()?);
                }
                '"' => {
                    tokens.push(self.lex_string()?);
                }
                '#' => {
                    tokens.push(self.lex_annotation()?);
                }
                _ if is_whitespace(ch) => {
                    // Consume whitespace
                }
                _ if ch.is_numeric() => {
                    self.put_back_char(ch);
                    tokens.push(self.lex_number()?);
                }
                _ => {
                    self.put_back_char(ch);
                    tokens.push(self.lex_symbol()?);
                }
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use std::num::IntErrorKind;

    use crate::{
        lexer::{Lexer, LexicalError, Token},
        span::Spanned,
        util::format::format_pretty_spanned_error,
    };

    fn read_input(filename: &str) -> String {
        std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
    }

    #[test]
    fn lex_handles_an_empty_string() {
        let input = read_input("empty.tan");
        let tokens = Lexer::new(&input).lex();

        let tokens = tokens.unwrap();

        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn lex_returns_tokens() {
        let input = "((+ 1   25 399)  )";
        let tokens = Lexer::new(input).lex();

        let tokens = tokens.unwrap();

        dbg!(&tokens);

        assert_eq!(tokens.len(), 8);
        assert!(matches!(tokens[0].as_ref(), Token::LeftParen));
        assert!(matches!(tokens[2].as_ref(), Token::Symbol(x) if x == "+"));
        assert_eq!(tokens[2].span.start, 2);
        assert!(matches!(tokens[3].as_ref(), Token::Number(..)));
        assert_eq!(tokens[3].span.start, 4);
        // #TODO add more assertions.
    }

    #[test]
    fn lex_parses_comments() {
        let input = "; This is a comment\n;; Another comment\n(write \"hello\")";
        let tokens = Lexer::new(input).lex();

        let tokens = tokens.unwrap();

        assert!(matches!(tokens[0].as_ref(), Token::Comment(x) if x == "; This is a comment"));
        assert!(matches!(tokens[1].as_ref(), Token::Comment(x) if x == ";; Another comment"));
    }

    #[test]
    fn lex_parses_annotations() {
        let input = "
            #deprecated
            #(inline 'always)
            (let #public (add x y) (+ x y))
        ";
        let tokens = Lexer::new(input).lex();

        let tokens = tokens.unwrap();

        assert!(matches!(tokens[0].as_ref(), Token::Annotation(x) if x == "deprecated"));
        assert!(matches!(tokens[1].as_ref(), Token::Annotation(x) if x == "(inline 'always)"));
    }

    #[test]
    fn lex_handles_number_separators() {
        let input = "(+ 1 3_000)";
        let tokens = Lexer::new(input).lex().unwrap();

        // dbg!(&tokens);

        assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == &3000));
    }

    #[test]
    fn lex_handles_number_signs() {
        // let input = "(+ 1 +3_000)";
        // let tokens = Lexer::new(input).lex().unwrap();

        // // dbg!(&tokens);

        // assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == &3000));
    }

    #[test]
    fn lex_handles_numbers_with_radix() {
        let input = "(let a 0xfe)";
        let tokens = Lexer::new(input).lex().unwrap();

        assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == &254));

        let input = "(let a 0b1010)";
        let tokens = Lexer::new(input).lex().unwrap();

        assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == &10));

        let input = "(let a 0b00000)";
        let tokens = Lexer::new(input).lex().unwrap();

        assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == &0));
    }

    #[test]
    fn lex_reports_number_errors() {
        let input = "(+ 1 3$%99)";
        let tokens = Lexer::new(input).lex();

        let result = tokens;

        assert!(result.is_err());

        let err = result.unwrap_err();

        assert!(matches!(err.value, LexicalError::NumberError(..)));

        eprintln!("{}", format_pretty_spanned_error(&err, input));

        if let Spanned {
            value: LexicalError::NumberError(pie),
            span,
        } = err
        {
            assert_eq!(pie.kind(), &IntErrorKind::InvalidDigit);
            assert_eq!(span.start, 5);
            assert_eq!(span.end, 10);
        }
    }

    #[test]
    fn lex_reports_unterminated_strings() {
        let input = r##"(write "Hello)"##;
        let tokens = Lexer::new(input).lex();

        let result = tokens;

        assert!(result.is_err());

        let err = result.unwrap_err();

        assert!(matches!(err.value, LexicalError::UnterminatedStringError));

        eprintln!("{}", format_pretty_spanned_error(&err, input));

        assert_eq!(err.span.start, 7);
        assert_eq!(err.span.end, 14);
    }

    #[test]
    fn lex_reports_unterminated_annotations() {
        let input = r##"
        #deprecated
        #(inline true
        (write "Hello)
        "##;
        let tokens = Lexer::new(input).lex();

        let result = tokens;

        assert!(result.is_err());

        let err = result.unwrap_err();

        assert!(matches!(
            err.value,
            LexicalError::UnterminatedAnnotationError
        ));

        eprintln!("{}", format_pretty_spanned_error(&err, input));

        assert_eq!(err.span.start, 29);
    }
}
