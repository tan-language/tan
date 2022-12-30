use crate::{
    eval::{env::Env, error::EvalError},
    expr::Expr,
};

pub fn eq(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #TODO support overloading,
    // #TODO make equality a method of Expr?
    // #TODO support non-Int types
    // #TODO support multiple arguments.
    let [a, b] = args else {
        // #TODO proper error!
        return Err(EvalError::UnknownError);
    };

    let Expr::Int(a) = a else {
        // #TODO proper error!
        return Err(EvalError::UnknownError);
    };

    let Expr::Int(b) = b else {
        // #TODO proper error!
        return Err(EvalError::UnknownError);
    };

    Ok(Expr::Bool(a == b))
}

pub fn gt(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    // #TODO support multiple arguments.
    let [a, b] = &args[..] else {
        // #TODO proper error!
        return Err(EvalError::UnknownError);
    };

    let Expr::Int(a) = a else {
        // #TODO proper error!
        return Err(EvalError::UnknownError);
    };

    let Expr::Int(b) = b else {
        // #TODO proper error!
        return Err(EvalError::UnknownError);
    };

    Ok(Expr::Bool(a > b))
}

pub fn lt(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    // #TODO support multiple arguments.
    let [a, b] = &args[..] else {
        // #TODO proper error!
        return Err(EvalError::UnknownError);
    };

    let Expr::Int(a) = a else {
        // #TODO proper error!
        return Err(EvalError::UnknownError);
    };

    let Expr::Int(b) = b else {
        // #TODO proper error!
        return Err(EvalError::UnknownError);
    };

    Ok(Expr::Bool(a < b))
}
