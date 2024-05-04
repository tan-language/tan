use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{args::unpack_int_arg, module_util::require_module},
};

// #todo support all types!

pub fn eq_int(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
    // #todo support overloading,
    // #todo make equality a method of Expr?
    // #todo support non-Int types
    // #todo support multiple arguments.

    // #todo also pass the function name, or at least show the function name upstream.
    let a = unpack_int_arg(args, 0, "a")?;
    let b = unpack_int_arg(args, 1, "b")?;

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

// #insight handles both (quoted) Symbol and KeySymbol, they are the same thing anyway. Also handles Type.
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

    let Some(a) = a.as_symbolic() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not a Symbol"),
            a.range(),
        ));
    };

    let Some(b) = b.as_symbolic() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not a Symbol"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a == b))
}

pub fn not_eq_int(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
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

    let Some(a) = a.as_symbolic() else {
        return Err(Error::invalid_arguments(
            &format!("`{a}` is not a String"),
            a.range(),
        ));
    };

    let Some(b) = b.as_symbolic() else {
        return Err(Error::invalid_arguments(
            &format!("`{b}` is not a Symbol"),
            b.range(),
        ));
    };

    Ok(Expr::Bool(a != b))
}

pub fn int_gt(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
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

pub fn float_gt(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo support multiple arguments.
    let [a, b] = args else {
        return Err(Error::invalid_arguments(
            "`>` requires at least two arguments",
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

// #todo should we have an explicit module for these functions?

pub fn setup_lib_eq(context: &mut Context) {
    let module = require_module("prelude", context);

    module.insert("=", Expr::ForeignFunc(Arc::new(eq_int)));
    module.insert("=$$Int$$Int", Expr::ForeignFunc(Arc::new(eq_int)));
    module.insert("=$$Float$$Float", Expr::ForeignFunc(Arc::new(eq_float)));
    module.insert("=$$String$$String", Expr::ForeignFunc(Arc::new(eq_string)));
    // module.insert("=$$Symbol$$Symbol", Expr::ForeignFunc(Arc::new(eq_symbol)));
    module.insert(
        "=$$KeySymbol$$KeySymbol",
        Expr::ForeignFunc(Arc::new(eq_symbol)),
    );
    // #todo #hack this is nasty!
    module.insert("=$$Type$$Type", Expr::ForeignFunc(Arc::new(eq_symbol)));
    module.insert("=$$Type$$String", Expr::ForeignFunc(Arc::new(eq_symbol)));
    module.insert("=$$Type$$KeySymbol", Expr::ForeignFunc(Arc::new(eq_symbol)));

    module.insert("!=", Expr::ForeignFunc(Arc::new(not_eq_int)));
    module.insert("!=$$Int$$Int", Expr::ForeignFunc(Arc::new(not_eq_int)));
    module.insert(
        "!=$$Float$$Float",
        Expr::ForeignFunc(Arc::new(not_eq_float)),
    );
    module.insert(
        "!=$$String$$String",
        Expr::ForeignFunc(Arc::new(not_eq_string)),
    );
    module.insert(
        "!=$$Symbol$$Symbol",
        Expr::ForeignFunc(Arc::new(not_eq_symbol)),
    );
    module.insert(
        "!=$$KeySymbol$$KeySymbol",
        Expr::ForeignFunc(Arc::new(not_eq_symbol)),
    );

    module.insert(">", Expr::ForeignFunc(Arc::new(int_gt)));
    module.insert(">$$Int$$Int", Expr::ForeignFunc(Arc::new(int_gt)));
    module.insert(">$$Float$$Float", Expr::ForeignFunc(Arc::new(float_gt)));
    module.insert("<", Expr::ForeignFunc(Arc::new(lt)));
}
