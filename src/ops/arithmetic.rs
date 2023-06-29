use crate::{ann::ANNO, error::Error, eval::env::Env, expr::Expr};

// #Insight
// Named `arithmetic` as those operators can apply to non-numbers, e.g. Time, Date

// #TODO use AsRef, to avoid Annotated!
// #TODO use macros to generate specializations for generic versions.
// #TODO deduct from type if the function can affect the env or have any other side-effects.

// #TODO autogen with a macro!
pub fn add_int(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let mut xs = Vec::new();

    for arg in args {
        let ANNO(Expr::Int(n), _) = arg else {
            return Err(Error::invalid_arguments(&format!("{arg} is not an Int"), arg.get_range()));
        };
        xs.push(*n);
    }

    let sum = add_int_impl(xs);

    Ok(Expr::Int(sum).into())
}

fn add_int_impl(xs: Vec<i64>) -> i64 {
    xs.iter().sum()
}

pub fn add_float(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let mut sum = 0.0;

    for arg in args {
        let ANNO(Expr::Float(n), ..) = arg else {
            return Err(Error::invalid_arguments(&format!("{arg} is not a Float"), arg.get_range()));
        };
        sum += n;
    }

    Ok(Expr::Float(sum).into())
}

// #TODO should return the error without range and range should be added by caller.
pub fn sub(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("- requires at least two arguments", None));
    };

    let ANNO(Expr::Int(a), ..) = a else {
        return Err(Error::invalid_arguments(&format!("{a} is not an Int"), a.get_range()));
    };

    let ANNO(Expr::Int(b), ..) = b else {
        return Err(Error::invalid_arguments(&format!("{b} is not an Int"), b.get_range()));
    };

    Ok(Expr::Int(a - b).into())
}

pub fn mul(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    // #TODO optimize!
    let mut prod = 1;

    for arg in args {
        let ANNO(Expr::Int(n), ..) = arg else {
            return Err(Error::invalid_arguments(&format!("{arg} is not an Int"), arg.get_range()));
        };
        prod *= n;
    }

    Ok(Expr::Int(prod).into())
}
