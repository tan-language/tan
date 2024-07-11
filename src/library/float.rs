use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{
        args::{unpack_bool_arg, unpack_stringable_arg},
        module_util::require_module,
    },
};

// #todo Implement with Tan.
pub fn float_from_int(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
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

// #todo Implement with Tan.
pub fn float_from_bool(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let value = unpack_bool_arg(args, 0, "value")?;

    Ok(Expr::Float(if value { 1.0 } else { 0.0 }))
}

// #todo Consider (Float/from-string ...)
pub fn float_from_string(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let string = unpack_stringable_arg(args, 0, "string")?;
    let Ok(value) = string.parse::<f64>() else {
        return Err(Error::invalid_arguments(
            &format!("string=`{string}` is not a valid Float number"),
            args[0].range(),
        ));
    };
    Ok(Expr::Float(value))
}

pub fn setup_lib_float(context: &mut Context) {
    // #todo put in 'float' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    // #todo consider to-float instead?

    // #todo make `float_new` the default.
    module.insert("Float", Expr::ForeignFunc(Arc::new(float_from_int)));
    module.insert("Float$$Int", Expr::ForeignFunc(Arc::new(float_from_int)));
    module.insert("Float$$Bool", Expr::ForeignFunc(Arc::new(float_from_bool)));
    module.insert(
        "Float$$String",
        Expr::ForeignFunc(Arc::new(float_from_string)),
    );
}
