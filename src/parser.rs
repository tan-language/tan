use crate::{
    lexer::token::Token,
    span::{Span, Spanned},
};

use self::{error::ParseError, expr::Expr};

pub mod error;
pub mod expr;

pub struct Parser<'a> {
    tokens: &'a [Spanned<Token>],
    index: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Spanned<Token>]) -> Self {
        Self { tokens, index: 0 }
    }

    // #TODO next_token()
    // #TODO put_back_token()
    // #TODO peek_token() (next+put_back)

    // #TODO returns AST
    pub fn parse(&mut self) -> Result<Expr, Spanned<ParseError>> {
        // #TODO don't use splitting, is there a reason to split?

        let (token, _rest) = self
            .tokens
            .split_first()
            .ok_or_else(|| Spanned::new(ParseError::NoToken, Span::default()))?;
        println!("--> {:?}", token);
        // println!("{:?}", self.tokens);

        // #TODO

        Ok(Expr::One)
    }

    pub fn parse_atom() {
        todo!()
    }

    pub fn parse_list() {
        todo!()
    }
}
