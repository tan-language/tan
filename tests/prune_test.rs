mod common;

use tan::{api::parse_string_all, expr::Expr, prune::prune};

#[test]
fn parse_removes_comments() {
    let input = "; This is a comment\n(+ 1 2)";
    let exprs = parse_string_all(input).unwrap();

    let exprs: Vec<Expr> = exprs.into_iter().filter_map(prune).collect();

    assert_eq!(exprs.len(), 1);
}
