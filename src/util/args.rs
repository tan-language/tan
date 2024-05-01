// Function argument-parsing utility functions.

use std::{
    any::Any,
    sync::{Arc, RwLock},
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

pub fn unpack_foreign_struct_arg<'a>(
    args: &'a [Expr],
    index: usize,
    name: &str,
    type_name: &str,
) -> Result<&'a Arc<RwLock<dyn Any + Sync + Send>>, Error> {
    let Some(expr) = args.get(index) else {
        // #todo introduce 'missing argument' error variant.
        // #todo also report the index.
        return Err(Error::invalid_arguments(
            &format!("missing required {type_name} argument `{name}`"),
            None,
        ));
    };

    let Expr::ForeignStructMut(s) = expr.unpack() else {
        return Err(Error::invalid_arguments(
            &format!("invalid {type_name}"),
            expr.range(),
        ));
    };

    Ok(s)
}
