use crate::{eval::env::Env, expr::Expr, error::Error};

// #Insight
// Named `arithmetic` as those operators can apply to non-numbers, e.g. Time, Date

// #TODO use AsRef, to avoid Annotated!
// #TODO use macros to generate specializations for generic versions.

pub fn add_int(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let mut sum = 0;

    for arg in args {
        let Expr::Int(n) = arg else {
            return Err(Error::invalid_arguments(format!("`{}` is not an Int", arg)));
        };
        sum += n;
    }

    Ok(Expr::Int(sum))
}

pub fn add_float(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let mut sum = 0.0;

    for arg in args {
        let Expr::Float(n) = arg else {
            return Err(Error::invalid_arguments(format!("`{}` is not a Float", arg)));
        };
        sum += n;
    }

    Ok(Expr::Float(sum))
}

pub fn sub(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("`-` requires at least two arguments"));
    };

    let Expr::Int(a) = a else {
        return Err(Error::invalid_arguments(format!("`{}` is not an Int", a)));
    };

    let Expr::Int(b) = b else {
        return Err(Error::invalid_arguments(format!("`{}` is not an Int", b)));
    };

    Ok(Expr::Int(a - b))
}

pub fn mul(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    // #TODO optimize!
    let mut prod = 1;

    for arg in args {
        let Expr::Int(n) = arg else {
            return Err(Error::invalid_arguments(format!("`{}` is not an Int", arg)));
        };
        prod *= n;
    }

    Ok(Expr::Int(prod))
}
