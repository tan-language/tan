use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, scope::Scope};

use super::eval;

// #todo #hack temp hack
// (let-ds [*q* 1]
//     (writeln q)
//     (writeln q)
// )

pub fn eval_let_ds(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    if args.len() < 2 {
        // #todo add more structural checks.
        // #todo proper error!
        return Err(Error::invalid_arguments("missing for arguments", None));
    }

    // #todo do should be 'monadic', propagate Eff (effect) wrapper.
    let mut value = Expr::Nil;

    let bindings = args.first().unwrap();
    let body = &args[1..];

    // #todo name this parent_scope?
    let prev_scope = context.dynamic_scope.clone();
    context.dynamic_scope = Arc::new(Scope::new(prev_scope.clone()));

    let Some(bindings) = bindings.as_array() else {
        return Err(Error::invalid_arguments(
            "malformed let-ds bindings",
            bindings.range(),
        ));
    };

    let bindings = bindings.clone();
    let mut bindings = bindings.iter();

    loop {
        let Some(name) = bindings.next() else {
            break;
        };

        let Some(value) = bindings.next() else {
            // #todo error?
            break;
        };

        let Some(s) = name.as_symbol() else {
            return Err(Error::invalid_arguments(
                &format!("`{name}` is not a Symbol"),
                name.range(),
            ));
        };

        // #todo add a check for *..* name, especially in debug profile.

        // no *..* reserved_symbols
        // // #todo do we really want this? Maybe convert to a lint?
        // if is_reserved_symbol(s) {
        //     return Err(Error::invalid_arguments(
        //         &format!("let cannot shadow the reserved symbol `{s}`"),
        //         name.range(),
        //     ));
        // }

        let value = eval(value, context)?;

        // #todo notify about overrides? use `set`?
        context.dynamic_scope.insert(s, value);
    }

    for expr in body {
        value = eval(expr, context)?;
    }

    context.dynamic_scope = prev_scope;

    // #todo return last value!
    Ok(value)
}
