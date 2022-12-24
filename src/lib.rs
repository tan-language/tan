pub mod ann;
pub mod eval;
pub mod expr;
pub mod lexer;
pub mod parser;
pub mod range;
pub mod util;

use ann::Annotated;
use eval::{env::Env, error::EvalError, eval};
use expr::Expr;
use lexer::{token::Token, Lexer};
use parser::Parser;
use range::Ranged;

pub fn lex_string(input: &str) -> Vec<Ranged<Token>> {
    let mut lexer = Lexer::new(input);
    // #TODO remove unwrap!
    lexer.lex().unwrap()
}

pub fn parse_string(input: &str) -> Annotated<Expr> {
    let tokens = lex_string(input);
    let mut parser = Parser::new(tokens);
    // #TODO remove unwrap!
    parser.parse().unwrap()
}

pub fn eval_string(input: &str) -> Result<Expr, EvalError> {
    let expr = parse_string(input);
    let mut env = Env::default();
    eval(&expr, &mut env)
}
