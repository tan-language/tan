use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

// #todo rename to nil?/is-nil?
// #insight with dynamic typing you don't strictly need a Maybe type?
pub fn is_none(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [expr] = args else {
        // #todo better error
        return Err(Error::invalid_arguments("one argument expected", None));
    };

    Ok(Expr::Bool(expr.is_none()))
}

pub fn is_some(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [expr] = args else {
        // #todo better error
        return Err(Error::invalid_arguments("one argument expected", None));
    };

    Ok(Expr::Bool(!expr.is_none()))
}

pub fn setup_lib_maybe(context: &mut Context) {
    //. #todo move to `maybe` namespace?
    let module = require_module("prelude", context);

    // #todo use is-some? to make more like a verb?
    // (if (some? user) ...)
    // (if (is-some? user) ...)
    // (if (is-some user) ...)
    module.insert("some?", Expr::ForeignFunc(Arc::new(is_some)));
    module.insert("none?", Expr::ForeignFunc(Arc::new(is_none)));
}

// #todo add unit tests!
