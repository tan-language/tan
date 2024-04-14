use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, scope::Scope};

use super::eval;

pub fn eval_do(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo do should be 'monadic', propagate Eff (effect) wrapper.
    let mut value = Expr::Nil;

    // #todo extract this.

    let prev_scope = context.scope.clone();
    context.scope = Arc::new(Scope::new(prev_scope.clone()));

    for expr in args {
        value = eval(expr, context)?;
    }

    context.scope = prev_scope;

    Ok(value)
}
