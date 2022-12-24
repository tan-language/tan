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
    active_annotations: Option<Vec<Ranged<String>>>,
}

impl<I> Parser<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
{
    pub fn new(tokens: I) -> Self {
        let tokens = tokens.into_iter();

        Self {
            tokens,
            active_annotations: None,
        }
    }

    /// Wrap the `expr` with the active (prefix) annotations.
    /// The annotations are parsed into an Expr representation.
    fn apply_annotations(&mut self, expr: Expr) -> Annotated<Expr> {
        let Some(annotations) = self.active_annotations.take() else {
            return Annotated::new(expr);
        };

        let mut ann_exprs = Vec::new();

        // #TODO really messy code, cleanup!

        for a in annotations {
            // #TODO don't use parse_string
            let ae = parse_string(&a.0);
            let Expr::Do(ae) = ae.0 else {
                continue;
            };
            let ae = ae.first().unwrap();
            ann_exprs.push(ae.0.clone());
        }

        Annotated(expr, Some(ann_exprs))
    }

    // #TODO better name!
    // #TODO what is list_range?
    pub fn parse_tokens(
        &mut self,
        list_range: Option<Range>,
    ) -> Result<Vec<Annotated<Expr>>, Ranged<ParseError>> {
        let mut exprs = Vec::new();

        let mut token: Option<Ranged<Token>>;

        loop {
            token = self.tokens.next();

            let Some(st) = token  else {
                if let Some(range) = list_range {
                    return Err(Ranged(
                        ParseError::UnterminatedList,
                        range,
                    ));
                }
                break;
            };

            let Ranged(t, range) = st;

            match t {
                Token::Comment(..) => (),
                Token::String(s) => {
                    let expr = self.apply_annotations(Expr::String(s));
                    exprs.push(expr);
                }
                Token::Symbol(s) => {
                    let expr = self.apply_annotations(Expr::Symbol(s));
                    exprs.push(expr);
                }
                Token::Number(n) => {
                    let expr = self.apply_annotations(Expr::Int(n));
                    exprs.push(expr);
                }
                Token::Annotation(s) => {
                    if self.active_annotations.is_none() {
                        self.active_annotations = Some(Vec::new());
                    }

                    self.active_annotations
                        .as_mut()
                        .unwrap()
                        .push(Ranged(s, range));
                }
                Token::LeftParen => {
                    let list_exprs = self.parse_tokens(Some(range))?;
                    let expr = self.apply_annotations(Expr::List(list_exprs));
                    exprs.push(expr);
                }
                Token::RightParen => {
                    if list_range.is_some() {
                        return Ok(exprs);
                    } else {
                        // #TODO custom error here?
                        return Err(Ranged(ParseError::UnexpectedToken(t), range));
                    }
                }
                _ => {
                    return Err(Ranged(ParseError::UnexpectedToken(t), range));
                }
            }
        }

        Ok(exprs)
    }

    // #TODO handle annotations
    pub fn parse(&mut self) -> Result<Annotated<Expr>, Ranged<ParseError>> {
        let exprs = self.parse_tokens(None)?;

        Ok(Annotated::new(Expr::Do(exprs)))
    }
}
