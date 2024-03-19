use crate::{context::Context, util::module_util::require_module};

// #todo have more sophisticated patterns and matching.
// #toto check swift patterns.

pub fn setup_lib_regex(context: &mut Context) {
    // #todo find a better module-path
    let module = require_module("regex", context);

    // module.insert("get-dirname", Expr::ForeignFunc(Arc::new(path_get_dirname)));

    // #todo think of a better name.
    // module.insert(
    //     "get-extension",
    //     Expr::ForeignFunc(Arc::new(path_get_extension)),
    // );
}
