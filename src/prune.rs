use crate::expr::Expr;

// #todo remove excessive clones.

// #todo find a better, more general name for this stage.

// #insight prune does not err.

// #insight prune strips unnecessary auxiliary exprs not needed for evaluation.

// #todo strip quoting of literals (nops)
// #todo consider only allowing the sigils, and not quot/unquot -> no, we need them to maintain the list/tree abstraction, it has to be syntax-sugar!
// #todo actually we could skip the `unquot`, think about it.

// #insight no need to convert Symbol to KeySymbol, just converting List -> Array works.

pub fn prune_fn(expr: Expr) -> Option<Expr> {
    // #todo use `extract` instead of `unpack`.
    match expr.unpack() {
        Expr::Comment(..) => {
            // #todo move prune elsewhere.
            // Prune Comment expressions.
            None
        }
        Expr::TextSeparator => {
            // #todo remove TextSeparator anws.
            // #todo move prune elsewhere.
            // Prune TextSeparator expressions.
            None
        }
        // #todo resolve quoting+interpolation here? i.e. quasiquoting
        // #todo maybe even resolve string interpolation here?
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
