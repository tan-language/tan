use std::sync::Arc;

use crate::context::Context;
use crate::error::Error;
use crate::expr::Expr;
use crate::util::module_util::require_module;

use super::arithmetic::setup_lib_arithmetic;
use super::chrono;
use super::cmp::setup_lib_cmp;
use super::dict::setup_lib_dict;
use super::eq::setup_lib_eq;
use super::io::setup_lib_io;
use super::seq::setup_lib_seq;
use super::string::setup_lib_string;

// #todo temporarily here, move to String?
/// Formats an expression into a string.
pub fn expr_to_string(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    // #todo improve default formatting
    Ok(Expr::string(this.to_string()))
}

// #todo instead of evaluating in prelude maybe it's better to use the functions from the actual modules?
pub fn setup_lib_prelude(context: &mut Context) {
    // #todo maybe context.require_module(path) instead?

    setup_lib_arithmetic(context);
    setup_lib_eq(context);
    setup_lib_cmp(context);
    setup_lib_io(context);
    setup_lib_string(context);
    setup_lib_seq(context);
    setup_lib_dict(context);

    // #todo #temp #hack
    let module = require_module("prelude", context);
    module.insert(
        "to-string",
        Expr::ForeignFunc(Arc::new(expr_to_string)), // #todo #temp
    );
    module.insert(
        "to-string$$Date",
        Expr::ForeignFunc(Arc::new(chrono::chrono_date_to_string)),
    );
}
