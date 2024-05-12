use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, scope::Scope};

use super::eval;

// #todo add unit test.
pub fn eval_for_each(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo this is a temp hack!
    let [seq, var, body] = args else {
        return Err(Error::invalid_arguments("malformed `for-each`", None));
    };

    let seq = eval(seq, context)?;

    let Some(arr) = seq.as_array() else {
        return Err(Error::invalid_arguments(
            "`for-each` requires a `Seq` as the first argument",
            seq.range(),
        ));
    };

    let Some(sym) = var.as_symbol() else {
        return Err(Error::invalid_arguments(
            "`for-each` requires a symbol as the second argument",
            var.range(),
        ));
    };

    let prev_scope = context.scope.clone();
    context.scope = Arc::new(Scope::new(prev_scope.clone()));

    for x in arr.iter() {
        // #todo array should have Ann<Expr> use Ann<Expr> everywhere, avoid the clones!
        // #todo replace the clone with custom expr::ref/copy?
        context.scope.insert(sym, x.clone());
        eval(body, context)?;
    }

    context.scope = prev_scope;

    // #todo intentionally don't return a value, reconsider this?
    Ok(Expr::None)
}
