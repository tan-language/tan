use std::{collections::HashMap, fmt};

use crate::{
    ann::Ann,
    error::Error,
    expr::{format_value, Expr},
    lexer::{token::Token, Lexer},
    range::{Range, Ranged},
};

// #TODO no need to keep iterator as state in parser!
// #TODO can the parser be just a function? -> yes, if we use a custom iterator to keep the parsing state.
// #TODO think some more how annotations should be handled.

// #Insight
// The syntax of the language is explicitly designed to _not_ require a lookahead buffer.

// #Insight
// We move the tokens into the parser to simplify the code. The tokens are useless outside the parser.

/// The`NonRecoverableError` is thrown when the parser cannot synchronize
/// to continue parsing to detect more errors. Parsing is stopped immediately.
#[derive(Debug)]
pub struct NonRecoverableError {}

impl std::error::Error for NonRecoverableError {}

impl fmt::Display for NonRecoverableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NRE")
    }
}

/// The Parser performs the syntactic analysis stage of the compilation pipeline.
/// The input token stream is reduced into and Abstract Syntax Tree (AST).
/// The nodes of the AST are associated with annotations.
pub struct Parser<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
{
    tokens: I::IntoIter,
    buffered_annotations: Option<Vec<Ranged<String>>>,
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
            errors: Vec::new(),
        }
    }

    fn push_error(&mut self, error: Error, range: &Range) {
        self.errors.push(Ranged(error, range.clone()));
    }

    /// Wrap the `expr` with the buffered (prefix) annotations.
    /// The annotations are parsed into an Expr representation.
    fn attach_buffered_annotations(&mut self, expr: Expr) -> Ann<Expr> {
        let Some(annotations) = self.buffered_annotations.take() else {
            // No annotations for the expression.
            return Ann::new(expr);
        };

        let mut ann: HashMap<String, Expr> = HashMap::new();

        for Ranged(ann_str, ann_range) in annotations {
            let mut lexer = Lexer::new(&ann_str);

            let Ok(tokens) = lexer.lex() else {
                self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                return Ann::new(expr);
            };

            let mut parser = Parser::new(tokens);

            let ann_expr = parser.parse();

            if let Err(ann_expr_errors) = ann_expr {
                for error in ann_expr_errors {
                    self.push_error(error.0, &error.1);
                }
                // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                return Ann::new(expr);
            }

            let ann_expr = ann_expr.unwrap().0;

            match &ann_expr {
                Expr::Symbol(sym) => {
                    if sym.is_empty() {
                        // #TODO specialized error needed.
                        self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
                        // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                        return Ann::new(expr);
                    }

                    if sym.chars().next().unwrap().is_uppercase() {
                        // Type shorthand
                        ann.insert("type".to_owned(), ann_expr);
                    } else {
                        // Bool=true shorthand
                        ann.insert(sym.clone(), Expr::Bool(true));
                    }
                }
                Expr::List(list) => {
                    if let Some(Ann(Expr::Symbol(sym), _)) = list.first() {
                        ann.insert(sym.clone(), ann_expr);
                    } else {
                        self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
                        // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                        return Ann::new(expr);
                    }
                }
                _ => {
                    self.push_error(Error::MalformedAnnotation(ann_str), &ann_range);
                    // Ignore the buffered annotations, and continue parsing to find more syntactic errors.
                    return Ann::new(expr);
                }
            }
        }

        Ann(expr, Some(ann))
    }

    pub fn parse_expr(
        &mut self,
        token: Ranged<Token>,
    ) -> Result<Option<Expr>, NonRecoverableError> {
        let Ranged(t, range) = token;

        let expr = match t {
            Token::Comment(..) => None,
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
                    Some(Expr::Bool(false))
                } else {
                    Some(Expr::Symbol(s))
                }
            }
            Token::Int(n) => Some(Expr::Int(n)),
            Token::Float(n) => Some(Expr::Float(n)),
            Token::Annotation(s) => {
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
                let Some(token) = self.tokens.next() else {
                    // #TODO specialized error-message needed.
                    // EOF reached, cannot continue parsing.
                    self.push_error(Error::InvalidQuote, &range);
                    return Err(NonRecoverableError {});
                };

                // #Insight we should allow consecutive quotes, emit a linter warning instead!

                // if token.0 == Token::Quote {
                //     // #TODO specialized error-message needed.
                //     // Report consecutive quote (i.e. '') as error
                //     self.push_error(Error::InvalidQuote, &range);
                //     // Parsing can continue.
                //     return Ok(None);
                // }

                let Ok(quot_expr) = self.parse_expr(token) else {
                    // Parsing the quoted expression failed.
                    // Continue parsing to detect more errors.
                    return Ok(None);
                };

                let Some(target) = quot_expr else {
                    self.push_error(Error::InvalidQuote, &range);
                    return Ok(None);
                };
                Some(Expr::List(vec![
                    Ann::new(Expr::symbol("quot")),
                    Ann::new(target),
                ]))
            }
            Token::LeftParen => {
                let list_exprs = self.parse_list(Token::RightParen, range)?;

                if list_exprs.is_empty() {
                    // `()` == One/Unit/Top
                    Some(Expr::One)
                } else {
                    let head = list_exprs[0].clone();
                    match head {
                        // #TODO optimize more special forms.

                        // #Insight
                        // Parsing built-ins as Exprs optimizes runtime evaluation, with more efficient
                        // matching.

                        // `if` expression
                        Ann(Expr::Symbol(s), ..) if s == "if" => {
                            // #TODO detailed checking and error-reporting
                            Some(Expr::If(
                                Box::new(list_exprs[1].clone()),
                                Box::new(list_exprs[2].clone()),
                                // #TODO optional parsing!
                                Some(Box::new(list_exprs[3].clone())),
                            ))
                        }
                        _ => Some(Expr::List(list_exprs)),
                    }
                }
            }
            Token::LeftBracket => {
                // Syntactic sugar for a List/Array.

                let args = self.parse_list(Token::RightBracket, range)?;

                let mut items = Vec::new();

                // #TODO also parse (Array ..)
                // #TODO add error checking!
                // #TODO optimize.
                // #TODO evaluate the list_exprs
                // #TODO list

                for x in args {
                    items.push(x.0);
                }

                Some(Expr::Array(items))
            }
            Token::LeftBrace => {
                // Syntactic sugar for a Dict.

                let args = self.parse_list(Token::RightBrace, range)?;

                let mut dict = HashMap::new();

                // #TODO also parse (Dict ..)
                // #TODO add error checking!
                // #TODO optimize.
                // #TODO use Hashable.
                // #TODO evaluate the list_exprs
                // #TODO list

                for pair in args.chunks(2) {
                    let k = pair[0].clone();
                    let v = pair[1].clone();
                    dict.insert(format_value(k.0), v.0);
                }

                Some(Expr::Dict(dict))
            }
            Token::RightParen | Token::RightBracket | Token::RightBrace => {
                // #TODO custom error for this?
                self.push_error(Error::UnexpectedToken(t), &range);
                // Parsing can continue.
                return Ok(None);
            }
        };

        Ok(expr)
    }

    // #TODO parse tokens here, to be consistent with parse_atom?
    pub fn parse_list(
        &mut self,
        delimiter: Token,
        list_range: Range,
    ) -> Result<Vec<Ann<Expr>>, NonRecoverableError> {
        // #TODO move range computation outside!

        let mut exprs = Vec::new();

        let mut index = list_range.start;

        loop {
            let token = self.tokens.next();

            let Some(token) = token  else {
                // #TODO set correct range.
                let range = list_range.start..(index - 1);
                self.push_error(Error::UnterminatedList, &range);
                return Err(NonRecoverableError {});
            };

            index = token.1.end;

            if token.0 == delimiter {
                // #TODO set correct range
                return Ok(exprs);
            } else {
                // #TODO set correct range
                if let Some(e) = self.parse_expr(token)? {
                    let e = self.attach_buffered_annotations(e);
                    exprs.push(e);
                }
            }
        }
    }

    // #TODO try to parse all available expressions, return a namespace?
    // #TODO the parser should return a Vec<Ann<Expr>>

    /// Tries to parse at least one expression.
    /// The parser tries to return as many errors as possible.
    pub fn parse(&mut self) -> Result<Ann<Expr>, Vec<Ranged<Error>>> {
        // #TODO can consolidate more with parse_expr

        // #Insight
        // The loop is currently used to skip over comments.

        loop {
            let Some(token) = self.tokens.next() else {
                break;
            };

            let expr = self.parse_expr(token);

            let Ok(expr) = expr else {
                // A non-recoverable parse error was detected, stop parsing.
                let errors = std::mem::take(&mut self.errors);
                return Err(errors);
            };

            if let Some(expr) = expr {
                let expr = self.attach_buffered_annotations(expr);

                if self.errors.is_empty() {
                    // #TODO
                    return Ok(expr);
                } else {
                    let errors = std::mem::take(&mut self.errors);
                    return Err(errors);
                }
            }
        }

        if self.errors.is_empty() {
            // #TODO what should we return on empty tokens list? Never? Error?
            Ok(Ann::new(Expr::One))
        } else {
            let errors = std::mem::take(&mut self.errors);
            Err(errors)
        }
    }
}
