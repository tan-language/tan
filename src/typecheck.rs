use crate::{
    ann::Ann,
    eval::{env::Env, error::EvalError},
    expr::Expr,
};

// #TODO consider renaming to `type_eval`.
pub fn resolve_type(expr: &Ann<Expr>, env: &mut Env) -> Result<Ann<Expr>, EvalError> {
    // #TODO update the original annotations!
    match expr {
        Ann(Expr::Int(n), _) => {
            // #TODO check if it already has the annotation!
            Ok(Ann(Expr::Int(*n), Some(vec![Expr::symbol("Int")])))
        }
        Ann(Expr::Symbol(sym), _) => {
            // #TODO handle 'PathSymbol'

            let result = env.get(sym);

            let Some(expr) = result else {
                return Err(EvalError::UndefinedSymbol(sym.clone()));
            };

            let expr = resolve_type(&expr.clone(), env)?;
            Ok(Ann(
                Expr::Symbol(sym.clone()),
                Some(vec![expr.get_type().unwrap()]),
            ))
        }
        // Ann(Expr::List(list), ann) => {
        //     if list.is_empty() {
        //         // This is handled statically, in the parser, but an extra, dynamic
        //         // check is needed in resolve to handle the case where the
        //         // expression is constructed programmatically (e.g. self-modifying code,
        //         // dynamically constructed expression, homoiconicity, etc).
        //         return Ok(expr.clone());
        //     }

        //     // The unwrap here is safe.
        //     let head = list.first().unwrap();
        //     let tail = &list[1..];

        //     // #TODO also perform error checking here, e.g. if the head is invocable.
        //     // #TODO Expr.is_invocable, Expr.get_invocable_name, Expr.get_type
        //     // #TODO handle non-symbol cases!
        //     // #TODO signature should be the type, e.g. +::(Func Int Int Int) instead of +$$Int$$Int
        //     if let Ann(Expr::Symbol(sym), ann_sym) = head {
        //         if sym == "let" {
        //             // #TODO also report some of these errors statically, maybe in a sema phase?
        //             let mut args = tail.iter();

        //             loop {
        //                 let Some(sym) = args.next() else {
        //                     break;
        //                 };

        //                 let Some(value) = args.next() else {
        //                     // #TODO error?
        //                     break;
        //                 };

        //                 let Ann(Expr::Symbol(s), ..) = sym else {
        //                     return Err(EvalError::invalid_arguments(format!("`{}` is not a Symbol", sym)));
        //                 };

        //                 if is_reserved_symbol(s) {
        //                     return Err(EvalError::invalid_arguments(format!(
        //                         "let cannot shadow the reserved symbol `{s}`"
        //                     )));
        //                 }

        //                 let value = resolve_type(value, env)?;

        //                 // #TODO notify about overrides? use `set`?
        //                 env.insert(s, value);
        //             }

        //             // #TODO return last value!
        //             // Ok(Expr::One);
        //             todo!()
        //         }

        //         let mut signature = Vec::new();

        //         for term in tail {
        //             // #TODO can potentially consult the ann?
        //             signature.push(term.0.to_type_string())
        //         }

        //         let signature = signature.join("$$");

        //         let sym = format!("{sym}$${signature}");
        //         let mut list = vec![Ann(Expr::Symbol(sym), ann_sym.clone())];
        //         list.extend(tail.iter().cloned());

        //         return Ok(Ann(Expr::List(list.clone()), ann.clone()));
        //     }

        //     Ok(Ann(Expr::List(list.clone()), ann.clone()))
        // }
        _ => Ok(expr.clone()),
    }
}

#[cfg(test)]
mod tests {
    use crate::{api::parse_string, eval::env::Env, typecheck::resolve_type};

    #[test]
    fn resolve_specializes_functions() {
        let expr = parse_string("(do (let a 1) (+ a 2))").unwrap();
        dbg!(&expr);
        let mut env = Env::prelude();
        let expr = resolve_type(&expr, &mut env).unwrap();
        dbg!(&expr);
    }
}
