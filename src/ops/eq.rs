use crate::{ann::Ann, error::Error, eval::env::Env, expr::Expr, range::Ranged};

pub fn eq(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #TODO support overloading,
    // #TODO make equality a method of Expr?
    // #TODO support non-Int types
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

    Ok(Expr::Bool(a == b).into())
}

pub fn gt(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
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

    Ok(Expr::Bool(a > b).into())
}

pub fn lt(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
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

    Ok(Expr::Bool(a < b).into())
}
