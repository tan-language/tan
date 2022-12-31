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
            return Err(EvalError::InvalidArguments("invalid argument type, expecting `Int`".to_string()));
        };
        sum += n;
    }

    Ok(Expr::Int(sum))
}

pub fn sub(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        // #TODO proper error!
        return Err(EvalError::Unknown);
    };

    let Expr::Int(a) = a else {
        // #TODO proper error!
        return Err(EvalError::Unknown);
    };

    let Expr::Int(b) = b else {
        // #TODO proper error!
        return Err(EvalError::Unknown);
    };

    Ok(Expr::Int(a - b))
}

pub fn mul(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    // #TODO optimize!
    let mut prod = 1;

    for arg in args {
        let Expr::Int(n) = arg else {
            // #TODO proper error!
            return Err(EvalError::Unknown);
        };
        prod *= n;
    }

    Ok(Expr::Int(prod))
}
