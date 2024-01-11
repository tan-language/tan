use std::sync::Arc;

use crate::expr::annotate_type;
use crate::util::module_util::require_module;
use crate::{context::Context, expr::Expr};

// #todo remove granular imports
use super::arithmetic;
use super::cmp::setup_lib_cmp;
use super::dict::setup_lib_dict;
use super::eq::setup_lib_eq;
use super::io::setup_lib_io;
use super::seq::setup_lib_seq;
use super::string::setup_lib_string;

// #todo instead of evaluating in prelude maybe it's better to use the functions from the actual modules?
pub fn setup_lib_prelude(context: &mut Context) {
    // #todo maybe context.require_module(path) instead?
    let module = require_module("prelude", context);

    // num

    // #todo forget the mangling, implement with a dispatcher function, multi-function.
    module.insert(
        "+",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_int)), "Int"),
    );
    module.insert(
        "+$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_int)), "Int"),
    );
    module.insert(
        "+$$Float$$Float",
        // #todo add the proper type: (Func Float Float Float)
        // #todo even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_float)), "Float"),
    );
    #[cfg(feature = "dec")]
    module.insert(
        "+$$Dec$$Dec",
        // #todo add the proper type: (Func Dec Dec Dec)
        // #todo even better: (Func (Many Dec) Dec)
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_dec)), "Dec"),
    );
    module.insert("-", Expr::ForeignFunc(Arc::new(arithmetic::sub_int)));
    module.insert(
        "-$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::sub_int)), "Int"),
    );
    module.insert(
        "-$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::sub_float)), "Float"),
    );
    module.insert("*", Expr::ForeignFunc(Arc::new(arithmetic::mul_int)));
    module.insert(
        "*$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::mul_int)), "Int"),
    );
    module.insert(
        "*$$Float$$Float",
        // #todo add the proper type: (Func Float Float Float)
        // #todo even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::mul_float)), "Float"),
    );
    module.insert(
        "/",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::div_float)), "Float"),
    );
    // #todo ultra-hack
    module.insert(
        "/$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::div_float)), "Float"),
    );
    // #todo ultra-hack
    module.insert(
        "/$$Float$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::div_float)), "Float"),
    );
    module.insert(
        "sin",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::sin_float)), "Float"),
    );
    module.insert(
        "cos",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::cos_float)), "Float"),
    );
    module.insert(
        "**",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::powi_float)), "Float"),
    );

    setup_lib_eq(context);
    setup_lib_cmp(context);
    setup_lib_io(context);
    setup_lib_string(context);
    setup_lib_seq(context);
    setup_lib_dict(context);
}
