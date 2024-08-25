use crate::{
    context::Context,
    error::Error,
    expr::{annotate_range, expr_clone, Expr},
};

use super::util::{eval_module, get_bindings_with_prefix};

// #todo Allow use to be nested in scopes (e.g. Func, do, etc)

// #todo annotate imported values with range
// #todo add extra annotation that this is imported.
// #todo consider also inserting the module in scope?

pub fn eval_use(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #insight modules are (currently) directories, _not_ files.

    // #todo add unit tests for all cases.
    // #todo support single-file modules (xxx.mod.tan, xxx.module.tan)
    // #insight this code is temporarily(?) moved from `resolver` here.
    // #todo also introduce a dynamic version of `use`.

    // #todo properly handle this here, strip the use expression, remove from eval
    // #todo move this to resolve? use should be stripped at dyn-time
    // #todo also support path as symbol.

    // Import a directory as a module.

    // #todo find a better name than qualifier.
    let (module_path, qualifier) = match args.len() {
        1 => (args.first().unwrap(), None),
        2 => (args.get(1).unwrap(), args.first()),
        _ => {
            return Err(Error::invalid_arguments("malformed use expression", None));
        }
    };

    // #todo is it safe to call unwrap here?
    let use_range = module_path.range().unwrap();

    // #todo the formatter should convert string paths to symbols.

    // #insight support both Strings and Symbols as module paths.
    // #insight, notice the `try_string`.
    let Some(module_path) = module_path.as_stringable() else {
        // let Some(module_path) = term.as_string() else {
        return Err(Error::invalid_arguments("malformed use expression", None));
    };

    // #todo make sure paths are relative to the current file.
    let result = eval_module(module_path, context, false);
    if let Err(errors) = result {
        // #todo precise formating is _required_ here!
        // eprintln!("{}", format_errors(&errors));
        // dbg!(&errors);``
        // println!("~~~~~ {errors:?}");
        // #todo add note with information here!
        // #todo SOS! need to surface the errors here!
        return Err(Error::failed_use(module_path, errors));
    };

    let Ok(Expr::Module(module)) = result else {
        unreachable!("invalid module for `{}`", module_path);
    };

    // #insight
    // Follows the syntax of let, for, etc:
    // (use [pi tau] math) ; pi
    // (use m math) ; m/pi

    // use the module stem as the deafault prefix.
    let mut module_prefix = module.stem.as_str();

    if let Some(arg) = qualifier {
        // there is qualifier, only import the selected
        // names.
        if let Some(names) = arg.as_array() {
            // (use [pi tau] math) ; pi, embed without namespace.
            for name in names.iter() {
                // #todo ONLY export public bindings
                // #todo assign as top-level bindings!
                let Some(name) = name.as_stringable() else {
                    return Err(Error::invalid_arguments(
                        "use explicit imports should be Stringables",
                        None,
                    ));
                };

                let bindings = get_bindings_with_prefix(&module.scope, name);

                if bindings.is_empty() {
                    // #todo make this a FailedUse error.
                    return Err(Error::invalid_arguments(
                        &format!("undefined import `{name}`"),
                        None,
                    ));
                }

                for binding in bindings {
                    // eprintln!("======> {} = {}", binding.0, binding.1);
                    // let value = update_annotation(
                    //     &mut Arc::get_mut(binding.1).unwrap(),
                    //     "range",
                    //     _use_range,
                    // );
                    // #todo should move to comp-time.
                    // #insight it's relatively OK to call expr_clone here (use).
                    let value = annotate_range(expr_clone(&binding.1), use_range.clone());
                    context.scope.insert(binding.0, value);
                }

                // let Some(value) = module.scope.get(name) else {
                //     return Err(Error::invalid_arguments(
                //         &format!("undefined import `{name}`"),
                //         expr.range(),
                //     ));
                // };
                // context.scope.insert(name, value.clone());
            }

            // #todo again consider returning the module here.
            return Ok(Expr::None);
        } else if let Some(prefix) = arg.as_symbol() {
            module_prefix = prefix;
        } else {
            return Err(Error::invalid_arguments(
                "malformed use expression, bad qualifier",
                None,
            ));
        }
    }

    // Import public names from module scope into the current scope.

    // #todo support (use "/path/to/module" *) or (use "/path/to/module" :embed)

    // #todo temp, needs cleanup!
    let bindings = module.scope.bindings.read().expect("poisoned lock");
    for (name, value) in bindings.iter() {
        // #todo temp fix to not override the special var
        if name.starts_with('*') {
            continue;
        }

        // #todo ONLY export public bindings

        let name = format!("{}/{}", module_prefix, name);

        // #todo add a unit test to verify range annotations.
        // #todo assign as top-level bindings!

        let value = annotate_range(expr_clone(value), use_range.clone());
        context.scope.insert(name, value);
    }

    // #todo allow for embedding explicit symbols, non-namespaced!

    // #todo what could we return here? the Expr::Module?
    // Ok(Expr::Module(module))
    Ok(Expr::None)
}
