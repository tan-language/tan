use std::sync::Arc;

use uuid::Uuid;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

// #todo UUID object with related methods.
// #todo UUID is a 128bit number, a buffer of 16 bytes.

pub fn uuid_new_v4(_args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let id = Uuid::new_v4();

    // #todo for the moment we use the UUID as a string alias, should be a buffer of 16 bytes?

    Ok(Expr::string(id))
}

pub fn setup_lib_uuid(context: &mut Context) {
    // #todo what is a good path? should avoid math?
    let module = require_module("uuid", context);

    // #todo better name?
    module.insert("make-v4-uuid", Expr::ForeignFunc(Arc::new(uuid_new_v4)));
}
