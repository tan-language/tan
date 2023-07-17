use std::fs;
use std::{rc::Rc, sync::Arc};

use crate::{context::Context, error::Error, expr::Expr};

use crate::module::Module;

// #TODO do FFI functions really need an env?
// #TODO differentiate pure functions that do not change the env!

// File < Resource
// #TODO extract file-system-related functionality to `fs` or even the more general `rs` == resource space.
// #TODO consider mapping `:` to `__` and use #[allow(snake_case)]

/// Reads the contents of a text file as a string.
pub fn file_read_as_string(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments("`read_as_string` requires a `path` argument", None));
    };

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments("`path` argument should be a String", path.range()));
    };

    let contents = fs::read_to_string(path)?;

    Ok(Expr::String(contents))
}

// #TODO decide on the parameters order.
pub fn file_write_string(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [path, content] = args else {
        return Err(Error::invalid_arguments("`read_as_string` requires `path` and `content` arguments", None));
    };

    let Expr::String(path) = path.unpack() else {
        return Err(Error::invalid_arguments("`path` argument should be a String", path.range()));
    };

    let Expr::String(content) = content.unpack() else {
        return Err(Error::invalid_arguments("`content` argument should be a String", content.range()));
    };

    fs::write(path, content)?;

    Ok(Expr::One)
}

// #todo use Rc/Arc consistently
// #todo some helpers are needed here, to streamline the code.

pub fn setup_std_fs(context: &mut Context) {
    let module = Module::new("fs", context.top_scope.clone());

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
