use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

// #insight `and` cannot be implemented with a function, needs a macro or a special form.
// pub fn bool_and(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
// ...
// }

pub fn bool_not(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo consider binary/bitmask version.
    // #todo consider operator `~` (_not_ `!`)

    let [value] = args else {
        return Err(Error::invalid_arguments("expects one argument", None));
    };

    let Some(predicate) = value.as_bool() else {
        return Err(Error::invalid_arguments(
            "`not` argument should be boolean",
            value.range(),
        ));
    };

    Ok(Expr::Bool(!predicate))
}

pub fn setup_lib_bool(context: &mut Context) {
    // #todo move to a 'bool' or 'boolean' module and import some functions to prelude.
    let module = require_module("prelude", context);

    // #todo better name?
    module.insert("not", Expr::ForeignFunc(Arc::new(bool_not)));
}
