// Function argument-parsing utility functions.

use std::{
    any::Any,
    sync::{Arc, RwLockReadGuard},
};

use crate::{error::Error, expr::Expr};

// #todo convert those to macros.
// #todo think about proper name for these functions.
// #todo #warning not used yet.

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

pub fn unpack_foreign_struct_arg(
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

    let Expr::ForeignStruct(s) = expr.unpack() else {
        return Err(Error::invalid_arguments(
            &format!("invalid 111 {type_name}"),
            expr.range(),
        ));
    };

    // #insight Arc::clone is cheap.
    Ok(s.clone())
}
