pub mod util;

use rust_decimal::prelude::*;
use util::desugar_key_path;

use crate::{
    error::{Error, ErrorVariant},
    expr::{annotate, annotate_range, format_value, Expr},
    lexer::{
        token::{Token, TokenKind},
        Lexer,
    },
    parser::util::recognize_string_template,
    range::{Position, Range},
    util::{is_range_literal, is_type, put_back_iterator::PutBackIterator, Break},
};

use self::util::{is_key_symbol, recognize_range};

// #todo Implement separate, analysis parser, keeps comments, annotations, etc.
// #todo Or implement an 'analysis mode', and remove the prune stage.

// #todo No need to keep iterator as state in parser!
// #todo Can the parser be just a function? -> yes, if we use a custom iterator to keep the parsing state.
// #todo Think some more how annotations should be handled.
// #todo Annotations are not correctly attributed to lists

// #insight
// The syntax of the language is explicitly designed to _not_ require a lookahead buffer.

// #insight
// We move the tokens into the parser to simplify the code. The tokens are useless outside the parser.

// #todo Can we remove the generics shenanigans form Parser?

/// The Parser performs the syntactic analysis stage of the compilation pipeline.
/// The input token stream is reduced into an Abstract Syntax Tree (AST).
/// The nodes of the AST are associated with annotations.
pub struct Parser<'a> {
    tokens: PutBackIterator<'a, Token>,
    buffered_annotations: Option<Vec<&'a Token>>,
    current_position: Position,
    errors: Vec<Error>,
    // If true, the parser is running in 'Analysis' mode.
    analysis: bool,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: PutBackIterator::new(tokens),
            buffered_annotations: None,
            current_position: Position::default(),
            errors: Vec::new(),
            analysis: false,
        }
    }

    pub fn for_analysis(tokens: &'a [Token]) -> Self {
        Self {
            tokens: PutBackIterator::new(tokens),
            buffered_annotations: None,
            current_position: Position::default(),
            errors: Vec::new(),
            analysis: true,
        }
    }

    pub fn with_position(self, position: Position) -> Self {
        Self {
            current_position: position,
            ..self
        }
    }

    fn next_token(&mut self) -> Option<&'a Token> {
        let maybe_token = self.tokens.next();

        if let Some(token) = maybe_token {
            self.current_position = token.range().end;
        }

        maybe_token
    }

    fn put_back_token(&mut self, token: &'a Token) {
        self.current_position = token.range().start;
        self.tokens.put_back(token);
    }

    // #todo When attaching the annotations ignore some Exprs like Comment, etc.
    /// Wrap the `expr` with the buffered (prefix) annotations. The annotations
    /// are parsed into an Expr representation. Also attaches the range of the
    /// expression as an annotation.
    fn attach_buffered_annotations(&mut self, expr: Expr, range: Range) -> Expr {
        // #todo Consider not annotating transients with range?
        // Annotate the expression with the range, by default.
        let mut expr = annotate_range(expr, range);

        if expr.is_transient() {
            // Annotations should 'pass through' transient expressions.
            return expr;
        }

        let Some(buffered_annotations) = self.buffered_annotations.take() else {
            // No annotations for the expression.
            return expr;
        };

        for annotation_token in buffered_annotations {
            let input = annotation_token.lexeme();

            let Some(first_char) = input.chars().next() else {
                // #todo Can this happen?
                // #todo Emit warning/error?
                continue;
            };

            // #todo Hackish way to detect a type expression.
            let is_type_expression = first_char.is_uppercase() || first_char == '(';

            let mut lexer = Lexer::new(&input);

            let Ok(tokens) = lexer.lex() else {
                let mut error = Error::new(ErrorVariant::MalformedAnnotation);
                error.push_note(
                    &format!(
                        "lexical error in annotation `{}`",
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
                let mut error = Error::new(ErrorVariant::MalformedAnnotation);
                error.push_note(
                    &format!("parse error in annotation `{}`", annotation_token.lexeme()),
                    Some(annotation_token.range()),
                );
                self.errors.push(error);
                // Also append the annotation parsing errors.
                self.errors.append(&mut errors);
                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                return expr;
            }

            // #todo what is this?
            // #todo temp, support multiple expressions in annotation?
            let ann_expr = ann_expr.unwrap().swap_remove(0);

            let ann_expr = ann_expr.unpack();

            match &ann_expr {
                Expr::Type(..) => {
                    // #insight
                    // Type shorthand: If the annotation starts with uppercase
                    // letter, it's considered a type annotation.
                    // #insight
                    // Don't use `:=` for type declarations, it's the assignment operator.
                    expr = annotate(expr, "type", ann_expr.clone());
                }
                Expr::Symbol(sym) => {
                    // #insight
                    // Type shorthand: If the annotation consists of a single
                    // symbol, it's considered a boolean annotation.
                    if sym.is_empty() {
                        let mut error = Error::new(ErrorVariant::MalformedAnnotation);
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

                    expr = annotate(expr, sym.clone(), Expr::Bool(true));
                }
                Expr::List(list) => {
                    // #todo problem {...} is still (Map ...) at this point!
                    // #insight a 'List' annotation always represents a type!
                    // #todo Validate that the list is a correct type expression.
                    // #todo Also handle parameterized types.
                    // #todo Support more than symbols, e.g. KeySymbols or Strings.

                    let Some(..) = list.first() else {
                        // #inside Empty annotation is considered as type annotation to the unit type?
                        // #todo It makes no sense, the annotation should just be ignored.
                        // #todo Throw a warning?
                        expr = annotate(expr, "type", Expr::None);
                        return expr;
                    };

                    if is_type_expression {
                        // #todo #IMPORTANT verify that the type expression is valid
                        // #todo investigate if some part of the annotation is missing from ann_expr!
                        expr = annotate(expr, "type", Expr::Type(format_value(ann_expr)));
                    } else {
                        let Some(ann_list) = ann_expr.as_list() else {
                            let mut error = Error::new(ErrorVariant::MalformedAnnotation);
                            error.push_note(
                                &format!("cannot parse `{}`", annotation_token.lexeme()),
                                Some(annotation_token.range()),
                            );
                            self.errors.push(error);
                            // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                            return expr;
                        };
                        let mut i = 1;
                        while i < ann_list.len() {
                            // #todo add error checking here!
                            let Some(k) = ann_list[i].as_stringable() else {
                                let mut error = Error::new(ErrorVariant::MalformedAnnotation);
                                error.push_note(
                                    &format!(
                                        "the annotation key should be a stringable `{}`",
                                        annotation_token.lexeme()
                                    ),
                                    Some(annotation_token.range()),
                                );
                                self.errors.push(error);
                                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                                return expr;
                            };
                            let v = ann_list[i + 1].clone();
                            expr = annotate(expr, k, v);
                            i += 2;
                        }
                    }
                }
                _ => {
                    let mut error = Error::new(ErrorVariant::MalformedAnnotation);
                    error.push_note(
                        &format!(
                            "an annotation should be either a symbol or a list `{}`",
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
            // #todo add detailed description.
            TokenKind::String(lexeme) => {
                if lexeme.contains("${") {
                    // #insight
                    // The transformation is handled at a later stage to not affect
                    // formatting. Only error checking happens here.

                    // #todo this should happen at a later stage to not affect formatting!
                    match recognize_string_template(lexeme, start_position) {
                        // Ok(format_expr) => Some(format_expr),
                        // #insight no transformation performed here.
                        Ok(_) => Some(Expr::String(lexeme.clone())),
                        Err(errs) => {
                            let mut error = Error::new(ErrorVariant::MalformedStringTemplate);
                            for err in errs {
                                error.push_note(&err.to_string(), Some(range.clone()));
                            }
                            self.errors.push(error);
                            None
                        }
                    }
                } else {
                    Some(Expr::String(lexeme.clone()))
                }
            }
            TokenKind::Symbol(lexeme) => {
                if lexeme.contains(':') {
                    if is_key_symbol(lexeme) {
                        // #todo Do not support ':' at the end.
                        // #todo Consider forcing `:` at the end or beginning? don't use as separators?
                        // #todo Consider converting to (quote (Symbol ...))? KeySymbol is slightly faster?
                        let sym = str::replace(lexeme, ":", "");
                        // #todo Consider Expr::Key instead of Expr::KeySymbol
                        Some(Expr::KeySymbol(sym))
                    } else {
                        // #todo The desugaring should happen later, so that
                        // it can be skipped in formatting.

                        // The lexeme is a key-path.
                        // #insight The key-path segments can only be strings and ints.
                        // #todo #IMPORTANT Better error-handling is needed here.
                        Some(desugar_key_path(lexeme))
                    }
                } else if is_type(lexeme) {
                    Some(Expr::Type(lexeme.into()))
                } else if lexeme == "true" {
                    // #todo Consider using (True) for true 'literal', e.g. (let flag (True))
                    // #todo Bool = True + False = True | False = ~False | False
                    Some(Expr::Bool(true))
                } else if lexeme == "false" {
                    // #todo Consider using False for false 'literal'.
                    // #todo Consider having only true (and use something like nil for false)
                    // #todo Consider using nil for false and everything else for true
                    // #todo Consider using nothing/never for false and everything else for true.
                    Some(Expr::Bool(false))
                } else if is_range_literal(lexeme) {
                    // #todo cleanup.
                    // #todo consider accepting as range `end/step`, without the `..` spread.
                    // #todo validate a range (e.g. only one .., no other random chars)
                    match recognize_range(lexeme) {
                        Some(r) => Some(r),
                        None => {
                            let mut error = Error::new(ErrorVariant::MalformedRange);
                            // #todo support range in error, not only in note.
                            error.push_note("invalid range", Some(range));
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

                // #todo Consider accepting scientific notation numbers without digit as floats.

                // #todo more detailed Number error!
                // #todo error handling not enough, we need to add context, check error_stack
                if lexeme.contains('.') {
                    // #todo support radix for non-integers?

                    if lexeme.ends_with('d') {
                        // Numbers ending with a 'd' postfix are Dec (decimal) literals.
                        // #todo A proper regular expression to match decimals is needed.

                        let lexeme = &lexeme[0..lexeme.len() - 1];
                        match Decimal::from_str(lexeme) {
                            Ok(num) => Some(Expr::Dec(num)),
                            Err(dec_error) => {
                                let mut error = Error::new(ErrorVariant::MalformedFloat); // #todo Introduce MalformedDec?
                                error.push_note(&format!("{dec_error}"), Some(range));
                                self.errors.push(error);
                                None
                            }
                        }
                    } else {
                        // #todo find a better name for 'non-integer'.
                        match lexeme.parse::<f64>() {
                            Ok(num) => Some(Expr::Float(num)),
                            Err(pf_error) => {
                                let mut error = Error::new(ErrorVariant::MalformedFloat);
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
                            let mut error = Error::new(ErrorVariant::MalformedInt);
                            error.push_note(&format!("{pi_error}"), Some(range));
                            self.errors.push(error);
                            None
                        }
                    }
                }
            }
            TokenKind::Annotation(_) => {
                // #todo Support multiple annotations, e.g. `#[(min 1) (max 2)]`

                if self.buffered_annotations.is_none() {
                    self.buffered_annotations = Some(Vec::new());
                }

                self.buffered_annotations.as_mut().unwrap().push(token);

                if self.analysis {
                    Some(Expr::Annotation(token.lexeme()))
                } else {
                    None
                }
            }
            TokenKind::Quote => {
                // #insight in the parser we just replace the quoting sigil with a `quot` function invocation
                // #todo maybe this should happen in the lexer?
                // #insight we should allow consecutive quotes, emit a linter warning instead!

                let Ok(quot_expr) = self.parse_expr() else {
                    // Parsing the quoted expression failed.
                    // Continue parsing to detect more errors.
                    let mut error = Error::new(ErrorVariant::InvalidQuote);
                    error.push_note("Cannot parse quoted expression", Some(token.range()));
                    self.errors.push(error);
                    return Ok(None);
                };

                let Some(target) = quot_expr else {
                    let mut error = Error::new(ErrorVariant::InvalidQuote);
                    error.push_note("Invalid quoted expression", Some(token.range()));
                    self.errors.push(error);
                    // It is recoverable error.
                    return Ok(None);
                };

                // #todo the actual quoting should be handled here?
                // #todo what about interpolation?

                Some(Expr::List(vec![Expr::symbol("quot"), target]))
            }
            TokenKind::Unquote => {
                // #insight in the parser we just replace the unquoting sigil with an `unquot` function invocation
                // #todo maybe this should happen in the lexer?

                let Ok(quot_expr) = self.parse_expr() else {
                    // Parsing the quoted expression failed.
                    // Continue parsing to detect more errors.
                    let mut error = Error::new(ErrorVariant::InvalidQuote);
                    error.push_note("Cannot parse unquoted expression", Some(token.range()));
                    self.errors.push(error);
                    return Ok(None);
                };

                let Some(target) = quot_expr else {
                    let mut error = Error::new(ErrorVariant::InvalidQuote);
                    error.push_note("Invalid unquoted expression", Some(token.range()));
                    self.errors.push(error);
                    // It is recoverable error.
                    return Ok(None);
                };

                // #todo the actual quoting should be handled here?
                // #todo what about interpolation?

                Some(Expr::List(vec![Expr::symbol("unquot"), target]))
            }
            TokenKind::LeftParen => {
                let terms = self.parse_many(TokenKind::RightParen, start_position)?;

                if terms.is_empty() {
                    // #todo do we _really_ want this or just return a list?
                    // #insight this was causing a problem with Func with no params.
                    // #todo maybe as_list should return empty list?
                    // `()` == One/Unit   (unit is _not_ Top)
                    Some(Expr::None)
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

                // #todo maybe should be Expr::typ?
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
                // Syntactic sugar for a Map.

                // #insight
                // Don't optimize to `Expr::Map` here, leave the parser expr
                // 'normalized as it is beneficial for some kinds of analysis.

                // #todo add error checking!
                // #todo optimize.

                // #insight error checking and inference will happen in (Map ...) eval.

                let exprs = self.parse_many(TokenKind::RightBrace, start_position)?;

                // #todo maybe should be Expr::typ?
                let mut items = vec![annotate_range(Expr::symbol("Map"), range)];

                for expr in exprs {
                    items.push(expr);
                }

                Some(Expr::List(items))
            }
            TokenKind::RightParen | TokenKind::RightBracket | TokenKind::RightBrace => {
                // #todo custom error for this?
                let mut error = Error::new(ErrorVariant::UnexpectedToken);
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
                let mut error = Error::new(ErrorVariant::UnterminatedList);
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
