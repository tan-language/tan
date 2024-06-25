use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    api::{compile_string, has_tan_extension, has_tan_extension_strict, strip_tan_extension},
    context::Context,
    error::Error,
    expr::Expr,
    module::Module,
    scope::Scope,
    util::{
        constants::INPUT_PSEUDO_FILE_PATH,
        standard_names::{CURRENT_FILE_PATH, CURRENT_MODULE_PATH},
    },
};

use super::eval;

// #todo need to introduce profiles.
// #todo skip *.test.tan, *.data.tan, etc unless you are in a special profile!

// #todo find a good name, consider `ground`.
/// If the result is an error, add a range from the 'anchor' expression.
pub fn anchor_error(result: Result<Expr, Error>, expr: &Expr) -> Result<Expr, Error> {
    if let Err(mut error) = result {
        // #todo consider anchoring all notes!
        // #todo notes in error is a hack, needs refactoring.

        // #todo #important #fix It seems that expr some times don't have range here, WHY?

        if let Some(note) = error.notes.first_mut() {
            if note.range.is_none() {
                note.range = expr.range()
            }
        };
        Err(error)
    } else {
        result
    }
}

// #todo split read_module, eval(_module)
// #todo not sure that dir as module is a good idea.

// // #todo map module name to path
// // #todo resolve crate.x.y, or this.x.y
// pub fn module_path_from_name(name: &str) -> String {
//     todo!()
// }

// #todo handle module_urls with https://, https://, ipfs://, etc, auto-download, like Deno.
// #todo the implicit URL scheme is tan://, e.g. tan://math, tan://@group/util
// #todo #think does the `@` mess with the url?
// #todo #think does the leading `/` mess with the url?, e.g. `tan:///math`?
// #todo write unit tests!
// #todo find another name, there is confusion with path_buf::canonicalize.
// #todo remove the _module_ from name, used also for files and dyn-libs.
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
    } else if path.starts_with("../") {
        if let Some(base_path) = context.scope.get(CURRENT_MODULE_PATH) {
            let Some(base_path) = base_path.as_string() else {
                // #todo!
                panic!("invalid current-module-path");
            };

            // Canonicalize the path using the current-module-path as the base path.
            path = format!("{base_path}/{path}");
        } else {
            // #todo!
            panic!("missing current-module-path");
        }
    } else if path.starts_with("file://") {
        // #insight used by tan-run.
        path = path[7..].to_string();
    } else if path.starts_with('/') {
        // #insight the leading `/` is ignored.
        path = format!("{}/@std{path}", context.root_path);
    } else {
        // #todo maybe we should always require the `/` prefix for the standard library?
        path = format!("{}/@std/{path}", context.root_path);
    }

    let canonical_path = canonicalize_path(path);

    Ok(canonical_path)
}

// #todo why does it consume path? this is problematic.
pub fn canonicalize_path(path: String) -> String {
    // #todo should call canonicalize_module_path?
    if let Ok(canonical_path) = PathBuf::from(&path).canonicalize() {
        canonical_path.to_string_lossy().to_string()
    } else {
        path
    }
}

// #todo add unit test.
pub fn compute_module_file_paths(
    module_path: impl AsRef<Path>,
    strict: bool,
) -> std::io::Result<Vec<String>> {
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
            // #insight
            // we check for _strict_ extension, to avoid loading test files,
            // data files, etc.
            // #todo should specifically allow for test.tan, or bench.tan, etc!
            let has_tan_extension = if strict {
                has_tan_extension_strict(&file_path)
            } else {
                has_tan_extension(&file_path)
            };

            if has_tan_extension {
                module_file_paths.push(file_path.canonicalize()?.display().to_string());
            }
        }
    } else if has_tan_extension(module_path) {
        // the module_path has an explicit "extension", treat is as a single-file module.
        // #insight
        // since we use an explicit extension extension here, no need to check
        // for strict extension.
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

// #todo maybe rename both to load_file, load_module?

// #todo used in load_file/(load ...)
pub fn eval_file(path: &str, context: &mut Context) -> Result<Expr, Vec<Error>> {
    // #todo keep all inputs in magic variable in env, associate url/key with error.

    // #todo add CURRENT_FILE_PATH to scope? no -> tan code will be able to access the special variables.
    let old_current_file_path = context.top_scope.get(CURRENT_FILE_PATH);
    context
        .top_scope
        .insert(CURRENT_FILE_PATH, Expr::string(path));

    let input = std::fs::read_to_string(path);
    let Ok(input) = input else {
        return Err(vec![input.unwrap_err().into()]);
    };

    let result = compile_string(input, context);

    let Ok(exprs) = result else {
        let mut errors = result.unwrap_err();

        for error in &mut errors {
            error.file_path = path.to_string();
        }

        // #todo better error handling here!
        // #todo maybe continue parsing/resolving to find more errors?
        return Err(errors);
    };

    let mut value = Expr::None;
    let mut errors = Vec::new();

    for expr in exprs {
        // if let Err(mut error) = eval(&expr, context) {
        match eval(&expr, context) {
            Ok(expr) => value = expr,
            Err(mut error) => {
                // #todo add a unit test to check that the file_path is added here!
                // #todo just make error.file_path optional and avoid this hack here!!!
                if error.file_path == INPUT_PSEUDO_FILE_PATH {
                    error.file_path = path.to_string();
                }

                // #todo better error here!
                errors.push(error);
            }
        }
    }

    if let Some(old_current_file_path) = old_current_file_path {
        // #insight we should revert the previous current file, in case of 'use'
        context
            .top_scope
            .insert(CURRENT_FILE_PATH, old_current_file_path.unpack().clone());
    }

    if errors.is_empty() {
        Ok(value)
    } else {
        Err(errors)
    }
}

/// Evaluates a language module.
pub fn eval_module(
    path: impl AsRef<Path>,
    context: &mut Context,
    force: bool,
) -> Result<Expr, Vec<Error>> {
    // #insight Useful for debugging.
    // println!("***** {}", path.as_ref().to_string_lossy());

    // #todo support import_map style rewriting

    // #todo explore trying module.TAN file if module directory is not found.
    // #todo maybe allow .tan extension in module_path to explicitly load a module _file_.

    let result = canonicalize_module_path(&path, context);

    let Ok(module_path) = result else {
        return Err(vec![result.unwrap_err().into()]);
    };

    // #todo is this really needed?
    let module_path = strip_tan_extension(module_path);

    // #todo add this check here.
    // println!("----1>> {module_path}");
    // if let Some(current_module_path) = context.top_scope.get(CURRENT_MODULE_PATH) {
    //     println!("----2>> {module_path}");
    //     if let Some(current_module_path) = current_module_path.as_stringable() {
    //         println!("----3>> {current_module_path} == {module_path}");
    //         if current_module_path == module_path {
    //             // #todo return some other kind of error!
    //             // #todo rephrase this error.
    //             println!("cannot load the same module as the current module");
    //             return Err(vec![Error::invalid_arguments(
    //                 "cannot load the same module as the current module",
    //                 None,
    //             )]);
    //         }
    //     }
    // }

    // context
    //     .top_scope
    //     .insert(CURRENT_MODULE_PATH, Expr::string(&module_path));

    // #insight module stem is used as prefix
    let module_stem = {
        if let Some(stem) = path.as_ref().file_stem() {
            stem.to_string_lossy().to_string()
        } else {
            "top-level".to_string() // #todo think about a good name, maybe repl?
        }
    };

    // Lookup into the module_registry first.

    // #todo optimize this lookup, no need for contains_key!
    let module = if context.module_registry.contains_key(&module_path) {
        let module = context.module_registry.get(&module_path).unwrap().clone();
        if !force {
            // #insight if not in force mode, just returned the evaluated (cached) module.
            return Ok(Expr::Module(module));
        }
        module
    } else {
        // The module is not registered, try to load it.
        Arc::new(Module::new(module_stem, context.top_scope.clone()))
    };

    // #todo consider inserting the new module here to avoid recursive module evals.
    // context
    //     .module_registry
    //     .insert(module_name.clone(), module.clone());

    // #insight pre-inserting the module also enables dyn-libs.
    context
        .module_registry
        .insert(module_path.clone(), module.clone());

    // #todo avoid the module.scope.clone()
    let prev_scope = context.scope.clone();
    context.scope = module.scope.clone();
    // #insight #IMPORTANT make sure the scope is restored before all exit points of this function!!!
    // #todo need a push_scope helper on context that uses Drop to emulate defer?
    // #todo e.g. it could return a prev_scope ScopeGuard!
    // let prev_scope = std::mem::replace(&mut context.scope, module.scope.clone());

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

    let strict = !context.is_test_profile();
    let file_paths = compute_module_file_paths(&module_path, strict);

    let Ok(file_paths) = file_paths else {
        // #todo argh! we need something like defer here!
        // #todo could use defer_lite crate and the defer! macro!!
        context.scope = prev_scope;
        return Err(vec![file_paths.unwrap_err().into()]);
    };

    // #todo return Expr::Module, add module metadata: name, path, exports, etc.
    // #todo collect errors from all module files!

    // #todo prohibit recursive module_evals.

    for file_path in &file_paths {
        if let Err(error) = eval_file(file_path, context) {
            context.scope = prev_scope;
            return Err(error);
        }
    }

    // #todo #IMPORTANT add a unit test to verify that the original scope is restored!

    context.scope = prev_scope;
    // #todo consider pre-inserting the module, see above!
    context.module_registry.insert(module_path, module.clone());

    Ok(Expr::Module(module.clone()))
}

/// Returns the binding within a scope that match the given prefix.
/// Does not search ancestors. Used in ...`use`.
pub fn get_bindings_with_prefix(
    scope: &Scope,
    prefix: impl AsRef<str>,
) -> Vec<(String, Arc<Expr>)> {
    let name = prefix.as_ref();
    let prefix = format!("{name}$$");

    let scope_bindings = scope.bindings.read().expect("poisoned lock");

    let mut matched_bindings = Vec::new();

    for key in scope_bindings.keys() {
        if key == name || key.starts_with(&prefix) {
            matched_bindings.push((key.clone(), scope_bindings.get(key).unwrap().clone()));
        }
    }

    matched_bindings
}

pub fn get_current_file_path(context: &Context) -> String {
    // #todo optimize!
    context
        .top_scope
        .get(CURRENT_FILE_PATH)
        // #todo think about how to best handle this.
        // #insight use unwrap_or_else to be more fault tolerant, when no file is available (eval_string, repl, etc...)
        .unwrap_or_else(|| Arc::new(Expr::string("UNKNOWN")))
        .as_string()
        .unwrap()
        .to_string()
}
#[cfg(test)]
mod tests {
    use crate::{
        context::Context, eval::util::compute_module_file_paths, expr::Expr,
        util::standard_names::CURRENT_MODULE_PATH,
    };

    use super::canonicalize_module_path;

    #[test]
    fn canonicalize_module_path_should_handle_relative_urls() {
        let context = Context::new();
        context.insert(CURRENT_MODULE_PATH, Expr::string("root"), false);
        let path = canonicalize_module_path("./hello/world", &context).unwrap();
        assert_eq!("root/hello/world", path);
    }

    #[test]
    fn compute_module_file_paths_returns_only_strict_tan_files() {
        let paths = compute_module_file_paths("tests/fixtures/dummy-module", true).unwrap();
        assert_eq!(paths.len(), 2);
        for path in paths {
            assert!(!path.ends_with(".test.tan"));
        }

        let paths = compute_module_file_paths("tests/fixtures/dummy-module", false).unwrap();
        assert_eq!(paths.len(), 3);
    }

    // #todo add unit test for `../` paths.
}
