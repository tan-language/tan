use std::{fs, path::Path};

use crate::{
    api::{has_tan_extension, resolve_string},
    error::Error,
    expr::Expr,
};

use super::{env::Env, eval};

// // #TODO map module name to path
// // #TODO resolve crate.x.y, or this.x.y
// pub fn module_path_from_name(name: &str) -> String {
//     todo!()
// }

// #TODO
// Alternative module-path syntax:
//
// (use "@gmosx/playground/ray")
// (use gmosx.playground.ray)
//
// (use "./sub/module")
// (use this.sub.module)
//
// (use "@std/math" (pi tau))
// (use std.math (pi tau))

// #TODO add unit test.
pub fn compute_module_file_paths(path: impl AsRef<Path>) -> std::io::Result<Vec<String>> {
    let module_path = path.as_ref().canonicalize()?;

    let mut module_file_paths: Vec<String> = Vec::new();
    // let mut buf: PathBuf;

    // if !module_path.exists() {
    //     // #TODO we don't want the auto-add extensions.
    //     // Automatically try adding the tan extensions.

    //     buf = module_path.to_path_buf();
    //     buf.set_extension(TAN_FILE_EXTENSION);
    //     module_path = buf.as_path();

    //     if !module_path.exists() {
    //         buf = module_path.to_path_buf();
    //         buf.set_extension(TAN_FILE_EMOJI_EXTENSION);
    //         module_path = buf.as_path();
    //     }

    if !module_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("`{}` not found", module_path.display()),
        ));
    }

    // }

    if module_path.is_dir() {
        let file_paths = fs::read_dir(module_path)?;

        for file_path in file_paths {
            let file_path = file_path?.path();
            if has_tan_extension(&file_path) {
                module_file_paths.push(file_path.canonicalize()?.display().to_string());
            }
        }
    } else if has_tan_extension(&module_path) {
        module_file_paths.push(module_path.canonicalize()?.display().to_string());
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "`{}` is not a valid module, unrecognized extension",
                module_path.display().to_string()
            ),
        ));
    }

    Ok(module_file_paths)
}

// #TODO also consider 'rusty' notation: `(use this.sub-module)`

// #insight It's also used in ..use

pub fn eval_module(path: impl AsRef<Path>, env: &mut Env) -> Result<Expr, Vec<Error>> {
    // #TODO more general solution needed.

    let mut path = path.as_ref().to_string_lossy().into_owned();

    // #TODO what is a good coding convention for 'system' variables?
    // #TODO support read-only 'system' variables.
    if let Some(base_path) = env.get("*current-module-path*") {
        let Expr::String(base_path) = base_path else {
            // #TODO!
            panic!("Invalid current-module-path");
        };

        // Canonicalize the path using the current-module-path as the base path.
        if path.starts_with("./") {
            path = format!("{base_path}{}", path.strip_prefix(".").unwrap());
        } else if !path.starts_with("/") {
            // #TODO consider not supporting this, always require the "./"
            path = format!("{base_path}/{}", path);
        }
    }

    // #TODO this is not really working, we need recursive, 'folded' environments, but it will do for the moment.
    env.insert("*current-module-path*", Expr::String(path.clone()));

    let file_paths = compute_module_file_paths(path);

    let Ok(file_paths) = file_paths else {
        return Err(vec![file_paths.unwrap_err().into()]);
    };

    // #TODO return Expr::Module, add module metadata: name, path, exports, etc.

    for file_path in &file_paths {
        // #TODO keep all inputs in magic variable in env, associate url/key with error.

        let input = std::fs::read_to_string(&file_path);
        let Ok(input) = input else {
            return Err(vec![input.unwrap_err().into()]);
        };

        let result = resolve_string(input, env);

        let Ok(exprs) = result else {
            let mut errors = result.unwrap_err();

            for error in &mut errors {
                error.file_path = file_path.clone();
            }

            // #TODO better error handling here!
            // #TODO maybe continue parsing/resolving to find more errors?
            return Err(errors);
        };

        for expr in exprs {
            if let Err(mut error) = eval(&expr, env) {
                error.file_path = file_path.clone();
                // #TODO better error here!
                return Err(vec![error]);
            }
        }
    }

    // #TODO what should we return here? the last value.
    Ok(Expr::One.into())
}
