//! Common testing-support functions and utilities.

use tan::{
    ann::Ann,
    eval::{env::Env, error::EvalError, eval, prelude::setup_prelude},
    expr::Expr,
    lexer::{token::Token, Lexer},
    parser::Parser,
    range::Ranged,
};

pub fn read_file(filename: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
}

#[allow(dead_code)]
pub fn lex_file(filename: &str) -> Vec<Ranged<Token>> {
    let input = &read_file(filename);
    lex_string(input)
}

#[allow(dead_code)]
pub fn parse_file(filename: &str) -> Ann<Expr> {
    let input = &read_file(filename);
    parse_string(input)
}

#[allow(dead_code)]
pub fn eval_file(filename: &str) -> Result<Expr, EvalError> {
    let input = &read_file(filename);
    eval_string(input)
}

pub fn lex_string(input: &str) -> Vec<Ranged<Token>> {
    let mut lexer = Lexer::new(input);
    lexer.lex().unwrap()
}

pub fn parse_string(input: &str) -> Ann<Expr> {
    let tokens = lex_string(input);
    let mut parser = Parser::new(tokens);
    parser.parse().unwrap()
}

pub fn eval_string(input: &str) -> Result<Expr, EvalError> {
    let expr = parse_string(input);
    let mut env = setup_prelude(Env::default());
    eval(&expr, &mut env)
}
