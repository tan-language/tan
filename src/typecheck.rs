use crate::{
    ann::Ann,
    eval::{env::Env, error::EvalError},
    expr::Expr,
    util::is_reserved_symbol,
};

// #TODO resolve_type and resolve_invocable should be combined, cannot be separate passes.

// #TODO consider renaming to `type_eval`.
pub fn resolve_type(mut expr: Ann<Expr>, env: &mut Env) -> Result<Ann<Expr>, EvalError> {
    // #TODO update the original annotations!
    match expr {
        Ann(Expr::Int(_), _) => {
            // #TODO check if it already has the annotation!
            expr.1 = Some(vec![Expr::symbol("Int")]);
            Ok(expr)
        }
        Ann(Expr::Float(_), _) => {
            // #TODO check if it already has the annotation!
            expr.1 = Some(vec![Expr::symbol("Float")]);
            Ok(expr)
        }
        Ann(Expr::String(_), _) => {
            // #TODO check if it already has the annotation!
            expr.1 = Some(vec![Expr::symbol("String")]);
            Ok(expr)
        }
        Ann(Expr::Symbol(ref sym), _) => {
            if is_reserved_symbol(sym) {
                expr.1 = Some(vec![Expr::symbol("Symbol")]);
                return Ok(expr);
            }

            // #TODO handle 'PathSymbol'

            let result = env.get(sym);

            let Some(value) = result else {
                return Err(EvalError::UndefinedSymbol(sym.clone()));
            };

            let value = resolve_type(value.clone(), env)?;
            expr.1 = Some(vec![value.type_annotation()]);
            Ok(expr)
        }
        Ann(Expr::List(ref list), _) => {
            if list.is_empty() {
                // This is handled statically, in the parser, but an extra, dynamic
                // check is needed in resolve to handle the case where the
                // expression is constructed programmatically (e.g. self-modifying code,
                // dynamically constructed expression, homoiconicity, etc).
                return Ok(expr);
            }

            // The unwrap here is safe.
            let head = list.first().unwrap();
            let tail = &list[1..];

            // #TODO also perform error checking here, e.g. if the head is invocable.
            // #TODO Expr.is_invocable, Expr.get_invocable_name, Expr.get_type
            // #TODO handle non-symbol cases!
            // #TODO signature should be the type, e.g. +::(Func Int Int Int) instead of +$$Int$$Int
            if let Ann(Expr::Symbol(ref sym), _) = head {
                if sym == "let" {
                    // #TODO also report some of these errors statically, maybe in a sema phase?
                    let mut args = tail.iter();

                    loop {
                        let Some(sym) = args.next() else {
                            break;
                        };

                        let Some(value) = args.next() else {
                            // #TODO error?
                            break;
                        };

                        let Ann(Expr::Symbol(s), ..) = sym else {
                            return Err(EvalError::invalid_arguments(format!("`{}` is not a Symbol", sym)));
                        };

                        if is_reserved_symbol(s) {
                            return Err(EvalError::invalid_arguments(format!(
                                "let cannot shadow the reserved symbol `{s}`"
                            )));
                        }

                        let value = resolve_type(value.clone(), env)?;
                        expr.1 = Some(vec![value.type_annotation()]);

                        // #TODO notify about overrides? use `set`?
                        env.insert(s, value);
                    }

                    Ok(expr)
                } else {
                    let mut resolved_tail = Vec::new();
                    for term in tail {
                        resolved_tail.push(resolve_type(term.clone(), env)?);
                    }

                    let head = if let Ann(Expr::Symbol(ref sym), ann_sym) = head {
                        let sym = if is_reserved_symbol(sym) {
                            sym.clone()
                        } else {
                            // #TODO should recursively resolve first!

                            let mut signature = Vec::new();

                            for term in &resolved_tail {
                                signature.push(term.to_type_string())
                            }

                            let signature = signature.join("$$");

                            format!("{sym}$${signature}")
                        };
                        Ann(Expr::Symbol(sym), ann_sym.clone())
                    } else {
                        head.clone()
                    };

                    // #Insight head should get resolved after the tail.
                    let head = resolve_type(head, env)?;

                    let mut list = vec![head.clone()];
                    list.extend(resolved_tail);

                    Ok(Ann(Expr::List(list), head.1))
                }
            } else {
                Ok(expr)
            }
        }
        _ => Ok(expr),
    }
}

#[cfg(test)]
mod tests {
    use crate::{api::parse_string, eval::env::Env, typecheck::resolve_type};

    #[test]
    fn resolve_specializes_functions() {
        // let expr = parse_string("(let a 1)").unwrap();
        // let expr = parse_string("(+ 1 2)").unwrap();
        // let expr = parse_string("(do (let a 1.3) (+ a 2.2))").unwrap();
        let expr = parse_string("(do (let a 1.3) (+ a (+ 1.0 2.2)))").unwrap();
        dbg!(&expr);
        let mut env = Env::prelude();
        let expr = resolve_type(expr, &mut env).unwrap();
        dbg!(&expr);
    }
}
