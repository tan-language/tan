//! Math, Arithmetic, Numerical operators.

use crate::{
    eval::{env::Env, error::EvalError},
    expr::Expr,
};

// #TODO use AsRef, to avoid Annotated!

pub fn add(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    let mut sum = 0;

    for arg in args {
        let Expr::Int(n) = arg else {
            // #TODO proper error!
            return Err(EvalError::UnknownError);
        };
        sum += n;
    }

    Ok(Expr::Int(sum))
}
