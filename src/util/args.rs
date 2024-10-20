// Function argument-parsing utility functions.

use std::{
    any::Any,
    collections::HashMap,
    ops::Range,
    sync::{Arc, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{error::Error, expr::Expr};

// #todo move to eval/utils or something.
// #todo convert those to macros.
// #todo think about proper name for these functions.

// #todo reduce this in the other functions here.
pub fn unpack_arg<'a>(args: &'a [Expr], index: usize, name: &str) -> Result<&'a Expr, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        return Err(Error::invalid_arguments(
            &format!("missing required argument `{name}` at index ${index}"),
            None,
        ));
    };

    Ok(expr.unpack())
}

pub fn unpack_bool_arg(args: &[Expr], index: usize, name: &str) -> Result<bool, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Bool argument `{name}`"),
            None,
        ));
    };

    let Some(n) = expr.as_bool() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Bool argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(n)
}

pub fn unpack_char_arg(args: &[Expr], index: usize, name: &str) -> Result<char, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Char argument `{name}`"),
            None,
        ));
    };

    let Some(c) = expr.as_char() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Char argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(c)
}

pub fn unpack_int_arg(args: &[Expr], index: usize, name: &str) -> Result<i64, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Int argument `{name}`"),
            None,
        ));
    };

    let Some(n) = expr.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Int argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(n)
}

pub fn unpack_u8_arg(args: &[Expr], index: usize, name: &str) -> Result<u8, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required U8 argument `{name}`"),
            None,
        ));
    };

    let Some(n) = expr.as_u8() else {
        return Err(Error::invalid_arguments(
            &format!("invalid U8 argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(n)
}

pub fn unpack_float_arg(args: &[Expr], index: usize, name: &str) -> Result<f64, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Float argument `{name}`"),
            None,
        ));
    };

    let Some(n) = expr.as_float() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Float argument: {name}=`{expr}`"),
            // #todo Use the correct range here.
            // #insight expr.range() is wrong, it's the range of the arg definition.
            // #insight None is wrong, it's the range of the upstream call function.
            expr.range(),
        ));
    };

    Ok(n)
}

pub fn unpack_stringable_arg<'a>(
    args: &'a [Expr],
    index: usize,
    name: &str,
) -> Result<&'a str, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Stringable argument `{name}`"),
            None,
        ));
    };

    let Some(s) = expr.as_stringable() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Stringable argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(s)
}

pub fn unpack_symbolic_arg<'a>(
    args: &'a [Expr],
    index: usize,
    name: &str,
) -> Result<&'a str, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Symbolic argument `{name}`"),
            None,
        ));
    };

    let Some(s) = expr.as_symbolic() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Symbolic argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(s)
}

pub fn unpack_map_arg<'a>(
    args: &'a [Expr],
    index: usize,
    name: &str,
) -> Result<RwLockReadGuard<'a, HashMap<String, Expr>>, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Map argument `{name}`"),
            None,
        ));
    };

    let Some(map) = expr.as_map() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Map argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(map)
}

pub fn unpack_map_mut_arg<'a>(
    args: &'a [Expr],
    index: usize,
    name: &str,
) -> Result<RwLockWriteGuard<'a, HashMap<String, Expr>>, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Map argument `{name}`"),
            None,
        ));
    };

    let Some(map) = expr.as_map_mut() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Map argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(map)
}

// #todo Also support lists and other iterables?
// #todo Also add _mut version.
pub fn unpack_array_arg<'a>(
    args: &'a [Expr],
    index: usize,
    name: &str,
) -> Result<RwLockReadGuard<'a, Vec<Expr>>, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Array argument `{name}`"),
            None,
        ));
    };

    let Some(array) = expr.as_array() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Array argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(array)
}

pub fn unpack_float_range_arg(
    args: &[Expr],
    index: usize,
    name: &str,
) -> Result<Range<f64>, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Float-Range argument `{name}`"),
            None,
        ));
    };

    let Some(r) = expr.as_float_range() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Float Range argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(r)
}

// #todo also add _mut version.
pub fn unpack_buffer_arg<'a>(
    args: &'a [Expr],
    index: usize,
    name: &str,
) -> Result<RwLockReadGuard<'a, Vec<u8>>, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required Buffer argument `{name}`"),
            None,
        ));
    };

    let Some(buffer) = expr.as_buffer() else {
        return Err(Error::invalid_arguments(
            &format!("invalid Buffer argument: {name}=`{expr}`"),
            expr.range(),
        ));
    };

    Ok(buffer)
}

// #todo also introduce unpack_foreign_struct_mut_arg

pub fn unpack_foreign_arg(
    args: &[Expr],
    index: usize,
    name: &str,
    type_name: &str,
) -> Result<Arc<dyn Any + Sync + Send>, Error> {
    // #todo also verify the type_name!!!

    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required {type_name} argument `{name}`"),
            None,
        ));
    };

    let Expr::Foreign(s) = expr.unpack() else {
        return Err(Error::invalid_arguments(
            &format!("invalid 111 {type_name}"),
            expr.range(),
        ));
    };

    // #insight Arc::clone is cheap.
    Ok(s.clone())
}
