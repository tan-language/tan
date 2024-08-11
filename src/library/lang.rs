use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use libloading::Library;

use crate::{
    api::compile_string,
    context::Context,
    error::Error,
    eval::{
        eval,
        util::{canonicalize_module_path, eval_file},
    },
    expr::{annotate, expr_clone, Expr},
    util::{
        args::{unpack_arg, unpack_map_arg, unpack_stringable_arg, unpack_symbolic_arg},
        module_util::require_module,
        standard_names::CURRENT_MODULE_PATH,
    },
};

// #todo move to another place.
static FOREIGN_DYN_LIB_MAP: OnceLock<Mutex<HashMap<String, Library>>> = OnceLock::new();

// #todo (if (= (get-type obj) Amount) ...) ; type, get-type, type-of
// #todo implement `type?` e.g. (if (type? obj Amount) ...)
// #todo op to set annotation.

// #todo consider meta instead of ann
// #todo consider get-ann? or (better ann-of)
// #todo where is this used?
// #todo extract *_impl function.
pub fn ann(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    if args.len() != 1 {
        return Err(Error::invalid_arguments(
            "`ann` requires one argument",
            None,
        ));
    }

    // #todo support multiple arguments.

    let expr = args.first().unwrap();

    let expr = eval(expr, context)?;

    if let Some(ann) = expr.annotations() {
        Ok(Expr::map(ann.clone()))
    } else {
        // #todo what to return here?
        // Ok(Expr::map(HashMap::new()))
        Ok(Expr::None)
    }
}

// #insight clojure passes the expression as the first argument.
// #todo consider 'inverse' design, e.g. (with-ann anns expr)
// (with-ann expr {:type Amount})
pub fn with_ann(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let annotations = unpack_map_arg(args, 0, "annotations")?;
    let target = unpack_arg(args, 1, "target")?;
    Ok(Expr::annotated(expr_clone(target), &annotations))
}

// #todo implement (with-type ...) with Tan code?
pub fn with_type(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let type_expr = unpack_arg(args, 0, "type")?;
    let target = unpack_arg(args, 1, "target")?;
    Ok(annotate(expr_clone(target), "type", type_expr.clone()))
}

// #insight this function returns a string, it does not write to stdout.
// #todo also have a version that prints to stdout?
pub fn debug_expr(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [expr, ..] = args else {
        // #todo better error
        return Err(Error::invalid_arguments("invalid arguments", None));
    };

    let s = match expr {
        Expr::Annotated(expr, ann) => format!("ANN({expr:?}, {ann:?})"),
        _ => format!("{expr:?}"),
    };

    Ok(Expr::string(s))
}

pub fn type_of(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let [expr] = args else {
        // #todo better error
        return Err(Error::invalid_arguments("one argument expected", None));
    };

    Ok(expr.dyn_type(context))
}

pub fn is_a(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let typ = unpack_symbolic_arg(args, 0, "typ")?;
    let expr = unpack_arg(args, 1, "expr")?;

    let dyn_type = expr.dyn_type(context);
    let Some(dyn_typ) = dyn_type.as_type() else {
        // #todo add range here.
        return Err(Error::general("cannot compute dyn type"));
    };

    Ok(Expr::Bool(typ == dyn_typ))
}

// #todo implement read from Port, and provide variant where a Port is a File.
// #insight could replace with (eval (read file-contents))
// #todo maybe should be just eval_module?
// #todo consider naming just `load`.
pub fn load_file(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments("requires `path` argument", None));
    };

    let Some(path) = path.as_stringable() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a Stringable",
            path.range(),
        ));
    };

    // #todo canonicalize path, support relative paths?
    // #todo similar code in eval "use", refactor!

    // #think: do we need a nested scope? I think not! should be an option or multiple functions

    // #todo I _think_this causes the crazy 'templates/layout' bug.

    // let prev_scope = context.scope.clone();
    // context.scope = Rc::new(Scope::new(prev_scope.clone()));

    // #todo #hack temp solution here! somehow unify with eval_module.

    let prev_current_module_path = context.scope.get(CURRENT_MODULE_PATH);

    // #todo use context.insert_special!
    context.scope.insert(
        CURRENT_MODULE_PATH,
        Expr::string(
            PathBuf::from(&path)
                .parent()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        ),
    );

    let result = eval_file(path, context);

    if let Some(prev_current_module_path) = prev_current_module_path {
        context
            .scope
            .insert(CURRENT_MODULE_PATH, prev_current_module_path);
    }

    // context.scope = prev_scope;

    match result {
        Ok(value) => Ok(value),
        Err(errors) => {
            // #todo precise formating is _required_ here!
            // eprintln!("{}", format_errors(&errors));
            // #todo for some reason, errors are lost here.
            // #todo seems that (map ...) drinks the errors.
            dbg!(&errors);
            // #todo add note with information here!
            // #todo add custom error here, e.g. failed_file_load
            // println!("/////// {errors:?}");
            // println!("{}", std::backtrace::Backtrace::force_capture());
            Err(Error::failed_use(path, errors))
        }
    }
}

// #insight plain eval is already provided as a builtin/special form.

// #todo this is not really needed, can use `(eval (read string))` instead.
pub fn eval_string(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo support all `Stringable`s

    if let Some(input) = args.first() {
        let Some(input_str) = input.as_string() else {
            return Err(Error::invalid_arguments(
                "expected String argument",
                input.range(),
            ));
        };

        // #todo should create a throwaway context instead?

        // #todo think carefully which eval function to use.
        // let result = eval_string(input, &mut context);
        let result = compile_string(input_str, context);

        if let Ok(exprs) = result {
            // #todo what would be the correct initialization?
            let mut value = Expr::None;
            for expr in exprs {
                value = eval(&expr, context)?;
                // if let Err(mut error) = eval(&expr, context) {
                //     // #todo add a unit test to check that the file_path is added here!
                //     // #todo just make error.file_path optional and avoid this hack here!!!
                //     // if error.file_path == INPUT_PSEUDO_FILE_PATH {
                //     //     error.file_path = file_path.clone();
                //     // }

                //     // #todo better error here!
                //     return Err(error);
                // }
            }
            Ok(value)
        } else {
            // #todo something more clever needed here!
            // #todo use an aggregate Error, something like Error::failed_use()
            dbg!(&result);
            Err(Error::general("cannot read string, eval failed"))
        }
    } else {
        Err(Error::invalid_arguments("expected one argument", None))
    }
}

/// #todo Find a good name for this function.
fn curry(params: &[Expr], body: &[Expr]) -> Option<(Vec<Expr>, Vec<Expr>)> {
    params.first().map(|param| {
        let rest_params = &params[1..];

        let curried_body = if let Some((nested_params, nested_body)) = curry(rest_params, body) {
            vec![Expr::List(vec![
                Expr::Type("Func".to_owned()),
                Expr::array(nested_params),
                nested_body.first().unwrap().clone(), // #todo Remove unwrap!
            ])]
        } else {
            body.to_owned()
        };

        (vec![param.clone()], curried_body)
    })
}

// #todo Also support passing argument, implements partial application:
//
// (let greet (Func [msg name] "${msg} ${name}"))
// (let hello (curry greet "Hello"))
// (let hello "George") ; => "Hello George"
//
// Currently you can do:
// (let hello ((curry greet) "Hello"))
// (let hello "George") ; => "Hello George"
//
// #todo Convert to macro, evaluate at compile/static time.
// #todo #insight Cannot curry a function with zero parameters, just return the function unchanged!
// (let add1 (Func [x y] (+ x y 1)))
// (let curried-add1 (curry add1))
pub fn func_curry(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let Some(func) = args.first() else {
        return Err(Error::invalid_arguments("expected `func` argument", None));
    };

    let Expr::Func(params, body, func_scope, filename) = func.unpack() else {
        return Err(Error::invalid_arguments(
            "`func` argument should be a Func",
            None,
        ));
    };

    let (params, curried_body) = curry(params, body).unwrap_or((vec![], body.clone()));

    // #todo Also re-apply annotations.

    let curried_func = Expr::Func(params, curried_body, func_scope.clone(), filename.clone());
    Ok(curried_func)
}

// #todo consider link_foreign_dyn_lib (and unlink_...)
// #todo introduce unlink_foreign_dyn_lib for completeness.
// #todo find a better name, consider `use` or `load` instead of `install`.
pub fn link_foreign_dyn_lib(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let dyn_lib_path = unpack_stringable_arg(args, 0, "path")?;

    // #todo find a better name for canonicalize_module_path.
    // #todo use another function, not module_path specific, don't rewrite if it starts with "/"
    let dyn_lib_path = canonicalize_module_path(dyn_lib_path, context)?;

    // #todo canonicalize the path, resolve paths relative to CURRENT_MODULE_PATH

    // #todo add unit tests!

    unsafe {
        // #insight #WARNING
        // to make sure the (foreign) dynamic library is not dropped, move
        // it to a static hashmap. If the library gets dropped, calling a function
        // in the library would trigger a core dump.
        let foreign_dyn_lib_map = FOREIGN_DYN_LIB_MAP.get_or_init(|| Mutex::new(HashMap::new()));

        // #insight the MutexGuard is implicitly dropped.
        if foreign_dyn_lib_map
            .lock()
            // #todo abstract the handling of poisoned lock.
            .expect("poisoned lock")
            .contains_key(&dyn_lib_path)
        {
            // #todo consider not throwing an error, and just nop?
            // #todo consider just a warning (add support for warnings)
            // #todo more specific error variant needed.
            return Err(Error::general(&format!(
                "foreign dyn lib `{dyn_lib_path}` is already installed"
            )));
        }

        let library = match Library::new(&dyn_lib_path) {
            Ok(library) => library,
            Err(error) => {
                // #todo seems the error is not surfaced when used in a tab module, or something worse happens.
                // #todo more specific error variant needed.
                return Err(Error::general(&format!(
                    "cannot open foreign dyn lib `{dyn_lib_path}`: {error}"
                )));
            }
        };

        let link_foreign_dyn_lib =
            match library.get::<unsafe fn(&mut Context) -> i32>(b"install_foreign_dyn_lib\0") {
                Ok(link_foreign_dyn_lib) => link_foreign_dyn_lib,
                Err(error) => {
                    return Err(Error::general(&format!(
                        "cannot get link_foreign_dyn_lib for `{dyn_lib_path}`: {error}"
                    )));
                }
            };

        link_foreign_dyn_lib(context);

        // #insight
        // to make sure the (foreign) dynamic library is not dropped, move
        // it to a static hashmap.
        foreign_dyn_lib_map
            .lock()
            .expect("poisoned lock")
            .insert(dyn_lib_path, library);
    }

    Ok(Expr::None)
}

pub fn setup_lib_lang(context: &mut Context) {
    let module = require_module("prelude", context);

    // #todo separate read/read-string.

    module.insert("ann", Expr::ForeignFunc(Arc::new(ann)));
    module.insert("with-ann", Expr::ForeignFunc(Arc::new(with_ann)));
    module.insert("with-type", Expr::ForeignFunc(Arc::new(with_type)));

    // #todo the `!` is confusing here.
    // #todo `dbg` is not following naming conventions, but maybe OK for this case?
    // #insight this function returns a string, it does not write to stdout!
    module.insert("dbg!", Expr::ForeignFunc(Arc::new(debug_expr)));

    module.insert("type-of", Expr::ForeignFunc(Arc::new(type_of)));
    module.insert("is-a?", Expr::ForeignFunc(Arc::new(is_a)));

    // #todo we also need an (eval ...) function.

    // #todo hmm, needs a differerent name
    // #todo use `(eval (read string))` instead?
    module.insert("eval-string", Expr::ForeignFunc(Arc::new(eval_string)));
    module.insert(
        "eval-string$$String",
        Expr::ForeignFunc(Arc::new(eval_string)),
    );

    module.insert("load-file", Expr::ForeignFunc(Arc::new(load_file)));

    // #todo Move to a namespace, e.g. `/func`.
    module.insert("curry", Expr::ForeignFunc(Arc::new(func_curry)));

    module.insert(
        "link-foreign-dyn-lib",
        Expr::ForeignFunc(Arc::new(link_foreign_dyn_lib)),
    );
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn with_type_usage() {
        let mut context = Context::new();

        // #todo hmm, this (with-type ...) seems reverse.
        let input = r#"
        (def a (with-type First-Name "George"))
        (type-of a)
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "First-Name";
        assert_eq!(value, expected);
    }

    #[test]
    fn type_of_usage() {
        let mut context = Context::new();

        let input = r#"(type-of 1)"#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "Int";
        assert_eq!(value, expected);

        let input = r#"
            (let my-func (Func [x] (+ 1 x)))
            (type-of my-func)
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "Func";
        assert_eq!(value, expected);

        let input = r#"
            (use [random] /rng)
            (type-of random)
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "ForeignFunc";
        assert_eq!(value, expected);
    }
}
