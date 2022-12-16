use crate::{lexer::Token, spanned::Spanned};

// #Insight
// AST = Expr = Value = Object

#[derive(Debug)]
pub enum Value {
    List,
}

pub struct Parser<'a> {
    tokens: &'a [Spanned<Token>],
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Spanned<Token>]) -> Self {
        Self { tokens }
    }

    // #TODO returns AST
    pub fn parse(&mut self) -> Value {
        // let (token, rest) = self.tokens.split_first()?;
        // .ok_or_else(|| Error::parse("could not get token", &Location::default(), 1))?;
        // println!("{:?}", token, tokens);
        println!("{:?}", self.tokens);
        Value::List
    }
}
