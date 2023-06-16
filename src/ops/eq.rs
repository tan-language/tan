use crate::{ann::Ann, error::Error, eval::env::Env, expr::Expr, range::Range};

// #TODO support all types!

pub fn eq(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #TODO support overloading,
    // #TODO make equality a method of Expr?
    // #TODO support non-Int types
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("`eq` requires at least two arguments", Range::default())); // #TODO set in caller.
    };

    let Ann(Expr::Int(a), ..) = a else {
        return Err(Error::invalid_arguments(&format!("`{a}` is not an Int"), a.get_range()));
    };

    let Ann(Expr::Int(b), ..) = b else {
        return Err(Error::invalid_arguments(&format!("`{b}` is not an Int"), b.get_range()));
    };

    Ok(Expr::Bool(a == b).into())
}

pub fn gt(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("`>` requires at least two arguments", Range::default())); // #TODO set in caller.
    };

    let Ann(Expr::Int(a), ..) = a else {
        return Err(Error::invalid_arguments(&format!("`{a}` is not an Int"), a.get_range()));
    };

    let Ann(Expr::Int(b), ..) = b else {
        return Err(Error::invalid_arguments(&format!("`{b}` is not an Int"), b.get_range()));
    };

    Ok(Expr::Bool(a > b).into())
}

pub fn lt(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("`<` requires at least two arguments", Range::default())); // #TODO set in caller.
    };

    let Ann(Expr::Int(a), ..) = a else {
        return Err(Error::invalid_arguments(&format!("`{a}` is not an Int"), a.get_range()));
    };

    let Ann(Expr::Int(b), ..) = b else {
        return Err(Error::invalid_arguments(&format!("`{b}` is not an Int"), b.get_range()));
    };

    Ok(Expr::Bool(a < b).into())
}
