use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    api::{has_tan_extension, resolve_string, strip_tan_extension},
    context::Context,
    error::Error,
    expr::Expr,
    module::Module,
    util::standard_names::CURRENT_MODULE_PATH,
};

use super::eval;

// #todo split read_module, eval(_module)

// #todo not sure that dir as module is a good idea.
// #todo automatically add last path component as prefix, like Go.

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
// (use "@std/math" :only (pi tau))
// (use "@std/math" :exclude (pi))
// (use "@std/math" :as "math")
// (use std.math (pi tau))

// #todo handle module_urls with https://, https://, ipfs://, etc, auto-download, like Deno.

pub fn canonicalize_module_path(
    path: impl AsRef<Path>,
    context: &Context,
) -> std::io::Result<String> {
    let mut path = path.as_ref().to_string_lossy().into_owned();

    // #TODO what is a good coding convention for 'system' variables?
    // #TODO support read-only 'system' variables.
    if let Some(base_path) = context.scope.get(CURRENT_MODULE_PATH) {
        let Some(base_path) = base_path.as_string() else {
            // #TODO!
            panic!("Invalid current-module-path");
        };

        // Canonicalize the path using the current-module-path as the base path.
        if path.starts_with("./") {
            path = format!("{base_path}{}", path.strip_prefix(".").unwrap());
        } else if path.starts_with("/") {
            path = format!("{}{path}", context.root_path);
        } else {
            // #TODO consider not supporting this, always require the "./"
            path = format!("{base_path}/{}", path);
        }
    }

    // #TODO move the canonicalize to the canonicalize_module_path function?
    let path = PathBuf::from(path).canonicalize()?;

    Ok(path.to_string_lossy().to_string())
}

// #TODO add unit test.
pub fn compute_module_file_paths(module_path: impl AsRef<Path>) -> std::io::Result<Vec<String>> {
    let module_path = module_path.as_ref();

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

    if module_file_paths.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "no tan files found in directory `{}`",
                module_path.display().to_string()
            ),
        ));
    }

    Ok(module_file_paths)
}

// #todo probably need to move at least the 'read' code somewhere else.
// #TODO also consider 'rusty' notation: `(use this.sub-module)`

// #insight It's also used in ..use

pub fn eval_module(path: impl AsRef<Path>, context: &mut Context) -> Result<Expr, Vec<Error>> {
    // #todo support import_map style rewriting

    let result = canonicalize_module_path(&path, context);

    let Ok(module_path) = result else {
        return Err(vec![result.unwrap_err().into()]);
    };

    let module_name = strip_tan_extension(&module_path);

    if let Some(module) = context.module_registry.get(&module_name) {
        return Ok(Expr::Module(module.clone()));
    }

    let module = Module::new();

    let prev_scope = context.scope.clone();
    context.scope = module.scope.clone();

    if has_tan_extension(&module_path) {
        // #todo use context.insert_special!
        context.scope.insert(
            CURRENT_MODULE_PATH,
            Expr::string(
                PathBuf::from(&module_path)
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            ),
        );
    } else {
        // #insight only update the current-module-path on 'proper' ('dir') modules.
        context
            .scope
            .insert(CURRENT_MODULE_PATH, Expr::string(&module_path));
    }

    let file_paths = compute_module_file_paths(&module_path);

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

        let result = resolve_string(input, context);

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
            if let Err(mut error) = eval(&expr, context) {
                // #TODO add a unit test to check that the file_path is added here!
                error.file_path = file_path.clone();
                // #TODO better error here!
                return Err(vec![error]);
            }
        }
    }

    context.scope = prev_scope;

    let module = Rc::new(module);
    context.module_registry.insert(module_name, module.clone());

    Ok(Expr::Module(module))
}
