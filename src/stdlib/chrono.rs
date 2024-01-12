use crate::{context::Context, util::module_util::require_module};

// #todo consider another namespace?

// #todo consider adding some chrono types to the prelude.

pub fn setup_lib_chrono(context: &mut Context) {
    let module = require_module("chrono", context);
    // #todo
}
