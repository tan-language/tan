pub mod error;

use crate::{
    ann::Annotated,
    expr::Expr,
    lexer::token::Token,
    parse_string,
    range::{Range, Ranged},
};

use self::error::ParseError;

// #TODO no need to keep iterator as state in parser!
// #TODO can the parser be just a function? -> yes, if we use a custom iterator to keep the parsing state.

// #Insight
// The syntax of the language is explicitly designed to _not_ require a lookahead buffer.

// #Insight
// We move the tokens into the parser to simplify the code. The tokens are useless outside the parser.

/// The Parser performs the syntax analysis stage of the compilation pipeline.
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
    fn attach_buffered_annotations(&mut self, expr: Expr) -> Annotated<Expr> {
        let Some(annotations) = self.buffered_annotations.take() else {
            return Annotated::new(expr);
        };

        let mut ann_exprs = Vec::new();

        for Ranged(s, ..) in annotations {
            // #TODO don't use parse_string
            let Annotated(ae, ..) = parse_string(&s);
            ann_exprs.push(ae);
        }

        Annotated(expr, Some(ann_exprs))
    }

    pub fn parse_atom(&mut self, token: Ranged<Token>) -> Result<Option<Expr>, Ranged<ParseError>> {
        let Ranged(t, range) = token;

        let expr = match t {
            Token::Comment(..) => None,
            Token::String(s) => Some(Expr::String(s)),
            Token::Symbol(s) => match s.as_str() {
                "do" => Some(Expr::Do),
                _ => Some(Expr::Symbol(s)),
            },
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
            Token::LeftParen => {
                let list_exprs = self.parse_list(range)?;
                Some(Expr::List(list_exprs))
            }
            Token::RightParen => {
                return Err(Ranged(ParseError::UnexpectedToken(t), range));
            }
            _ => {
                return Err(Ranged(ParseError::UnexpectedToken(t), range));
            }
        };

        Ok(expr)
    }

    pub fn parse_list(
        &mut self,
        list_range: Range,
    ) -> Result<Vec<Annotated<Expr>>, Ranged<ParseError>> {
        let mut exprs = Vec::new();

        let mut token: Option<Ranged<Token>>;

        loop {
            token = self.tokens.next();

            let Some(token) = token  else {
                break;
            };

            match token.0 {
                Token::RightParen => {
                    return Ok(exprs);
                }
                _ => {
                    if let Some(e) = self.parse_atom(token)? {
                        let e = self.attach_buffered_annotations(e);
                        exprs.push(e);
                    }
                }
            };
        }

        Err(Ranged(ParseError::UnterminatedList, list_range))
    }

    /// Tries to parse at least one expression.
    pub fn parse(&mut self) -> Result<Annotated<Expr>, Ranged<ParseError>> {
        loop {
            let token = self.tokens.next();

            let Some(token) = token  else {
                // #TODO what should we return on empty tokens list? Never? Error?
                return Ok(Annotated::new(Expr::One));
            };

            let expr = match token.0 {
                Token::LeftParen => {
                    let list_exprs = self.parse_list(token.1)?;
                    Some(Expr::List(list_exprs))
                }
                _ => self.parse_atom(token)?,
            };

            if let Some(expr) = expr {
                return Ok(self.attach_buffered_annotations(expr));
            }
        }
    }
}
