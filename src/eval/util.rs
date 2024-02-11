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

// // #todo map module name to path
// // #todo resolve crate.x.y, or this.x.y
// pub fn module_path_from_name(name: &str) -> String {
//     todo!()
// }

// (use /math) ; #todo better (use math)
// (use /math :only [pi tau])
// (use /math :exclude [pi])
// (use /math :as "mathematics")
// (use /math [pi tau])
// (use /@gmosx/my-lib) ; #todo better (use @gmosx/my-lib)
// (use http://www.tan.org/modules/my-lib)

// #todo handle module_urls with https://, https://, ipfs://, etc, auto-download, like Deno.

// #todo write unit tests!
// #todo find another name, there is confusion with path_buf::canonicalize.
pub fn canonicalize_module_path(
    path: impl AsRef<Path>,
    context: &Context,
) -> std::io::Result<String> {
    let mut path = path.as_ref().to_string_lossy().into_owned();

    // #todo what is a good coding convention for 'system' variables?
    // #todo support read-only 'system' variables.
    // #todo convert /xxx -> /@std/xxx

    if path.starts_with('@') {
        path = format!("{}/{path}", context.root_path);
    } else if path.starts_with("./") {
        if let Some(base_path) = context.scope.get(CURRENT_MODULE_PATH) {
            let Some(base_path) = base_path.as_string() else {
                // #todo!
                panic!("invalid current-module-path");
            };

            // Canonicalize the path using the current-module-path as the base path.
            path = format!("{base_path}{}", path.strip_prefix('.').unwrap());
        } else {
            // #todo!
            panic!("missing current-module-path");
        }
    } else if path.starts_with('/') {
        path = format!("{}/@std{path}", context.root_path);
    } else if path.starts_with("file://") {
        // #insight used by tan-run.
        path = path[7..].to_string();
    } else {
        path = format!("{}/@std/{path}", context.root_path);
    }
    // } else {
    //     if let Some(base_path) = context.scope.get(CURRENT_MODULE_PATH) {
    //         let Some(base_path) = base_path.as_string() else {
    //             // #todo!
    //             panic!("Invalid current-module-path");
    //         };

    //         // Canonicalize the path using the current-module-path as the base path.
    //         if path.starts_with("./") {
    //             path = format!("{base_path}{}", path.strip_prefix(".").unwrap());
    //         } else {
    //             // #todo consider not supporting this, always require the "./"
    //             path = format!("{base_path}/{}", path);
    //         }
    //     }
    // }

    let canonical_path = canonicalize_path(path);

    Ok(canonical_path)
}

// #todo why does it consume path? this is problematic.
pub fn canonicalize_path(path: String) -> String {
    if let Ok(canonical_path) = PathBuf::from(&path).canonicalize() {
        canonical_path.to_string_lossy().to_string()
    } else {
        path
    }
}

// #todo add unit test.
pub fn compute_module_file_paths(module_path: impl AsRef<Path>) -> std::io::Result<Vec<String>> {
    let module_path = module_path.as_ref();

    let mut module_file_paths: Vec<String> = Vec::new();
    // let mut buf: PathBuf;

    // if !module_path.exists() {
    //     // #todo we don't want the auto-add extensions.
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
    } else if has_tan_extension(module_path) {
        module_file_paths.push(module_path.canonicalize()?.display().to_string());
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "`{}` is not a valid module, unrecognized extension",
                module_path.display()
            ),
        ));
    }

    if module_file_paths.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "no tan files found in directory `{}`",
                module_path.display()
            ),
        ));
    }

    Ok(module_file_paths)
}

// #todo probably need to move at least the 'read' code somewhere else.
// #todo also consider 'rusty' notation: `(use this.sub-module)`

// #insight It's also used in ..use
// #todo split into multiple functions.

/// Evaluates a language module.
pub fn eval_module(
    path: impl AsRef<Path>,
    context: &mut Context,
    force: bool,
) -> Result<Expr, Vec<Error>> {
    // #todo support import_map style rewriting

    let result = canonicalize_module_path(&path, context);

    let Ok(module_path) = result else {
        return Err(vec![result.unwrap_err().into()]);
    };

    // #todo is this really needed?
    let module_name = strip_tan_extension(&module_path);

    // #insight module stem is used as prefix
    let module_stem = {
        if let Some(stem) = path.as_ref().file_stem() {
            stem.to_string_lossy().to_string()
        } else {
            "top-level".to_string() // #todo think about a good name, maybe repl?
        }
    };

    // Lookup into the module_registry first.

    let module = if context.module_registry.contains_key(&module_name) {
        let module = context.module_registry.get(&module_name).unwrap().clone();
        if !force {
            // #insight if not in force mode, just returned the evaluated (cached) module.
            return Ok(Expr::Module(module));
        }
        module
    } else {
        // The module is not registered, try to load it.
        Rc::new(Module::new(module_stem, context.top_scope.clone()))
    };

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

    // #todo return Expr::Module, add module metadata: name, path, exports, etc.

    for file_path in &file_paths {
        // #todo keep all inputs in magic variable in env, associate url/key with error.

        let input = std::fs::read_to_string(file_path);
        let Ok(input) = input else {
            return Err(vec![input.unwrap_err().into()]);
        };

        let result = resolve_string(input, context);

        let Ok(exprs) = result else {
            let mut errors = result.unwrap_err();

            for error in &mut errors {
                error.file_path = file_path.clone();
            }

            // #todo better error handling here!
            // #todo maybe continue parsing/resolving to find more errors?
            return Err(errors);
        };

        for expr in exprs {
            if let Err(mut error) = eval(&expr, context) {
                // #todo add a unit test to check that the file_path is added here!
                error.file_path = file_path.clone();
                // #todo better error here!
                return Err(vec![error]);
            }
        }
    }

    context.scope = prev_scope;
    context.module_registry.insert(module_name, module.clone());

    Ok(Expr::Module(module.clone()))
}
