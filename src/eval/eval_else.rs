use crate::{context::Context, error::Error, expr::Expr};

pub fn eval_else(_args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let err = Error::panic_with_context(
        "`eval` can only be used within a conditional block, e.g. `if`, `unless`, etc.",
        context,
    );
    Err(err)
}
