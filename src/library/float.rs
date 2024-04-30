use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

pub fn float_int_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo make some of the arguments optional, e.g. step.
    let [value] = args else {
        return Err(Error::invalid_arguments("requires `value` argument", None));
    };

    // #todo create a helper.
    let Some(value) = value.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("value=`{value}` is not Int"),
            value.range(),
        ));
    };

    Ok(Expr::Float(value as f64))
}

pub fn setup_lib_float(context: &mut Context) {
    // #todo put in 'float' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    // #todo make `float_new` the default.
    module.insert("Float", Expr::ForeignFunc(Arc::new(float_int_new)));
    module.insert("Float$$Int", Expr::ForeignFunc(Arc::new(float_int_new)));
}
