mod common;

use tan::{
    api::parse_string_all,
    expr::{format_value, Expr},
    prune::prune,
};

#[test]
fn parse_removes_comments() {
    let input = "; This is a comment\n(+ 1 2)";
    let exprs = parse_string_all(input).unwrap();

    let exprs: Vec<Expr> = exprs.into_iter().filter_map(|expr| prune(expr)).collect();

    assert_eq!(exprs.len(), 1);
}

#[test]
fn parse_transforms_quasiquotes() {
    let input = "'(hello world (cos (+ 1 0)) $(cos 1))"; // #todo extract as fixture file.
    let exprs = parse_string_all(input).unwrap();

    let exprs: Vec<Expr> = exprs.into_iter().filter_map(|expr| prune(expr)).collect();

    // #todo extract as a helper!
    println!("{}", format_value(exprs[0].clone()));

    // assert_eq!(exprs.len(), 1);
}
