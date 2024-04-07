use std::{path::PathBuf, sync::Arc};

use crate::{
    api::resolve_string,
    context::Context,
    error::Error,
    eval::{eval, util::eval_file},
    expr::{annotate, expr_clone, Expr},
    util::{module_util::require_module, standard_names::CURRENT_MODULE_PATH},
};

// #todo (if (= (get-type obj) Amount) ...) ; type, get-type, type-of
// #todo implement `type?` e.g. (if (type? obj Amount) ...)
// #todo op to set annotation.

// #todo consider meta instead of ann
// #todo consider get-ann?
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
        Ok(Expr::One)
    }
}

// #todo find better name.
// #todo support multiple annotations (pass map)
// (ann! expr :type Amount)
pub fn set_ann(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [target, key, value] = args else {
        // #todo better error
        return Err(Error::invalid_arguments("invalid arguments", None));
    };

    let Some(key) = key.as_stringable() else {
        // #todo better error.
        return Err(Error::invalid_arguments("invalid arguments", None));
    };

    // #todo remove the clones.
    Ok(annotate(expr_clone(target), key, expr_clone(value)))
}

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

// #insight with dynamic typing you don't strictly need a Maybe type?
pub fn is_none(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [expr] = args else {
        // #todo better error
        return Err(Error::invalid_arguments("one argument expected", None));
    };

    Ok(Expr::Bool(expr.is_one()))
}

pub fn is_some(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [expr] = args else {
        // #todo better error
        return Err(Error::invalid_arguments("one argument expected", None));
    };

    Ok(Expr::Bool(!expr.is_one()))
}

pub fn type_of(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let [expr] = args else {
        // #todo better error
        return Err(Error::invalid_arguments("one argument expected", None));
    };

    Ok(expr.dyn_type(context))
}

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
            // dbg!(errors);
            // #todo add note with information here!
            // #todo add custom error here, e.g. failed_file_load
            // println!("/////// {errors:?}");
            // println!("{}", std::backtrace::Backtrace::force_capture());
            Err(Error::failed_use(path, errors))
        }
    }
}

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
        let result = resolve_string(input_str, context);

        if let Ok(exprs) = result {
            // #todo what would be the correct initialization?
            let mut value = Expr::One;
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

pub fn setup_lib_lang(context: &mut Context) {
    let module = require_module("prelude", context);

    // #todo separate read/read-string.

    module.insert("ann", Expr::ForeignFunc(Arc::new(ann)));
    module.insert("ann!", Expr::ForeignFunc(Arc::new(set_ann)));

    // #todo the `!` is confusing here.
    module.insert("dbg!", Expr::ForeignFunc(Arc::new(debug_expr)));

    // #todo use is-some? to make more like a verb?
    // (if (some? user) ...)
    // (if (is-some? user) ...)
    // (if (is-some user) ...)
    module.insert("some?", Expr::ForeignFunc(Arc::new(is_some)));
    module.insert("none?", Expr::ForeignFunc(Arc::new(is_none)));

    module.insert("type-of", Expr::ForeignFunc(Arc::new(type_of)));

    module.insert("eval-string", Expr::ForeignFunc(Arc::new(eval_string)));
    module.insert(
        "eval-string$$String",
        Expr::ForeignFunc(Arc::new(eval_string)),
    );

    module.insert("load-file", Expr::ForeignFunc(Arc::new(load_file)));
}
