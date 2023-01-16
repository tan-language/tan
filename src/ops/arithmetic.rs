use crate::{api::Result, error::Error, eval::env::Env, expr::Expr};

// #Insight
// Named `arithmetic` as those operators can apply to non-numbers, e.g. Time, Date

// #TODO use AsRef, to avoid Annotated!
// #TODO use macros to generate specializations for generic versions.

pub fn add_int(args: &[Expr], _env: &Env) -> Result<Expr> {
    let mut sum = 0;

    for arg in args {
        let Expr::Int(n) = arg else {
            return Err(Error::invalid_arguments(format!("`{}` is not an Int", arg)).into());
        };
        sum += n;
    }

    Ok(Expr::Int(sum))
}

pub fn add_float(args: &[Expr], _env: &Env) -> Result<Expr> {
    let mut sum = 0.0;

    for arg in args {
        let Expr::Float(n) = arg else {
            return Err(Error::invalid_arguments(format!("`{}` is not a Float", arg)).into());
        };
        sum += n;
    }

    Ok(Expr::Float(sum))
}

pub fn sub(args: &[Expr], _env: &Env) -> Result<Expr> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("`-` requires at least two arguments").into());
    };

    let Expr::Int(a) = a else {
        return Err(Error::invalid_arguments(format!("`{}` is not an Int", a)).into());
    };

    let Expr::Int(b) = b else {
        return Err(Error::invalid_arguments(format!("`{}` is not an Int", b)).into());
    };

    Ok(Expr::Int(a - b))
}

pub fn mul(args: &[Expr], _env: &Env) -> Result<Expr> {
    // #TODO optimize!
    let mut prod = 1;

    for arg in args {
        let Expr::Int(n) = arg else {
            return Err(Error::invalid_arguments(format!("`{}` is not an Int", arg)).into());
        };
        prod *= n;
    }

    Ok(Expr::Int(prod))
}
