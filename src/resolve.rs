use crate::{ann::Ann, eval::error::EvalError, expr::Expr, util::is_reserved_symbol};

// #TODO consider renaming to `resolver` or `typecheck` or `type_eval`.
// #TODO resolve-types pass
// #TODO resolve-invocables pass

// #TODO consider moving the arguments.
pub fn resolve(expr: &Ann<Expr>) -> Result<Ann<Expr>, EvalError> {
    // let expr = expr.as_ref();

    match expr {
        Ann(Expr::List(list), ann) => {
            if list.is_empty() {
                // This is handled statically, in the parser, but an extra, dynamic
                // check is needed in resolve to handle the case where the
                // expression is constructed programmatically (e.g. self-modifying code,
                // dynamically constructed expression, homoiconicity, etc).
                return Ok(expr.clone());
            }

            // The unwrap here is safe.
            let head = list.first().unwrap();
            let tail = &list[1..];

            // #TODO also perform error checking here, e.g. if the head is invocable.
            // #TODO Expr.is_invocable, Expr.get_invocable_name, Expr.get_type
            // #TODO handle non-symbol cases!
            // #TODO signature should be the type, e.g. +::(Func Int Int Int) instead of +$$Int$$Int
            if let Ann(Expr::Symbol(sym), ann_sym) = head {
                if is_reserved_symbol(sym) {
                    return Ok(expr.clone());
                }

                let mut signature = Vec::new();

                for term in tail {
                    // #TODO can potentially consult the ann?
                    signature.push(term.0.to_type_string())
                }

                let signature = signature.join("$$");

                let sym = format!("{sym}$${signature}");
                let mut list = vec![Ann(Expr::Symbol(sym), ann_sym.clone())];
                list.extend(tail.iter().cloned());

                return Ok(Ann(Expr::List(list.clone()), ann.clone()));
            }

            Ok(Ann(Expr::List(list.clone()), ann.clone()))
        }
        _ => Ok(expr.clone()),
    }
}

#[cfg(test)]
mod tests {
    use crate::{api::parse_string, resolve::resolve};

    #[test]
    fn resolve_specializes_functions() {
        let expr = parse_string("#test (+ 1 #zonk 2)").unwrap();
        dbg!(&expr);
        let expr = resolve(&expr).unwrap();
        dbg!(&expr);
    }
}
