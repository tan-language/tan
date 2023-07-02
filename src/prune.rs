use crate::expr::Expr;

// #insight prune does not err.

// #insight
// Prune strips unnecessary auxiliary exprs not needed for evaluation.

pub fn prune_fn(expr: Expr) -> Option<Expr> {
    match expr.unpack() {
        Expr::Comment(..) => {
            // #TODO move prune elsewhere.
            // Prune Comment expressions.
            None
        }
        Expr::TextSeparator => {
            // #TODO remove TextSeparator anws.
            // #TODO move prune elsewhere.
            // Prune TextSeparator expressions.
            None
        }
        _ => Some(expr),
    }
}

pub fn prune(expr: Expr) -> Option<Expr> {
    expr.filter_transform(&prune_fn)
}

#[cfg(test)]
mod tests {
    use crate::{api::parse_string, prune::prune};

    #[test]
    fn prune_removes_comments() {
        let input = "(do ; comment\n(let a [1 2 3 4]) ; a comment\n(writeln (+ 2 3)))";

        let expr = parse_string(input).unwrap();

        let expr = prune(expr).unwrap();

        let s = format!("--- {expr}");

        assert!(s.contains("(do (let a (Array 1 2 3 4)) (writeln (+ 2 3)))"));
    }
}
