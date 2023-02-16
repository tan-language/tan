mod common;

use tan::{
    ann::Ann, api::parse_string_all, eval::env::Env, expr::Expr, macro_expand::macro_expand,
};

#[test]
fn parse_removes_comments() {
    let input = "-- This is a comment\n(+ 1 2)";
    let exprs = parse_string_all(input).unwrap();

    let mut env = Env::prelude();
    let exprs: Vec<Ann<Expr>> = exprs
        .into_iter()
        .filter_map(|expr| macro_expand(expr, &mut env).unwrap())
        .collect();

    assert_eq!(exprs.len(), 1);
}
