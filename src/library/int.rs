use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{args::unpack_bool_arg, module_util::require_module},
};

// #todo Implement with Tan.
pub fn int_from_bool(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let value = unpack_bool_arg(args, 0, "value")?;

    Ok(Expr::Int(if value { 1 } else { 0 }))
}

pub fn setup_lib_int(context: &mut Context) {
    // #todo put in 'int' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    // #todo consider to-int instead?

    // #todo Make `int_from_float` the default.
    module.insert("Int", Expr::ForeignFunc(Arc::new(int_from_bool)));
    module.insert("Int$$Bool", Expr::ForeignFunc(Arc::new(int_from_bool)));
}
