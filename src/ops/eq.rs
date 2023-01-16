use crate::{api::Result, error::Error, eval::env::Env, expr::Expr};

pub fn eq(args: &[Expr], _env: &Env) -> Result<Expr> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #TODO support overloading,
    // #TODO make equality a method of Expr?
    // #TODO support non-Int types
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

    Ok(Expr::Bool(a == b))
}

pub fn gt(args: &[Expr], _env: &Env) -> Result<Expr> {
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

    Ok(Expr::Bool(a > b))
}

pub fn lt(args: &[Expr], _env: &Env) -> Result<Expr> {
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

    Ok(Expr::Bool(a < b))
}
