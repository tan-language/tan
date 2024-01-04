use crate::{context::Context, error::Error, expr::Expr};

// #todo support all types!

pub fn eq(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #todo support overloading,
    // #todo make equality a method of Expr?
    // #todo support non-Int types
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`=` requires at least two arguments",
            None,
        ));
    };

    let Some(a) = a.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not an Int"),
            a.range(),
        ));
    };

    let Some(b) = b.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not an Int"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a == b))
}

pub fn eq_float(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #todo support overloading,
    // #todo make equality a method of Expr?
    // #todo support non-Int types
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`=` requires at least two arguments",
            None,
        ));
    };

    let Some(a) = a.as_float() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not a Float"),
            a.range(),
        ));
    };

    let Some(b) = b.as_float() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not a Float"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a == b))
}

pub fn eq_string(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #todo support overloading,
    // #todo make equality a method of Expr?
    // #todo support non-Int types
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`=` requires at least two arguments",
            None,
        ));
    };

    let Some(a) = a.as_string() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not a String"),
            a.range(),
        ));
    };

    let Some(b) = b.as_string() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not a String"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a == b))
}

// #insight handles both (quoted) Symbol and KeySymbol, they are the same thing anyway.
pub fn eq_symbol(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #todo support overloading,
    // #todo make equality a method of Expr?
    // #todo support non-Int types
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`=` requires at least two arguments",
            None,
        ));
    };

    let Some(a) = a.try_symbol() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not a Symbol"),
            a.range(),
        ));
    };

    let Some(b) = b.try_symbol() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not a Symbol"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a == b))
}

pub fn not_eq(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #todo support overloading,
    // #todo make equality a method of Expr?
    // #todo support non-Int types
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`!=` requires at least two arguments", // #todo what should be the symbol?
            None,
        ));
    };

    let Some(a) = a.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not an Int"),
            a.range(),
        ));
    };

    let Some(b) = b.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not an Int"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a != b))
}

pub fn not_eq_float(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #todo support overloading,
    // #todo make equality a method of Expr?
    // #todo support non-Int types
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`!=` requires at least two arguments",
            None,
        ));
    };

    let Some(a) = a.as_float() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not a Float"),
            a.range(),
        ));
    };

    let Some(b) = b.as_float() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not a Float"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a != b))
}

pub fn not_eq_string(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #todo support overloading,
    // #todo make equality a method of Expr?
    // #todo support non-Int types
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`!=` requires at least two arguments",
            None,
        ));
    };

    let Some(a) = a.as_string() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not a String"),
            a.range(),
        ));
    };

    let Some(b) = b.as_string() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not a String"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a != b))
}

// #insight handles both (quoted) Symbol and KeySymbol, they are the same thing anyway.
pub fn not_eq_symbol(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #todo support overloading,
    // #todo make equality a method of Expr?
    // #todo support non-Int types
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`!=` requires at least two arguments",
            None,
        ));
    };

    let Some(a) = a.try_symbol() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not a String"),
            a.range(),
        ));
    };

    let Some(b) = b.try_symbol() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not a Symbol"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a != b))
}

pub fn gt(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`>` requires at least two arguments",
            None,
        ));
    };

    let Some(a) = a.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not an Int"),
            a.range(),
        ));
    };

    let Some(b) = b.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not an Int"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a > b))
}

pub fn lt(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`<` requires at least two arguments",
            None,
        ));
    };

    let Some(a) = a.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not an Int"),
            a.range(),
        ));
    };

    let Some(b) = b.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not an Int"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a < b))
}
