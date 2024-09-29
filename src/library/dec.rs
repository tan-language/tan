use std::str::FromStr;

use rust_decimal::Decimal;

use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{args::unpack_stringable_arg, module_util::require_module},
};

// #todo Implement Dec/from-int.
// #todo Implement Dec/from-float.
// #todo Implement Dec/from-string.

// #todo Consider (Dec/from-string ...)
pub fn dec_from_string(args: &[Expr]) -> Result<Expr, Error> {
    let string = unpack_stringable_arg(args, 0, "string")?;
    let Ok(value) = Decimal::from_str(string) else {
        return Err(Error::invalid_arguments(
            &format!("string=`{string}` is not a valid Dec number"),
            args[0].range(),
        ));
    };
    Ok(Expr::Dec(value))
}

pub fn setup_lib_dec(context: &mut Context) {
    // #todo put in 'dec' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    // #todo consider to-dec instead?

    module.insert_invocable("Dec", Expr::foreign_func(&dec_from_string));
    module.insert_invocable("Dec$$String", Expr::foreign_func(&dec_from_string));
}
