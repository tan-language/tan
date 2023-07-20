use crate::{context::Context, error::Error, expr::Expr};

// #TODO support all types!

pub fn eq(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #TODO support overloading,
    // #TODO make equality a method of Expr?
    // #TODO support non-Int types
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("`eq` requires at least two arguments", None));
    };

    let Some(a) = a.as_int() else {
        return Err(Error::invalid_arguments(&format!("`{a}` is not an Int"), a.range()));
    };

    let Some(b) = b.as_int() else {
        return Err(Error::invalid_arguments(&format!("`{b}` is not an Int"), b.range()));
    };

    Ok(Expr::Bool(a == b))
}

pub fn gt(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("`>` requires at least two arguments", None));
    };

    let Some(a) = a.as_int() else {
        return Err(Error::invalid_arguments(&format!("`{a}` is not an Int"), a.range()));
    };

    let Some(b) = b.as_int() else {
        return Err(Error::invalid_arguments(&format!("`{b}` is not an Int"), b.range()));
    };

    Ok(Expr::Bool(a > b))
}

pub fn lt(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    // #TODO support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments("`<` requires at least two arguments", None));
    };

    let Some(a) = a.as_int() else {
        return Err(Error::invalid_arguments(&format!("`{a}` is not an Int"), a.range()));
    };

    let Some(b) = b.as_int() else {
        return Err(Error::invalid_arguments(&format!("`{b}` is not an Int"), b.range()));
    };

    Ok(Expr::Bool(a < b))
}
