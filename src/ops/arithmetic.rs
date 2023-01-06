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
            return Err(EvalError::invalid_arguments(format!("`{}` is not an Int", arg)));
        };
        sum += n;
    }

    Ok(Expr::Int(sum))
}

pub fn sub(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(EvalError::invalid_arguments("`-` requires at least two arguments"));
    };

    let Expr::Int(a) = a else {
        return Err(EvalError::invalid_arguments(format!("`{}` is not an Int", a)));
    };

    let Expr::Int(b) = b else {
        return Err(EvalError::invalid_arguments(format!("`{}` is not an Int", b)));
    };

    Ok(Expr::Int(a - b))
}

pub fn mul(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    // #TODO optimize!
    let mut prod = 1;

    for arg in args {
        let Expr::Int(n) = arg else {
            return Err(EvalError::invalid_arguments(format!("`{}` is not an Int", arg)));
        };
        prod *= n;
    }

    Ok(Expr::Int(prod))
}
