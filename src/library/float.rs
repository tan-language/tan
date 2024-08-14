use core::f64;
use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{
        args::{unpack_bool_arg, unpack_float_arg, unpack_stringable_arg},
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

// #todo Introduce Float/+Infinity, Float/-Infinity.

// #todo Consider skipping the prelude for min?
// #todo What could be another name instead of min? `min-of`? `minimum`?
pub fn float_min(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let mut min = f64::MAX;

    for arg in args {
        let Some(n) = arg.as_float() else {
            return Err(Error::invalid_arguments(
                &format!("{arg} is not a Float"),
                arg.range(),
            ));
        };
        if n < min {
            min = n;
        }
    }

    Ok(Expr::Float(min))
}

pub fn float_max(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let mut max = f64::MIN;

    for arg in args {
        let Some(n) = arg.as_float() else {
            return Err(Error::invalid_arguments(
                &format!("{arg} is not a Float"),
                arg.range(),
            ));
        };
        if n > max {
            max = n;
        }
    }

    Ok(Expr::Float(max))
}

// #todo Implement in Tan.
pub fn float_abs(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let n = unpack_float_arg(args, 0, "n")?;
    Ok(Expr::Float(n.abs()))
}

pub fn float_sqrt(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let n = unpack_float_arg(args, 0, "n")?;
    Ok(Expr::Float(n.sqrt()))
}

// #todo Introduce clamp

pub fn setup_lib_float(context: &mut Context) {
    // #todo put in 'float' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    // #todo consider to-float instead?

    // #todo #think having so many overloads can cover issues, e.g. use the wrong implicit overload.

    // #todo make `float_new` the default.
    module.insert("Float", Expr::ForeignFunc(Arc::new(float_from_int)));
    module.insert("Float$$Int", Expr::ForeignFunc(Arc::new(float_from_int)));
    module.insert("Float$$Bool", Expr::ForeignFunc(Arc::new(float_from_bool)));
    module.insert(
        "Float$$String",
        Expr::ForeignFunc(Arc::new(float_from_string)),
    );
    module.insert("min", Expr::ForeignFunc(Arc::new(float_min)));
    module.insert(
        "min$$Float$$Float",
        // annotate_type(Expr::ForeignFunc(Arc::new(add_float)), "Float"),
        Expr::ForeignFunc(Arc::new(float_min)),
    );
    module.insert("max", Expr::ForeignFunc(Arc::new(float_max)));
    module.insert(
        "max$$Float$$Float",
        // annotate_type(Expr::ForeignFunc(Arc::new(add_float)), "Float"),
        Expr::ForeignFunc(Arc::new(float_max)),
    );

    module.insert("abs", Expr::ForeignFunc(Arc::new(float_abs)));
    module.insert("abs$$Float", Expr::ForeignFunc(Arc::new(float_abs)));

    // #todo Note that `sqrt` does not follow Tan naming conventions but it's a standard term.
    module.insert("sqrt", Expr::ForeignFunc(Arc::new(float_sqrt)));
    module.insert("sqrt$$Float", Expr::ForeignFunc(Arc::new(float_sqrt)));

    // Constants.

    // #warning Don't use those yet!
    // #todo Fix Float/max, it self-evaluates, duh!
    // #todo Mark as constant / make immutable?
    // #todo Should we skip `Float/` prefix?
    // #todo Rename to max-value?
    module.insert("float/max", Expr::Float(f64::MAX));
    module.insert("float/infinity", Expr::Float(f64::INFINITY));
}
