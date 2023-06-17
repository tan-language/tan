use crate::{
    ann::Ann,
    error::{Error, ErrorKind},
    expr::Expr,
    lexer::{
        token::{Token, TokenKind},
        Lexer,
    },
    range::Range,
    util::Break,
};

// #TODO no need to keep iterator as state in parser!
// #TODO can the parser be just a function? -> yes, if we use a custom iterator to keep the parsing state.
// #TODO think some more how annotations should be handled.
// #TODO annotations are not correctly attributed to lists

// #Insight
// The syntax of the language is explicitly designed to _not_ require a lookahead buffer.

// #Insight
// We move the tokens into the parser to simplify the code. The tokens are useless outside the parser.

/// The Parser performs the syntactic analysis stage of the compilation pipeline.
/// The input token stream is reduced into and Abstract Syntax Tree (AST).
/// The nodes of the AST are associated with annotations.
pub struct Parser<I>
where
    I: IntoIterator<Item = Token>,
{
    tokens: I::IntoIter,
    buffered_annotations: Option<Vec<Token>>,
    index: usize,
    lookahead: Vec<Token>,
    errors: Vec<Error>,
}

impl<I> Parser<I>
where
    I: IntoIterator<Item = Token>,
{
    pub fn new(tokens: I) -> Self {
        let tokens = tokens.into_iter();

        Self {
            tokens,
            buffered_annotations: None,
            index: 0,
            lookahead: Vec::new(),
            errors: Vec::new(),
        }
    }

    // #TODO unit test
    // #TODO refactor
    fn next_token(&mut self) -> Option<Token> {
        if let Some(token) = self.lookahead.pop() {
            self.index = token.range().end;
            return Some(token);
        }

        if let Some(token) = self.tokens.next() {
            self.index = token.range().end;
            Some(token)
        } else {
            None
        }
    }

    fn put_back_token(&mut self, token: Token) {
        self.index = token.range().start;
        self.lookahead.push(token);
    }

    /// Wrap the `expr` with the buffered (prefix) annotations. The annotations
    /// are parsed into an Expr representation. Also attaches the range of the
    /// expression as an annotation.
    fn attach_buffered_annotations(&mut self, expr: Expr, range: Range) -> Ann<Expr> {
        // Annotate the expression with the range, by default.
        let mut expr = Ann::with_range(expr, range);

        let Some(buffered_annotations) = self.buffered_annotations.take() else {
            // No annotations for the expression.
            return expr;
        };

        for annotation_token in buffered_annotations {
            let input = annotation_token.lexeme();
            let mut lexer = Lexer::new(&input);

            let Ok(tokens) = lexer.lex() else {
                let mut error = Error::new(ErrorKind::MalformedAnnotation);
                error.push_note(&format!("Lexical error in annotation `{}`", annotation_token.lexeme()), Some(annotation_token.range()));
                self.errors.push(error);
                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                return expr;
            };

            let mut parser = Parser::new(tokens);

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

            // #TODO temp, support multiple expressions in annotation?
            let ann_expr = ann_expr.unwrap().swap_remove(0);

            let ann_expr = ann_expr.0;

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
                        expr.set_annotation("type", ann_expr);
                    } else {
                        // Bool=true shorthand: If the annotation starts with lowercase
                        // letter, it's considered a boolean flag.
                        expr.set_annotation(sym.clone(), Expr::Bool(true));
                    }
                }
                Expr::List(list) => {
                    // #TODO support more than symbols, e.g. KeySymbols or Strings.
                    if let Some(Ann(Expr::Symbol(sym), _)) = list.first() {
                        expr.set_annotation(sym.clone(), ann_expr);
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

    pub fn parse_expr(&mut self) -> Result<Option<Ann<Expr>>, Break> {
        let Some(token) = self.next_token() else {
            return Err(Break {});
        };

        let range = token.range();

        let start = range.start;

        let expr = match token.kind() {
            TokenKind::Comment(lexeme) => {
                // Preserve the comments as expressions, may be useful for analysis passes (e.g. formatting)
                // Comments are elided statically, before the evaluation pass.
                Some(Expr::Comment(lexeme.clone()))
            }
            TokenKind::MultiLineWhitespace => {
                // Preserve for formatter, will be elided statically, before the
                // evaluation pass.
                Some(Expr::TextSeparator)
            }
            // Token::Char(c) => Some(Expr::Char(c)),
            TokenKind::String(lexeme) => Some(Expr::String(lexeme.clone())),
            TokenKind::Symbol(lexeme) => {
                if lexeme.starts_with(':') {
                    let sym = lexeme.strip_prefix(':').unwrap().to_string();
                    Some(Expr::KeySymbol(sym))
                } else if lexeme == "true" {
                    // #TODO consider using (True) for true 'literal'.
                    // #TODO e.g. (let flag (True))
                    // #TODO Bool = True + False = True | False = ~False | False
                    Some(Expr::Bool(true))
                } else if lexeme == "false" {
                    // #TODO consider using False for false 'literal'.
                    // #TODO consider having only true (and use something like nil for false)
                    // #TODO consider using nil for false and everything else for true
                    // #TODO consider using nothing/never for false and everything else for true.
                    Some(Expr::Bool(false))
                } else {
                    Some(Expr::Symbol(lexeme.clone()))
                }
            }
            TokenKind::Number(lexeme) => {
                let mut lexeme = lexeme.clone();

                // #TODO more detailed Number error!
                // #TODO error handling not enough, we need to add context, check error_stack
                if lexeme.contains('.') {
                    // #TODO support radix for non-integers?
                    // #TODO find a better name for 'non-integer'.
                    match lexeme.parse::<f64>() {
                        Ok(n) => Some(Expr::Float(n)),
                        Err(pf_error) => {
                            let mut error = Error::new(ErrorKind::MalformedFloat);
                            error.push_note(&format!("{pf_error}"), Some(range));
                            self.errors.push(error);
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
                // #TODO support multiple annotations, e.g. `#[(min 1) (max 2)]`

                if self.buffered_annotations.is_none() {
                    self.buffered_annotations = Some(Vec::new());
                }

                self.buffered_annotations.as_mut().unwrap().push(token);

                None
            }
            TokenKind::Quote => {
                // #Insight we should allow consecutive quotes, emit a linter warning instead!

                let Ok(quot_expr) = self.parse_expr() else {
                    // Parsing the quoted expression failed.
                    // Continue parsing to detect more errors.
                    let mut error = Error::new(ErrorKind::InvalidQuote);
                    error.push_note(
                        "Cannot parse quoted expression",
                        Some(token.range()),
                    );
                    self.errors.push(error);
                    return Ok(None);
                };

                let Some(target) = quot_expr else {
                    let mut error = Error::new(ErrorKind::InvalidQuote);
                    error.push_note(
                        "Invalid quoted expression",
                        Some(token.range()),
                    );
                    self.errors.push(error);
                    // It is recoverable error.
                    return Ok(None);
                };

                // #TODO the actual quoting should be handled here?
                // #TODO what about interpolation?

                Some(Expr::List(vec![Expr::symbol("quot").into(), target]))
            }
            TokenKind::LeftParen => {
                let terms = self.parse_many(TokenKind::RightParen, start)?;

                if terms.is_empty() {
                    // #TODO do we _really_ want this or just return a list?
                    // `()` == One/Unit/Top
                    Some(Expr::One)
                } else {
                    Some(Expr::List(terms))

                    // #TODO optimize some special forms but in another comptime pass.

                    // let head = list_exprs[0].clone();
                    // match head {
                    //     // #TODO optimize more special forms.

                    //     // #Insight
                    //     // Parsing built-ins as Exprs optimizes runtime evaluation, with more efficient
                    //     // matching.

                    //     // `if` expression
                    //     Ann(Expr::Symbol(s), ..) if s == "if" => {
                    //         // #TODO detailed checking and error-reporting
                    //         Some(Expr::If(
                    //             Box::new(list_exprs[1].clone()),
                    //             Box::new(list_exprs[2].clone()),
                    //             // #TODO optional parsing!
                    //             Some(Box::new(list_exprs[3].clone())),
                    //         ))
                    //     }
                    //     _ => Some(Expr::List(list_exprs)),
                    // }
                }
            }
            TokenKind::LeftBracket => {
                // Syntactic sugar for a List/Array.

                // #Insight
                // Don't optimize to `Expr::Array` here, leave the parser expr
                // 'normalized as it is beneficial for some kinds of analysis.

                let exprs = self.parse_many(TokenKind::RightBracket, start)?;

                let mut items = vec![Ann::with_range(Expr::symbol("Array"), range)];

                // #TODO add error checking!
                // #TODO optimize.
                // #TODO evaluate the list_exprs

                for expr in exprs {
                    items.push(expr);
                }

                Some(Expr::List(items))
            }
            TokenKind::LeftBrace => {
                // Syntactic sugar for a Dict.

                // #Insight
                // Don't optimize to `Expr::Dict` here, leave the parser expr
                // 'normalized as it is beneficial for some kinds of analysis.

                // #TODO add error checking!
                // #TODO optimize.

                let exprs = self.parse_many(TokenKind::RightBrace, start)?;

                let mut items = vec![Ann::with_range(Expr::symbol("Dict"), range)];

                for expr in exprs {
                    items.push(expr);
                }

                Some(Expr::List(items))
            }
            TokenKind::RightParen | TokenKind::RightBracket | TokenKind::RightBrace => {
                // #TODO custom error for this?
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
                let range = start..self.index;
                Ok(Some(self.attach_buffered_annotations(expr, range)))
            }
            _ => Ok(None),
        }
    }

    // #TODO rename to `parse_until`?
    pub fn parse_many(
        &mut self,
        delimiter: TokenKind,
        start: usize,
    ) -> Result<Vec<Ann<Expr>>, Break> {
        let mut exprs = Vec::new();

        loop {
            let Some(token) = self.next_token() else {
                let range = start..self.index;
                let mut error = Error::new(ErrorKind::UnterminatedList);
                error.push_note(
                    "List not terminated",
                    Some(range),
                );
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

    // #Insight
    // The parse function intentionally returns an 'unstructured' vector of
    // expressions instead of something like a do-block or a module. Downstream
    // functions can enforce some structure.dd

    // #Insight
    // The loop in the parser is also useful to skip over comments.

    /// Parses the input tokens into expressions.
    /// The parser tries to return as many errors as possible.
    pub fn parse(&mut self) -> Result<Vec<Ann<Expr>>, Vec<Error>> {
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
