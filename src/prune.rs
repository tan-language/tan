use crate::{expr::Expr, parser::util::recognize_string_template};

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
    let (unpacked_expr, annotations) = expr.extract();

    match unpacked_expr {
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
        Expr::String(str) => {
            // #insight
            // only apply the transformation, error checking happened in the
            // parsing stage.
            if str.contains("${") {
                match recognize_string_template(str) {
                    Ok(format_expr) => {
                        // #todo extract a helper function.
                        if let Some(annotations) = annotations {
                            Some(Expr::Annotated(Box::new(format_expr), annotations.clone()))
                        } else {
                            Some(format_expr)
                        }
                    }
                    Err(_) => {
                        // #todo what should be done here?
                        // #insight this state should not be valid.
                        panic!("invalid state");
                    }
                }
            } else {
                Some(expr)
            }
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
    use assert_matches::assert_matches;

    use crate::{api::parse_string, expr::Expr, prune::prune};

    #[test]
    fn prune_removes_comments() {
        let input = "(do ; comment\n(let a [1 2 3 4]) ; a comment\n(writeln (+ 2 3)))";

        let expr = parse_string(input).unwrap();

        let expr = prune(expr).unwrap();

        let s = format!("--- {expr}");

        assert!(s.contains("(do (let a (Array 1 2 3 4)) (writeln (+ 2 3)))"));
    }

    #[test]
    fn prune_transforms_template_strings() {
        let input = r#"(let m "An amount: $110.00. Here is a number: ${num}, and another: ${another-num}")"#;
        let expr = parse_string(input).unwrap();

        let expr = prune(expr).unwrap();

        let Expr::List(exprs) = expr.unpack() else {
            panic!("assertion failed: invalid form")
        };

        // dbg!(exprs);

        let Expr::List(ref exprs) = exprs[2].unpack() else {
            panic!("assertion failed: invalid form")
        };

        assert_matches!(&exprs[0].unpack(), Expr::Symbol(s) if s == "format");
        assert_eq!(exprs.len(), 5);
    }
}
