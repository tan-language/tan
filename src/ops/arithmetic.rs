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

// #TODO keep separate, optimized version with just 2 arguments!
// #TODO should support varargs.
// #TODO should return the error without range and range should be added by caller.
pub fn sub_int(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
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

pub fn sub_float(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("- requires at least two arguments", None));
    };

    let Some(a) = a.as_float() else {
        return Err(Error::invalid_arguments(&format!("{a} is not a Float"), a.range()));
    };

    let Some(b) = b.as_float() else {
        return Err(Error::invalid_arguments(&format!("{b} is not a Float"), b.range()));
    };

    Ok(Expr::Float(a - b))
}

pub fn mul_int(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    // #TODO optimize!
    let mut product = 1;

    for arg in args {
        let Some(n) = arg.as_int() else {
            return Err(Error::invalid_arguments(&format!("{arg} is not an Int"), arg.range()));
        };
        product *= n;
    }

    Ok(Expr::Int(product))
}

pub fn mul_float(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    // #TODO optimize!
    let mut product = 1.0;

    for arg in args {
        let Some(n) = arg.as_float() else {
            return Err(Error::invalid_arguments(&format!("{arg} is not a Float"), arg.range()));
        };
        product *= n;
    }

    Ok(Expr::Float(product))
}

// #TODO support int/float.
pub fn div_float(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    // #TODO optimize!
    let mut quotient = f64::NAN;

    // #TODO check for divide by zero! even statically check!
    // #TODO actually, divide by zero should return Infinity, not panic!!

    for arg in args {
        let Some(n) = arg.as_float() else {
            return Err(Error::invalid_arguments(&format!("{arg} is not a Float"), arg.range()));
        };

        if quotient.is_nan() {
            quotient = n;
        } else {
            quotient /= n;
        }
    }

    Ok(Expr::Float(quotient))
}

pub fn sin_float(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let Some(n) = args.first() else {
        return Err(Error::invalid_arguments("missing argument", None));
    };

    let Some(n) = n.as_float() else {
        return Err(Error::invalid_arguments("expected Float argument", n.range()));
    };

    Ok(Expr::Float(n.sin()))
}

pub fn cos_float(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let Some(n) = args.first() else {
        return Err(Error::invalid_arguments("missing argument", None));
    };

    let Some(n) = n.as_float() else {
        return Err(Error::invalid_arguments("expected Float argument", n.range()));
    };

    Ok(Expr::Float(n.cos()))
}

// #TODO support varargs?
pub fn powi_float(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let [n, e] = args else {
        return Err(Error::invalid_arguments("- requires at least two arguments", None));
    };

    // #TODO version of as_float that automatically throws an Error?
    let Some(n) = n.as_float() else {
        return Err(Error::invalid_arguments(&format!("{n} is not a Float"), n.range()));
    };

    let Some(e) = e.as_int() else {
        return Err(Error::invalid_arguments(&format!("{e} is not an Int"), e.range()));
    };

    Ok(Expr::Float(n.powi(e as i32)))
}
