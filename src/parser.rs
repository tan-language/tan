pub mod error;

use std::collections::HashMap;

use crate::{
    ann::Ann,
    expr::{format_value, Expr},
    lexer::{token::Token, Lexer},
    range::{Range, Ranged},
};

use self::error::ParseError;

// #TODO no need to keep iterator as state in parser!
// #TODO can the parser be just a function? -> yes, if we use a custom iterator to keep the parsing state.

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
        }
    }

    /// Wrap the `expr` with the buffered (prefix) annotations.
    /// The annotations are parsed into an Expr representation.
    fn attach_buffered_annotations(&mut self, expr: Expr) -> Result<Ann<Expr>, Ranged<ParseError>> {
        let Some(annotations) = self.buffered_annotations.take() else {
            return Ok(Ann::new(expr));
        };

        let mut ann_exprs = Vec::new();

        for Ranged(ann_str, ann_range) in annotations {
            let mut lexer = Lexer::new(&ann_str);

            let Ok(tokens) = lexer.lex() else {
                return Err(Ranged(ParseError::MalformedAnnotation(ann_str), ann_range));
            };

            let mut parser = Parser::new(tokens);

            let Ann(ann_expr, ..) = parser.parse()?;

            ann_exprs.push(ann_expr);
        }

        Ok(Ann(expr, Some(ann_exprs)))
    }

    pub fn parse_expr(&mut self, token: Ranged<Token>) -> Result<Option<Expr>, Ranged<ParseError>> {
        let Ranged(t, range) = token;

        let expr = match t {
            Token::Comment(..) => None,
            Token::String(s) => Some(Expr::String(s)),
            Token::Symbol(s) => {
                if s.starts_with(':') {
                    let s = s.strip_prefix(':').unwrap();
                    Some(Expr::KeySymbol(s.to_string()))
                } else {
                    Some(Expr::Symbol(s))
                }
            }
            Token::Number(n) => Some(Expr::Int(n)),
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
                    return Err(Ranged(ParseError::InvalidQuote, range));
                };
                if token.0 == Token::Quote {
                    // Report consecutive quote (i.e. '') as error.
                    return Err(Ranged(ParseError::InvalidQuote, range));
                }
                let Some(target) = self.parse_expr(token)? else {
                    return Err(Ranged(ParseError::InvalidQuote, range));
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
                return Err(Ranged(ParseError::UnexpectedToken(t), range));
            }
        };

        Ok(expr)
    }

    // #TODO parse tokens here, to be consistent with parse_atom?
    pub fn parse_list(
        &mut self,
        delimiter: Token,
        list_range: Range,
    ) -> Result<Vec<Ann<Expr>>, Ranged<ParseError>> {
        // #TODO move range computation outside!

        let mut exprs = Vec::new();

        let mut index = list_range.start;

        loop {
            let token = self.tokens.next();

            let Some(token) = token  else {
                break;
            };

            index = token.1.end;

            if token.0 == delimiter {
                // #TODO set correct range
                return Ok(exprs);
            } else {
                // #TODO set correct range
                if let Some(e) = self.parse_expr(token)? {
                    let e = self.attach_buffered_annotations(e)?;
                    exprs.push(e);
                }
            }
        }

        // #TODO set correct range.
        let range = list_range.start..(index - 1);
        Err(Ranged(ParseError::UnterminatedList, range))
    }

    /// Tries to parse at least one expression.
    pub fn parse(&mut self) -> Result<Ann<Expr>, Ranged<ParseError>> {
        // #TODO can consolidate more with parse_atom

        loop {
            let token = self.tokens.next();

            let Some(token) = token  else {
                // #TODO what should we return on empty tokens list? Never? Error?
                return Ok(Ann::new(Expr::One));
            };

            let expr = self.parse_expr(token)?;

            if let Some(expr) = expr {
                return self.attach_buffered_annotations(expr);
            }
        }
    }
}
