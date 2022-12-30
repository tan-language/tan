use crate::{
    eval::{env::Env, error::EvalError},
    expr::Expr,
};

// #Insight
// Named `arithmetic` as those operators can apply to non-numbers, e.g. Time, Date

// #TODO use AsRef, to avoid Annotated!

pub fn add(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    let mut sum = 0;

    for arg in args {
        let Expr::Int(n) = arg else {
            return Err(EvalError::ArgumentError("invalid argument type, expecting `Int`".to_string()));
        };
        sum += n;
    }

    Ok(Expr::Int(sum))
}
