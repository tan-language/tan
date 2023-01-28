use crate::{ann::Ann, error::Error, eval::env::Env, expr::Expr, range::Ranged};

// #Insight
// Named `arithmetic` as those operators can apply to non-numbers, e.g. Time, Date

// #TODO use AsRef, to avoid Annotated!
// #TODO use macros to generate specializations for generic versions.

pub fn add_int(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
    let mut sum = 0;

    for arg in args {
        let Ann(Expr::Int(n), ..) = arg else {
            return Err(Error::invalid_arguments(format!("`{}` is not an Int", arg)).into());
        };
        sum += n;
    }

    Ok(Expr::Int(sum).into())
}

pub fn add_float(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
    let mut sum = 0.0;

    for arg in args {
        let Ann(Expr::Float(n), ..) = arg else {
            return Err(Error::invalid_arguments(format!("`{}` is not a Float", arg)).into());
        };
        sum += n;
    }

    Ok(Expr::Float(sum).into())
}

pub fn sub(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("`-` requires at least two arguments").into());
    };

    let Ann(Expr::Int(a), ..) = a else {
        return Err(Error::invalid_arguments(format!("`{}` is not an Int", a)).into());
    };

    let Ann(Expr::Int(b), ..) = b else {
        return Err(Error::invalid_arguments(format!("`{}` is not an Int", b)).into());
    };

    Ok(Expr::Int(a - b).into())
}

pub fn mul(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
    // #TODO optimize!
    let mut prod = 1;

    for arg in args {
        let Ann(Expr::Int(n), ..) = arg else {
            return Err(Error::invalid_arguments(format!("`{}` is not an Int", arg)).into());
        };
        prod *= n;
    }

    Ok(Expr::Int(prod).into())
}
