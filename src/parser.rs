use crate::{
    ann::Ann,
    error::Error,
    expr::Expr,
    lexer::{token::Token, Lexer},
    range::{Range, Ranged},
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
    I: IntoIterator<Item = Ranged<Token>>,
{
    tokens: I::IntoIter,
    buffered_annotations: Option<Vec<Ranged<String>>>,
    index: usize,
    lookahead: Vec<Ranged<Token>>,
    errors: Vec<Ranged<Error>>,
}

impl<I> Parser<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
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
    fn next_token(&mut self) -> Option<Ranged<Token>> {
        if let Some(token) = self.lookahead.pop() {
            self.index = token.1.end;
            return Some(token);
        }

        if let Some(token) = self.tokens.next() {
            self.index = token.1.end;
            Some(token)
        } else {
            None
        }
    }

    fn put_back_token(&mut self, token: Ranged<Token>) {
        self.index = token.1.start;
        self.lookahead.push(token);
    }

    fn push_error(&mut self, error: Error, range: &Range) {
        self.errors.push(Ranged(error, range.clone()));
    }

    /// Wrap the `expr` with the buffered (prefix) annotations. The annotations
    /// are parsed into an Expr representation. Also attaches the range of the
    /// expression as an annotation.
    fn attach_annotations(&mut self, expr: Expr, range: Range) -> Ann<Expr> {
        // Annotate the expression with the range, by default.
        let mut expr = Ann::with_range(expr, range);

        let Some(annotations) = self.buffered_annotations.take() else {
            // No annotations for the expression.
            return expr;
        };

        for Ranged(ann_str, ann_range) in annotations {
            let mut lexer = Lexer::new(&ann_str);

            let Ok(tokens) = lexer.lex() else {
                self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                return expr;
            };

            let mut parser = Parser::new(tokens);

            let ann_expr = parser.parse();

            if let Err(ann_expr_errors) = ann_expr {
                for error in ann_expr_errors {
                    self.push_error(error.0, &error.1);
                }
                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                return expr;
            }

            // #TODO temp, support multiple expressions in annotation?
            let ann_expr = ann_expr.unwrap().swap_remove(0);

            let ann_expr = ann_expr.0;

            match &ann_expr {
                Expr::Symbol(sym) => {
                    if sym.is_empty() {
                        // #TODO specialized error needed.
                        self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
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
                        self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
                        // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                        return expr;
                    }
                }
                _ => {
                    self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
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

        let Ranged(t, range) = token;

        let start = range.start;

        let expr = match t {
            Token::Comment(s) => {
                // Preserve the comments as expressions, may be useful for analysis passes (e.g. formatting)
                // Comments are elided statically, before the evaluation pass.
                Some(Expr::Comment(s))
            }
            Token::MultiLineWhitespace => {
                // Preserve for formatter, will be elided statically, before the
                // evaluation pass.
                Some(Expr::TextSeparator)
            }
            // Token::Char(c) => Some(Expr::Char(c)),
            Token::String(s) => Some(Expr::String(s)),
            Token::Symbol(s) => {
                if s.starts_with(':') {
                    let s = s.strip_prefix(':').unwrap();
                    Some(Expr::KeySymbol(s.to_string()))
                } else if s == "true" {
                    // #TODO consider using (True) for true 'literal'.
                    // #TODO e.g. (let flag (True))
                    // #TODO Bool = True + False = True | False = ~False | False
                    Some(Expr::Bool(true))
                } else if s == "false" {
                    // #TODO consider using False for false 'literal'.
                    // #TODO consider having only true (and use something like nil for false)
                    // #TODO consider using nil for false and everything else for true
                    // #TODO consider using nothing/never for false and everything else for true.
                    Some(Expr::Bool(false))
                } else {
                    Some(Expr::Symbol(s))
                }
            }
            Token::Number(mut s) => {
                // #TODO more detailed Number error!
                // #TODO error handling not enough, we need to add context, check error_stack

                if s.contains('.') {
                    // #TODO support radix for non-integers?
                    // #TODO find a better name for 'non-integer'.
                    match s.parse::<f64>().map_err(Error::MalformedFloat) {
                        Ok(n) => Some(Expr::Float(n)),
                        Err(error) => {
                            self.push_error(error, &range);
                            None
                        }
                    }
                } else {
                    // #TODO support arbitrary radix https://github.com/golang/go/issues/28256
                    let mut radix = 10;

                    if s.starts_with("0x") {
                        s = s.replace("0x", "");
                        radix = 16
                    } else if s.starts_with("0b") {
                        s = s.replace("0b", "");
                        radix = 2
                    } else if s.starts_with("0o") {
                        // #Insight Octal literals are supported for historical reasons.
                        s = s.replace("0o", "");
                        radix = 8
                    }

                    match i64::from_str_radix(&s, radix).map_err(Error::MalformedInt) {
                        Ok(n) => Some(Expr::Int(n)),
                        Err(error) => {
                            self.push_error(error, &range);
                            None
                        }
                    }
                }
            }
            Token::Annotation(s) => {
                // #TODO support multiple annotations, e.g. `#[(min 1) (max 2)]`

                if self.buffered_annotations.is_none() {
                    self.buffered_annotations = Some(Vec::new());
                }

                self.buffered_annotations
                    .as_mut()
                    .unwrap()
                    .push(Ranged(s, range));

                None
            }
            Token::Quote => {
                // #Insight we should allow consecutive quotes, emit a linter warning instead!

                let Ok(quot_expr) = self.parse_expr() else {
                    // Parsing the quoted expression failed.
                    // Continue parsing to detect more errors.
                    self.push_error(Error::InvalidQuote, &range);
                    return Ok(None);
                };

                let Some(target) = quot_expr else {
                    self.push_error(Error::InvalidQuote, &range);
                    // It is recoverable error.
                    return Ok(None);
                };

                // #TODO the actual quoting should be handled here?
                // #TODO what about interpolation?

                Some(Expr::List(vec![Expr::symbol("quot").into(), target]))
            }
            Token::LeftParen => {
                let terms = self.parse_many(Token::RightParen, start)?;

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
            Token::LeftBracket => {
                // Syntactic sugar for a List/Array.

                // #Insight
                // Don't optimize to `Expr::Array` here, leave the parser expr
                // 'normalized as it is beneficial for some kinds of analysis.

                let exprs = self.parse_many(Token::RightBracket, start)?;

                let mut items = vec![Ann::with_range(Expr::symbol("Array"), range)];

                // #TODO add error checking!
                // #TODO optimize.
                // #TODO evaluate the list_exprs

                for expr in exprs {
                    items.push(expr);
                }

                Some(Expr::List(items))
            }
            Token::LeftBrace => {
                // Syntactic sugar for a Dict.

                // #Insight
                // Don't optimize to `Expr::Dict` here, leave the parser expr
                // 'normalized as it is beneficial for some kinds of analysis.

                // #TODO add error checking!
                // #TODO optimize.

                let exprs = self.parse_many(Token::RightBrace, start)?;

                let mut items = vec![Ann::with_range(Expr::symbol("Dict"), range)];

                for expr in exprs {
                    items.push(expr);
                }

                Some(Expr::List(items))
            }
            Token::RightParen | Token::RightBracket | Token::RightBrace => {
                // #TODO custom error for this?
                self.push_error(Error::UnexpectedToken(t), &range);
                // Parsing can continue.
                return Ok(None);
            }
        };

        match expr {
            Some(expr) => {
                let range = start..self.index;
                Ok(Some(self.attach_annotations(expr, range)))
            }
            _ => Ok(None),
        }
    }

    // #TODO rename to `parse_until`?
    pub fn parse_many(&mut self, delimiter: Token, start: usize) -> Result<Vec<Ann<Expr>>, Break> {
        let mut exprs = Vec::new();

        loop {
            let Some(token) = self.next_token() else {
                let range = start..self.index;
                self.push_error(Error::UnterminatedList, &range);
                return Err(Break {});
            };

            if token.0 == delimiter {
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
    pub fn parse(&mut self) -> Result<Vec<Ann<Expr>>, Vec<Ranged<Error>>> {
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
