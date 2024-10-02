use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{
        args::{unpack_bool_arg, unpack_float_arg, unpack_int_arg, unpack_stringable_arg},
        module_util::require_module,
    },
};

// #todo Make this explicit at call site?
pub fn int_from_float(args: &[Expr]) -> Result<Expr, Error> {
    // #todo support more 'source' types.
    let value = unpack_float_arg(args, 0, "value")?;

    Ok(Expr::Int(value as i64))
}

// #todo Implement with Tan.
pub fn int_from_bool(args: &[Expr]) -> Result<Expr, Error> {
    let value = unpack_bool_arg(args, 0, "value")?;

    Ok(Expr::Int(if value { 1 } else { 0 }))
}

// #todo Support both (Int "123") and (Int/from-string "123")
pub fn int_from_string(args: &[Expr]) -> Result<Expr, Error> {
    let string = unpack_stringable_arg(args, 0, "string")?;
    let Ok(value) = string.parse::<i64>() else {
        return Err(Error::invalid_arguments(
            &format!("string=`{string}` is not a valid Int number"),
            args[0].range(),
        ));
    };
    Ok(Expr::Int(value))
}

// #insight the bitwise functions are not that frequent, no need for operators?
// #todo Or maybe have operators but not in prelude?

pub fn int_bitwise_and(args: &[Expr]) -> Result<Expr, Error> {
    let n = unpack_int_arg(args, 0, "n")?;
    let mask = unpack_int_arg(args, 1, "mask")?;

    Ok(Expr::Int(n & mask))
}

pub fn int_bitwise_or(args: &[Expr]) -> Result<Expr, Error> {
    let n = unpack_int_arg(args, 0, "n")?;
    let mask = unpack_int_arg(args, 1, "mask")?;

    Ok(Expr::Int(n | mask))
}

// #todo Implement int_from_float, should use ceil, floor or round?

pub fn setup_lib_int(context: &mut Context) {
    // #todo put in 'int' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    // #todo consider to-int instead?

    // #todo Make `int_from_float` the default.
    // module.insert_invocable("Int", Expr::foreign_func(&int_from_float));
    module.insert_invocable("Int$$Float", Expr::foreign_func(&int_from_float));
    module.insert_invocable("Int$$Bool", Expr::foreign_func(&int_from_bool));
    module.insert_invocable("Int$$String", Expr::foreign_func(&int_from_string));

    // #todo Add bit-checking functions.
    // #todo Introduce more bitwise operators (xor, etc...)
    // #todo Consider other names and/or operators (&, |)
    // #todo Constrain with Int?
    module.insert_invocable("bit-and", Expr::foreign_func(&int_bitwise_and));
    module.insert_invocable("bit-or", Expr::foreign_func(&int_bitwise_or));
}
