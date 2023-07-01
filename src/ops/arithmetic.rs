use crate::{error::Error, eval::env::Env, expr::Expr};

// #Insight
// Named `arithmetic` as those operators can apply to non-numbers, e.g. Time, Date

// #TODO use AsRef, to avoid Annotated!
// #TODO use macros to generate specializations for generic versions.
// #TODO deduct from type if the function can affect the env or have any other side-effects.

// #TODO ranges for arguments is too detailed, most probably we do not have the ranges!
// #TODO support invalid_arguments without range.

// #TODO autogen with a macro!
pub fn add_int(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let mut xs = Vec::new();

    for arg in args {
        let Some(n) = arg.as_int() else {
            // #TODO we could return the argument position here and enrich the error upstream.
            // #TODO hmm, the error is too precise here, do we really need the annotations?
            return Err(Error::invalid_arguments(&format!("{arg} is not an Int"), arg.range()));
        };
        xs.push(n);
    }

    let sum = add_int_impl(xs);

    Ok(Expr::Int(sum))
}

// #insight example of splitting wrapper from impl.
fn add_int_impl(xs: Vec<i64>) -> i64 {
    xs.iter().sum()
}

pub fn add_float(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let mut sum = 0.0;

    for arg in args {
        let Some(n) = arg.as_float() else {
            return Err(Error::invalid_arguments(&format!("{arg} is not a Float"), arg.range()));
        };
        sum += n;
    }

    Ok(Expr::Float(sum))
}

// #TODO should return the error without range and range should be added by caller.
pub fn sub(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("- requires at least two arguments", None));
    };

    let Some(a) = a.as_int() else {
        return Err(Error::invalid_arguments(&format!("{a} is not an Int"), a.range()));
    };

    let Some(b) = b.as_int() else {
        return Err(Error::invalid_arguments(&format!("{b} is not an Int"), b.range()));
    };

    Ok(Expr::Int(a - b))
}

pub fn mul(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    // #TODO optimize!
    let mut prod = 1;

    for arg in args {
        let Some(n) = arg.as_int() else {
            return Err(Error::invalid_arguments(&format!("{arg} is not an Int"), arg.range()));
        };
        prod *= n;
    }

    Ok(Expr::Int(prod))
}
