use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

// #todo consider to associate most functions to the `Path` type.
// #todo support (path :extension)
// #todo support (path :full-extension)
// #todo support (path :filename)
// #todo support (path :directory)
// #todo implement (get-parent ..)

fn get_dirname(path: &str) -> Option<&str> {
    if let Some(slash_position) = path.rfind('/') {
        Some(&path[0..slash_position])
    } else {
        None
    }
}

// #todo consider moving to util, but what if we extract the foreign-library implementation?
// #todo also support getting the last part of the extension.
// #todo optimize this.
pub fn get_full_extension(path: &str) -> Option<&str> {
    if let Some(dot_position) = path.find('.') {
        if dot_position == 0 {
            // This is a hidden file, skip the leading dot and try again.
            get_full_extension(&path[1..])
        } else {
            Some(&path[(dot_position + 1)..])
        }
    } else {
        None
    }
}

// #todo should it include the final `/`?
/// Returns the directory part of a path.
pub fn path_get_dirname(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments("requires a `path` argument", None));
    };

    // #todo in the future check for `Path`.
    // #todo return Maybe::None if no directory found.

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a String",
            path.range(),
        ));
    };

    // #todo should return a Maybe.
    let dirname = get_dirname(path).unwrap_or("");

    // #todo should return a `Path` value.

    Ok(Expr::string(dirname))
}

/// Returns the 'full' extension of a path.
pub fn path_get_extension(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments("requires a `path` argument", None));
    };

    // #todo in the future check for `Path`.
    // #todo return Maybe::None if no extension found.

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a String",
            path.range(),
        ));
    };

    // #todo should return a Maybe.
    let extension = get_full_extension(path).unwrap_or("");

    // #todo should return a `Path` value.

    Ok(Expr::string(extension))
}

pub fn setup_lib_path(context: &mut Context) {
    let module = require_module("path", context);

    // #todo think of a better name.
    module.insert("get-dirname", Expr::ForeignFunc(Arc::new(path_get_dirname)));

    // #todo think of a better name.
    module.insert(
        "get-extension",
        Expr::ForeignFunc(Arc::new(path_get_extension)),
    );
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context};

    #[test]
    fn path_get_dirname_usage() {
        let input = r#"
            (use path)
            (path/get-dirname "/home/george/data/users.data.tan")
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let dirname = expr.as_string().unwrap();
        // #todo #think should it include the trailing `/`?
        assert_eq!(dirname, "/home/george/data");

        let input = r#"
            (use path)
            (path/get-dirname "users.data.tan")
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let dirname = expr.as_string().unwrap();
        // #todo should return Maybe::None.
        assert_eq!(dirname, "");
    }

    #[test]
    fn path_get_extension_usage() {
        let input = r#"
            (use path)
            (path/get-extension "users.data.tan")
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let extension = expr.as_string().unwrap();
        assert_eq!(extension, "data.tan");

        let input = r#"
            (use path)
            (path/get-extension ".secret-file.data.tan")
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let extension = expr.as_string().unwrap();
        assert_eq!(extension, "data.tan");
    }
}
