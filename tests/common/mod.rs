//! Common testing-support functions and utilities.

use tan::{
    ann::Annotated, eval::error::EvalError, eval_string, expr::Expr, lex_string,
    lexer::token::Token, parse_string, range::Ranged,
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
pub fn parse_file(filename: &str) -> Annotated<Expr> {
    let input = &read_file(filename);
    parse_string(input)
}

#[allow(dead_code)]
pub fn eval_file(filename: &str) -> Result<Expr, EvalError> {
    let input = &read_file(filename);
    eval_string(input)
}
