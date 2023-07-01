//! Common testing-support functions and utilities.

use tan::{
    api::{eval_string, parse_string, resolve_string},
    error::Error,
    eval::env::Env,
    expr::Expr,
    lexer::{token::Token, Lexer},
};

pub fn lex_string(input: &str) -> Result<Vec<Token>, Vec<Error>> {
    let mut lexer = Lexer::new(input);
    lexer.lex()
}

pub fn read_file(filename: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
}

#[allow(dead_code)]
pub fn lex_file(filename: &str) -> Result<Vec<Token>, Vec<Error>> {
    let input = &read_file(filename);
    lex_string(input)
}

#[allow(dead_code)]
pub fn parse_file(filename: &str) -> Result<Expr, Vec<Error>> {
    let input = &read_file(filename);
    parse_string(input)
}

#[allow(dead_code)]
pub fn resolve_file(filename: &str) -> Result<Vec<Expr>, Vec<Error>> {
    let input = &read_file(filename);
    let mut env = Env::prelude();
    resolve_string(input, &mut env)
}

#[allow(dead_code)]
pub fn eval_file(filename: &str) -> Result<Expr, Vec<Error>> {
    let input = &read_file(filename);
    let mut env = Env::prelude();
    eval_string(input, &mut env)
}
