#[cfg(feature = "dec")]
use rust_decimal::prelude::*;

use crate::{
    error::{Error, ErrorKind},
    expr::{annotate, annotate_range, Expr},
    lexer::{
        token::{Token, TokenKind},
        Lexer,
    },
    range::{Position, Range},
    util::{put_back_iterator::PutBackIterator, Break},
};

// #todo no need to keep iterator as state in parser!
// #todo can the parser be just a function? -> yes, if we use a custom iterator to keep the parsing state.
// #todo think some more how annotations should be handled.
// #todo annotations are not correctly attributed to lists

// #insight
// The syntax of the language is explicitly designed to _not_ require a lookahead buffer.

// #insight
// We move the tokens into the parser to simplify the code. The tokens are useless outside the parser.

// #todo can we remove the generics shenanigans form Parser?

// #ai-generated
// #todo cleanup the implementation.
// #todo move to another file.
/// Parses a range string: start..end/step.
fn split_range(range_str: &str) -> Option<Expr> {
    let parts: Vec<&str> = range_str.split('/').collect();

    if parts.len() > 2 {
        // If there are more than two parts, the input format is invalid
        return None;
    }

    // Parse the start and end values from the first part of the split
    let start_end: Vec<&str> = parts[0].split("..").collect();
    if start_end.len() != 2 {
        // If the start-end part doesn't have exactly two elements, the input format is invalid
        return None;
    }

    if start_end[0].contains(".") {
        let start: f64 = start_end[0].parse().ok()?;
        let end: f64 = start_end[1].parse().ok()?;

        // Default step value is 1.0 if not provided
        let step: f64 = if parts.len() == 2 {
            parts[1].parse().ok()?
        } else {
            1.0
        };

        Some(Expr::FloatRange(start, end, step))
    } else {
        let start: i64 = start_end[0].parse().ok()?;
        let end: i64 = start_end[1].parse().ok()?;

        // Default step value is 1 if not provided
        let step: i64 = if parts.len() == 2 {
            parts[1].parse().ok()?
        } else {
            1
        };

        Some(Expr::IntRange(start, end, step))
    }
}

/// A key is considered a `KeySymbol` (aka 'keyword') if it contains a collon. A collon
/// can be at the end or at the beginning, or even in the middle of the lexeme.
/// A `KeySymbol` always evaluates to itself.
#[inline(always)]
pub fn is_key_symbol(lexeme: &str) -> bool {
    lexeme.contains(':')
}

/// The Parser performs the syntactic analysis stage of the compilation pipeline.
/// The input token stream is reduced into and Abstract Syntax Tree (AST).
/// The nodes of the AST are associated with annotations.
pub struct Parser<'a> {
    tokens: PutBackIterator<'a, Token>,
    buffered_annotations: Option<Vec<&'a Token>>,
    current_position: Position,
    errors: Vec<Error>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: PutBackIterator::new(tokens),
            buffered_annotations: None,
            current_position: Position::default(),
            errors: Vec::new(),
        }
    }

    fn next_token(&mut self) -> Option<&'a Token> {
        let maybe_token = self.tokens.next();

        if let Some(ref token) = maybe_token {
            self.current_position = token.range().end;
        }

        maybe_token
    }

    fn put_back_token(&mut self, token: &'a Token) {
        self.current_position = token.range().start;
        self.tokens.put_back(token);
    }

    /// Wrap the `expr` with the buffered (prefix) annotations. The annotations
    /// are parsed into an Expr representation. Also attaches the range of the
    /// expression as an annotation.
    fn attach_buffered_annotations(&mut self, expr: Expr, range: Range) -> Expr {
        // Annotate the expression with the range, by default.
        let mut expr = annotate_range(expr, range);

        let Some(buffered_annotations) = self.buffered_annotations.take() else {
            // No annotations for the expression.
            return expr;
        };

        for annotation_token in buffered_annotations {
            let input = annotation_token.lexeme();
            let mut lexer = Lexer::new(&input);

            let Ok(tokens) = lexer.lex() else {
                let mut error = Error::new(ErrorKind::MalformedAnnotation);
                error.push_note(
                    &format!(
                        "Lexical error in annotation `{}`",
                        annotation_token.lexeme()
                    ),
                    Some(annotation_token.range()),
                );
                self.errors.push(error);
                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                return expr;
            };

            let mut parser = Parser::new(&tokens);

            let ann_expr = parser.parse();

            if let Err(mut errors) = ann_expr {
                let mut error = Error::new(ErrorKind::MalformedAnnotation);
                error.push_note(
                    &format!("Parse error in annotation `{}`", annotation_token.lexeme()),
                    Some(annotation_token.range()),
                );
                self.errors.push(error);
                // Also append the annotation parsing errors.
                self.errors.append(&mut errors);
                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                return expr;
            }

            // #todo temp, support multiple expressions in annotation?
            let ann_expr = ann_expr.unwrap().swap_remove(0);

            let ann_expr = ann_expr.unpack();

            match &ann_expr {
                Expr::Symbol(sym) => {
                    if sym.is_empty() {
                        let mut error = Error::new(ErrorKind::MalformedAnnotation);
                        error.push_note(
                            &format!(
                                "Invalid single-symbol annotation`{}`",
                                annotation_token.lexeme()
                            ),
                            Some(annotation_token.range()),
                        );
                        self.errors.push(error);
                        // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                        return expr;
                    }

                    if sym.chars().next().unwrap().is_uppercase() {
                        // Type shorthand: If the annotation starts with uppercase
                        // letter, it's considered type annotations.
                        expr = annotate(expr, "type", ann_expr.clone());
                    } else {
                        // Bool=true shorthand: If the annotation starts with lowercase
                        // letter, it's considered a boolean flag.
                        expr = annotate(expr, sym.clone(), Expr::Bool(true));
                    }
                }
                Expr::List(list) => {
                    // #todo support more than symbols, e.g. KeySymbols or Strings.
                    if let Some(Expr::Symbol(sym)) = list.first().map(|x| x.unpack()) {
                        expr = annotate(expr, sym.clone(), ann_expr.clone());
                    } else {
                        let mut error = Error::new(ErrorKind::MalformedAnnotation);
                        error.push_note(
                            &format!(
                                "First term must be a symbol `{}`",
                                annotation_token.lexeme()
                            ),
                            Some(annotation_token.range()),
                        );
                        self.errors.push(error);
                        // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                        return expr;
                    }
                }
                _ => {
                    let mut error = Error::new(ErrorKind::MalformedAnnotation);
                    error.push_note(
                        &format!(
                            "An annotation should be either a symbol or a list `{}`",
                            annotation_token.lexeme()
                        ),
                        Some(annotation_token.range()),
                    );
                    self.errors.push(error);
                    // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                    return expr;
                }
            }
        }

        expr
    }

    pub fn parse_expr(&mut self) -> Result<Option<Expr>, Break> {
        let Some(token) = self.next_token() else {
            return Err(Break {});
        };

        let range = token.range();

        let start_position = range.start;

        let expr = match token.kind() {
            TokenKind::Comment(lexeme, comment_kind) => {
                // Preserve the comments as expressions, may be useful for analysis passes (e.g. formatting)
                // Comments are elided statically, before the evaluation pass.
                Some(Expr::Comment(lexeme.clone(), *comment_kind))
            }
            TokenKind::MultiLineWhitespace => {
                // Preserve for formatter, will be elided statically, before the
                // evaluation pass.
                Some(Expr::TextSeparator)
            }
            // Token::Char(c) => Some(Expr::Char(c)),
            // #todo handle strings with interpolation (String-Template)
            // #todo javascript-style templated/tagged string, with key at the end.
            TokenKind::String(lexeme) => Some(Expr::String(lexeme.clone())),
            TokenKind::Symbol(lexeme) => {
                if is_key_symbol(lexeme) {
                    // #todo do not support ':' at the end.
                    // #todo consider forcing `:` at the end or beginning? don't use as separators?
                    // #todo consider converting to (quote (Symbol ...))? KeySymbol is slightly faster?
                    let sym = str::replace(lexeme, ":", "");
                    // #todo consider Expr::Key instead of Expr::KeySymbol
                    Some(Expr::KeySymbol(sym))
                } else if lexeme == "true" {
                    // #todo consider using (True) for true 'literal'.
                    // #todo e.g. (let flag (True))
                    // #todo Bool = True + False = True | False = ~False | False
                    Some(Expr::Bool(true))
                } else if lexeme == "false" {
                    // #todo consider using False for false 'literal'.
                    // #todo consider having only true (and use something like nil for false)
                    // #todo consider using nil for false and everything else for true
                    // #todo consider using nothing/never for false and everything else for true.
                    Some(Expr::Bool(false))
                } else if lexeme.contains("..") {
                    // #todo cleanup.
                    // #todo consider accepting as range `end/step`, without the `..` spread.
                    // #todo validate a range (e.g. only one .., no other random chars)
                    match split_range(lexeme) {
                        Some(r) => Some(r),
                        None => {
                            let error = Error::new(ErrorKind::MalformedRange);
                            self.errors.push(error);
                            None
                        }
                    }
                } else {
                    Some(Expr::Symbol(lexeme.clone()))
                }
            }
            TokenKind::Number(lexeme) => {
                let mut lexeme = lexeme.clone();

                // #todo more detailed Number error!
                // #todo error handling not enough, we need to add context, check error_stack
                if lexeme.contains('.') {
                    // #todo support radix for non-integers?

                    if lexeme.ends_with("d") {
                        // numbers ending with a 'd' postfix are Dec (decimal) literals.
                        // #todo a proper regular expression to match decimals is needed.

                        #[cfg(feature = "dec")]
                        {
                            let lexeme = &lexeme[0..lexeme.len() - 1];
                            match Decimal::from_str(lexeme) {
                                Ok(num) => Some(Expr::Dec(num)),
                                Err(dec_error) => {
                                    let mut error = Error::new(ErrorKind::MalformedFloat); // #todo introduce MalformedDec?
                                    error.push_note(&format!("{dec_error}"), Some(range));
                                    self.errors.push(error);
                                    None
                                }
                            }
                        }

                        // #todo return error instead!
                        #[cfg(not(feature = "dec"))]
                        panic!("Dec not supported in this build");
                    } else {
                        // #todo find a better name for 'non-integer'.
                        match lexeme.parse::<f64>() {
                            Ok(num) => Some(Expr::Float(num)),
                            Err(pf_error) => {
                                let mut error = Error::new(ErrorKind::MalformedFloat);
                                error.push_note(&format!("{pf_error}"), Some(range));
                                self.errors.push(error);
                                None
                            }
                        }
                    }
                } else {
                    // #todo support arbitrary radix https://github.com/golang/go/issues/28256
                    let mut radix = 10;

                    if lexeme.starts_with("0x") {
                        lexeme = lexeme.replace("0x", "");
                        radix = 16
                    } else if lexeme.starts_with("0b") {
                        lexeme = lexeme.replace("0b", "");
                        radix = 2
                    } else if lexeme.starts_with("0o") {
                        // #insight Octal literals are supported for historical reasons.
                        lexeme = lexeme.replace("0o", "");
                        radix = 8
                    }

                    match i64::from_str_radix(&lexeme, radix) {
                        Ok(n) => Some(Expr::Int(n)),
                        Err(pi_error) => {
                            let mut error = Error::new(ErrorKind::MalformedInt);
                            error.push_note(&format!("{pi_error}"), Some(range));
                            self.errors.push(error);
                            None
                        }
                    }
                }
            }
            TokenKind::Annotation(_) => {
                // #todo support multiple annotations, e.g. `#[(min 1) (max 2)]`

                if self.buffered_annotations.is_none() {
                    self.buffered_annotations = Some(Vec::new());
                }

                self.buffered_annotations.as_mut().unwrap().push(token);

                None
            }
            TokenKind::Quote => {
                // #insight we should allow consecutive quotes, emit a linter warning instead!

                let Ok(quot_expr) = self.parse_expr() else {
                    // Parsing the quoted expression failed.
                    // Continue parsing to detect more errors.
                    let mut error = Error::new(ErrorKind::InvalidQuote);
                    error.push_note("Cannot parse quoted expression", Some(token.range()));
                    self.errors.push(error);
                    return Ok(None);
                };

                let Some(target) = quot_expr else {
                    let mut error = Error::new(ErrorKind::InvalidQuote);
                    error.push_note("Invalid quoted expression", Some(token.range()));
                    self.errors.push(error);
                    // It is recoverable error.
                    return Ok(None);
                };

                // #todo the actual quoting should be handled here?
                // #todo what about interpolation?

                Some(Expr::List(vec![Expr::symbol("quot").into(), target]))
            }
            TokenKind::LeftParen => {
                let terms = self.parse_many(TokenKind::RightParen, start_position)?;

                if terms.is_empty() {
                    // #todo do we _really_ want this or just return a list?
                    // `()` == One/Unit/Top
                    Some(Expr::One)
                } else {
                    Some(Expr::List(terms))

                    // #todo optimize some special forms but in another comptime pass.

                    // let head = list_exprs[0].clone();
                    // match head {
                    //     // #todo optimize more special forms.

                    //     // #insight
                    //     // Parsing built-ins as Exprs optimizes runtime evaluation, with more efficient
                    //     // matching.

                    //     // `if` expression
                    //     Ann(Expr::Symbol(s), ..) if s == "if" => {
                    //         // #todo detailed checking and error-reporting
                    //         Some(Expr::If(
                    //             Box::new(list_exprs[1].clone()),
                    //             Box::new(list_exprs[2].clone()),
                    //             // #todo optional parsing!
                    //             Some(Box::new(list_exprs[3].clone())),
                    //         ))
                    //     }
                    //     _ => Some(Expr::List(list_exprs)),
                    // }
                }
            }
            TokenKind::LeftBracket => {
                // Syntactic sugar for a List/Array.

                // #insight
                // Don't optimize to `Expr::Array` here, leave the parser expr
                // 'normalized as it is beneficial for some kinds of analysis.

                let exprs = self.parse_many(TokenKind::RightBracket, start_position)?;

                let mut items = vec![annotate_range(Expr::symbol("Array"), range)];

                // #todo add error checking!
                // #todo optimize.
                // #todo evaluate the list_exprs

                for expr in exprs {
                    items.push(expr);
                }

                Some(Expr::List(items))
            }
            TokenKind::LeftBrace => {
                // Syntactic sugar for a Dict.

                // #insight
                // Don't optimize to `Expr::Dict` here, leave the parser expr
                // 'normalized as it is beneficial for some kinds of analysis.

                // #todo add error checking!
                // #todo optimize.

                let exprs = self.parse_many(TokenKind::RightBrace, start_position)?;

                let mut items = vec![annotate_range(Expr::symbol("Dict"), range)];

                for expr in exprs {
                    items.push(expr);
                }

                Some(Expr::List(items))
            }
            TokenKind::RightParen | TokenKind::RightBracket | TokenKind::RightBrace => {
                // #todo custom error for this?
                let mut error = Error::new(ErrorKind::UnexpectedToken);
                error.push_note(
                    &format!("Not expecting token `{}`", token.lexeme()),
                    Some(token.range()),
                );
                self.errors.push(error);
                // Parsing can continue.
                return Ok(None);
            }
        };

        match expr {
            Some(expr) => {
                let range = start_position..self.current_position;
                Ok(Some(self.attach_buffered_annotations(expr, range)))
            }
            _ => Ok(None),
        }
    }

    // #todo rename to `parse_until`?
    pub fn parse_many(
        &mut self,
        delimiter: TokenKind,
        start_position: Position,
    ) -> Result<Vec<Expr>, Break> {
        let mut exprs = Vec::new();

        loop {
            let Some(token) = self.next_token() else {
                let range = start_position..self.current_position;
                let mut error = Error::new(ErrorKind::UnterminatedList);
                error.push_note("List not terminated", Some(range));
                self.errors.push(error);
                return Err(Break {});
            };

            if *token.kind() == delimiter {
                // Will be annotated upstream.
                return Ok(exprs);
            } else {
                self.put_back_token(token);
                if let Some(e) = self.parse_expr()? {
                    exprs.push(e);
                }
            }
        }
    }

    // #insight
    // The parse function intentionally returns an 'unstructured' vector of
    // expressions instead of something like a do-block or a module. Downstream
    // functions can enforce some structure.

    // #insight
    // The loop in the parser is also useful to skip over comments.

    /// Parses the input tokens into expressions.
    /// The parser tries to return as many errors as possible.
    pub fn parse(&mut self) -> Result<Vec<Expr>, Vec<Error>> {
        let mut exprs = Vec::new();

        loop {
            let expr = self.parse_expr();

            let Ok(expr) = expr else {
                // A non-recoverable parse error was detected, stop parsing.
                break;
            };

            if let Some(expr) = expr {
                if self.errors.is_empty() {
                    exprs.push(expr);
                } else {
                    break;
                }
            }
        }

        if self.errors.is_empty() {
            Ok(exprs)
        } else {
            let errors = std::mem::take(&mut self.errors);
            Err(errors)
        }
    }
}
