use crate::expr::Expr;

// #todo remove excessive clones.

// #todo find a better, more general name for this stage.

// #insight prune does not err.

// #insight
// Prune strips unnecessary auxiliary exprs not needed for evaluation.

// #todo consider only allowing the sigils, and not quot/unquot -> no, we need them to maintain the list/tree abstraction, it has to be syntax-sugar!
// #todo actually we could skip the `unquot`, think about it.

// #insight no need to convert Symbol to KeySymbol, just converting List -> Array works.

fn quote_list(expr: Expr) -> Expr {
    let (expr, ann) = expr.extract();
    let Expr::List(terms) = expr else {
        // #todo should never happen? maybe panic here?
        panic!("not a list");
    };
    Expr::maybe_annotated(
        Expr::Array(terms.iter().map(|expr| quote_fn(expr.clone())).collect()),
        ann,
    )
}

// #todo maintain annotations, use extract instead of unpack!!

fn quote_fn(expr: Expr) -> Expr {
    match expr.unpack() {
        // #todo handle unquote!
        Expr::List(terms) => {
            if terms.is_empty() {
                quote_list(expr)
            } else {
                if let Some(sym) = terms[0].unpack().as_symbol() {
                    if sym == "unquot" {
                        debug_assert!(terms.len() == 2);
                        terms[1].clone()
                    } else {
                        quote_list(expr)
                    }
                } else {
                    quote_list(expr)
                }
            }
        }
        _ => expr,
    }
}

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
        // #todo quote: list->array, symbol->key
        // #todo resolve quoting+interpolation here? i.e. quasiquoting
        // #todo maybe even resolve string interpolation here?
        Expr::List(terms) => {
            if terms.is_empty() {
                Some(expr)
            } else {
                if let Some(sym) = terms[0].unpack().as_symbol() {
                    if sym == "quot" {
                        debug_assert!(terms.len() == 2);
                        // #insight unquoting can happen only within quote, so it's handled in quasiquote_fn
                        // Some(terms[1].clone().transform(&quote_fn))
                        Some(quote_fn(terms[1].clone()))
                    } else {
                        Some(expr)
                    }
                } else {
                    Some(expr)
                }
            }
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
