use crate::expr::Expr;

// #todo find a better, more general name for this stage.

// #insight prune does not err.

// #insight
// Prune strips unnecessary auxiliary exprs not needed for evaluation.

pub fn prune_fn(expr: Expr) -> Option<Expr> {
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
        // Expr::List(terms) => {
        //     if let Some(Expr::Symbol(sym)) = terms.first() {
        //         if sym == "quot" {
        //             println!("--- QUOTE ---");
        //             Some(terms[1].clone())
        //         } else {
        //             Some(expr)
        //         }
        //     } else {
        //         Some(expr)
        //     }
        // }
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
