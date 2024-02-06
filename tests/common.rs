//! Common testing-support functions and utilities.

#![allow(dead_code)]

// #todo consider moving these functions to api.rs

use tan::{
    api::{eval_string, lex_string, parse_string, resolve_string},
    context::Context,
    error::Error,
    expr::Expr,
    lexer::token::Token,
};

pub fn read_file(filename: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
}

pub fn lex_file(filename: &str) -> Result<Vec<Token>, Vec<Error>> {
    let input = &read_file(filename);
    lex_string(input)
}

pub fn parse_file(filename: &str) -> Result<Expr, Vec<Error>> {
    let input = &read_file(filename);
    parse_string(input)
}

pub fn resolve_file(filename: &str) -> Result<Vec<Expr>, Vec<Error>> {
    let input = &read_file(filename);
    let mut context = Context::new();
    resolve_string(input, &mut context)
}

pub fn eval_file(filename: &str) -> Result<Expr, Vec<Error>> {
    // #todo use eval_module here!!
    let input = &read_file(filename);
    let mut context = Context::new();
    eval_string(input, &mut context)
}

// #todo find a better name.
// #todo move this function to api.rs?
/// Evaluates an input string. A thin wrapper around eval_string.
pub fn eval_input(input: &str) -> Result<Expr, Vec<Error>> {
    let mut context = Context::new();
    eval_string(input, &mut context)
}
