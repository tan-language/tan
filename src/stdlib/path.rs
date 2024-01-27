use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

// #todo consider to associate most functions to the `Path` type.
// #todo support (path :extension)
// #todo support (path :full-extension)
// #todo support (path :filename)
// #todo support (path :directory)

// #todo optimize this.
fn get_full_extension(path: &str) -> Option<&str> {
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
    module.insert(
        "get-extension",
        Expr::ForeignFunc(Arc::new(path_get_extension)),
    );
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context};

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
