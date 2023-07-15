pub mod fs;
pub mod io;

use std::{rc::Rc, sync::Arc};

use crate::{
    context::Context,
    expr::Expr,
    module::Module,
    stdlib::fs::{file_read_as_string, file_write_string},
};

// #todo consider removing the `std` prefix from module paths, like haskell.
// #todo find a better prefix than setup_
// #todo use Rc/Arc consistently

pub fn setup_std_fs(context: &mut Context) {
    let module = Module::new("fs");

    // #todo aaargh!!! module has all copied of prelude

    let scope = &module.scope;

    scope.insert(
        "read-file-to-string",
        Expr::ForeignFunc(Arc::new(file_read_as_string)),
    );
    scope.insert(
        "read-file-to-string$$String",
        Expr::ForeignFunc(Arc::new(file_read_as_string)),
    );

    // #TODO consider just `write`.
    scope.insert(
        // #TODO alternatives: "std:fs:write_string", "std:url:write_string", "str.url.write-string"
        "write-string-to-file",
        Expr::ForeignFunc(Arc::new(file_write_string)),
    );

    scope.insert(
        "write-string-to-file$$String",
        Expr::ForeignFunc(Arc::new(file_write_string)),
    );

    // #todo this is a hack.
    let module_path = format!("{}/std/fs", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}

pub fn setup_std(context: &mut Context) {
    setup_std_fs(context);
}
