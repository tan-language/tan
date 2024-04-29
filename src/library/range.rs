use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

// #todo make some fields optional.
// struct ForeignRange<T> {
//     pub start: T,
//     pub end: T,
//     pub step: T,
// }

// #todo Range is Immutable, Copy.

// #todo not used yet.
// fn make_range<T: Send + Sync + 'static>(start: T, end: T, step: T) -> Expr {
//     // #todo use IntRange, FloatRange.
//     let foreign_range = ForeignRange { start, end, step };
//     let expr = Expr::ForeignStruct(Arc::new(foreign_range));
//     // #todo should annotate (Range Int) or (Range Float)
//     annotate_type(expr, "Range")
// }

pub fn range_int_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo make some of the arguments optional, e.g. step.
    let [start, end, step] = args else {
        return Err(Error::invalid_arguments(
            "requires `start`, `end`, and `step` arguments",
            None,
        ));
    };

    // #todo create a helper.
    let Some(start) = start.as_int() else {
        return Err(Error::invalid_arguments(
            "expected Int argument",
            start.range(),
        ));
    };

    let Some(end) = end.as_int() else {
        return Err(Error::invalid_arguments(
            "expected Int argument",
            end.range(),
        ));
    };

    let Some(step) = step.as_int() else {
        return Err(Error::invalid_arguments(
            "expected Int argument",
            step.range(),
        ));
    };

    // #todo use Expr::ForeignStruct
    Ok(Expr::IntRange(start, end, step))
}

pub fn range_float_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo make some of the arguments optional, e.g. step.
    let [start, end, step] = args else {
        return Err(Error::invalid_arguments(
            "requires `start`, `end`, and `step` arguments",
            None,
        ));
    };

    // #todo create a helper.
    let Some(start) = start.as_float() else {
        return Err(Error::invalid_arguments(
            "expected Float argument",
            start.range(),
        ));
    };

    let Some(end) = end.as_float() else {
        return Err(Error::invalid_arguments(
            "expected Float argument",
            end.range(),
        ));
    };

    let Some(step) = step.as_float() else {
        return Err(Error::invalid_arguments(
            "expected Float argument",
            step.range(),
        ));
    };

    // #todo use Expr::ForeignStruct
    Ok(Expr::FloatRange(start, end, step))
}

pub fn setup_lib_range(context: &mut Context) {
    // #todo put in 'range' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    module.insert("Range", Expr::ForeignFunc(Arc::new(range_int_new)));
    module.insert(
        "Range$$Int$$Int$$Int",
        Expr::ForeignFunc(Arc::new(range_int_new)),
    );
    module.insert(
        "Range$$Float$$Float$$Float",
        Expr::ForeignFunc(Arc::new(range_float_new)),
    );
}

// #todo add unit tests.
