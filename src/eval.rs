#![allow(clippy::manual_strip)]

// #todo Move these external eval functions into library, e.g. library/lang?

mod eval_assertions;
mod eval_assign;
mod eval_cond;
mod eval_def;
mod eval_do;
mod eval_else;
mod eval_for;
mod eval_for_each;
mod eval_for_list;
mod eval_if;
mod eval_is_defined;
pub mod eval_let;
mod eval_let_ds;
mod eval_panic;
mod eval_pipe;
mod eval_scope_update;
mod eval_unless;
mod eval_use;
mod eval_when;
mod eval_while;
pub mod iterator;
pub mod util;

use std::{collections::HashMap, sync::Arc};

use eval_else::eval_else;
use eval_is_defined::eval_is_defined;
use eval_pipe::eval_pipe;
use eval_unless::eval_unless;
use eval_when::eval_when;

use crate::{
    context::Context,
    error::{Error, ErrorVariant},
    expr::{expr_clone, format_value, Expr, ForeignFnRef},
    range::Range,
    resolver::resolve_op_method,
    scope::Scope,
    util::{
        is_dynamically_scoped, is_ellipsis, is_reserved_symbol,
        method::compute_signature_from_annotations, standard_names::CURRENT_FILE_PATH,
        try_lock_read,
    },
};

use self::{
    eval_assertions::{eval_assert, eval_assert_eq},
    eval_assign::eval_assign,
    eval_cond::eval_cond,
    eval_def::eval_def,
    eval_do::eval_do,
    eval_for::eval_for,
    eval_for_each::eval_for_each,
    eval_for_list::eval_for_list,
    eval_if::eval_if,
    eval_let::eval_let,
    eval_let_ds::eval_let_ds,
    eval_panic::eval_panic,
    eval_scope_update::eval_scope_update,
    eval_use::eval_use,
    eval_while::eval_while,
    util::{anchor_error, get_current_file_path},
};

// #insight Not a pure evaluator, performs side-effects.

// #insight I don't like the name `interpreter`.

// #todo Move excessive error-checking/linting to the resolve/typecheck pass.
// #todo Encode effects in the type-system.
// #todo Alternative names: Processor, Runner, Interpreter
// #todo Split eval_special, eval_func -> not needed if we put everything uniformly in prelude.
// #todo Stack-trace is needed!
// #todo https://clojure.org/reference/evaluation

// #todo try to remove non-needed .into()s <--

// #todo Give more 'general' name -> `eval_all` or `eval_vec`?
// #todo What about if a required argument is not passed to a function? currently we report undefined symbol.
pub fn eval_args(args: &[Expr], context: &mut Context) -> Result<Vec<Expr>, Error> {
    // #todo should report ALL errors!

    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        values.push(eval(arg, context)?);
    }
    Ok(values)
}

// #todo Add unit test.
fn insert_symbol_binding(
    sym: &str,
    range: &Option<Range>,
    value: Expr,
    context: &mut Context,
) -> Result<(), Error> {
    // #insight reserved words are not polymorphic, so we can check before signature, rething about this.
    // #todo Also is_reserved_symbol is slow, optimize.
    // #todo Do we really want this? Maybe convert to a lint?
    if is_reserved_symbol(sym) {
        return Err(Error::invalid_arguments(
            &format!("cannot shadow the reserved symbol `{sym}`"),
            range.clone(),
        ));
    }

    // #todo Move this even more up-stream.
    // #todo Move this up-stream to insert_binding.
    // #todo This is a temp hack!
    let sym = if value.is_invocable() {
        if let Some(signature) = compute_signature_from_annotations(&value) {
            // Make sure the symbol without a signature exists.
            // #todo #hack This is a temp fix until we properly implement multi-methods/overloaded ops.
            // #insight We cannot avoid this recursive contains check, and we cannot just force the insertion of a 'sentinel' value.
            if !context.scope.contains_name_recursive(sym) {
                // #todo #optimize Could just put a dummy value here? e.g. Expr::Never, probably not?
                context.scope.insert(sym, expr_clone(&value));
            }

            format!("{sym}{signature}")
        } else {
            // #todo Should  insert a more specialized symbol.
            // format!("{sym}$$*")
            sym.to_owned()
        }
    } else {
        sym.to_owned()
    };

    // #todo notify about overrides? use `set`?
    context.scope.insert(sym, value);

    Ok(())
}

// #todo find a better name.
fn insert_binding(name: &Expr, value: Expr, context: &mut Context) -> Result<(), Error> {
    // #todo Consider special op/syntax for destructuring? e.g. ~[a b], `~` operator.

    // #todo Handle potential relevant annotations.

    match name.unpack() {
        // #todo Type/Symbol duplication needs to be resolved, separate Types from Symbols.
        Expr::Type(sym) => {
            // #todo report error if sym == _ or ...
            insert_symbol_binding(sym, &name.range(), value, context)?;
        }
        Expr::Symbol(sym) => {
            // #todo Report error if sym == _ or ...
            insert_symbol_binding(sym, &name.range(), value, context)?;
        }
        Expr::List(names) => {
            // list destructuring.
            if names.len() != 2 {
                return Err(Error::invalid_arguments(
                    "malformed List destructuring, needs two names",
                    name.range(),
                ));
            }

            let Some(head_name) = names[0].as_symbol() else {
                return Err(Error::invalid_arguments(
                    "malformed List destructuring bind, pattern should contain head symbol",
                    name.range(),
                ));
            };

            let Some(tail_name) = names[1].as_symbol() else {
                return Err(Error::invalid_arguments(
                    "malformed List destructuring bind, pattern should contain tail symbol",
                    name.range(),
                ));
            };

            if !tail_name.starts_with("...") {
                return Err(Error::invalid_arguments(
                    "malformed List destructuring bind, tail symbol should start with ellisis",
                    name.range(),
                ));
            }

            let Some(values) = value.as_list() else {
                // #todo better error message.
                // #todo annotate the value.
                // #todo add multiple notes to the error.
                return Err(Error::invalid_arguments(
                    "malformed List destructuring bind, the value should be a List",
                    value.range(),
                ));
            };

            // #insight unwrap is safe here after the previous checks.
            let (head, tail) = values.split_first().unwrap();

            // #todo omg expr_clone() and to_vec() are expensive!
            insert_symbol_binding(head_name, &names[0].range(), expr_clone(head), context)?;
            insert_symbol_binding(
                &tail_name[3..],
                &names[0].range(),
                Expr::List(tail.to_vec()),
                context,
            )?;

            // #todo add unit tests.
        }
        Expr::Array(names) => {
            // array destructuring.
            // #todo temp, nasty code.
            // ensure that the values is also an Array.
            let Some(values) = value.as_array() else {
                // #todo better error message.
                // #todo annotate the value.
                // #todo add multiple notes to the error.
                return Err(Error::invalid_arguments(
                    "malformed destructuring bind, the value should be an Array",
                    value.range(),
                ));
            };

            let names = try_lock_read(names, name.range())?;

            // #todo check if the item count matches, report mismatches.
            for (i, name) in names.iter().enumerate() {
                let Some(sym) = name.as_symbol() else {
                    return Err(Error::invalid_arguments(
                        "malformed destructuring bind, array pattern should contain symbols",
                        name.range(),
                    ));
                };
                if sym == "_" {
                    continue;
                }

                // #insight '...' is called `ellipsis`.

                // #todo consider `..._` for ignoring?
                if sym == "..." {
                    break;
                }

                if sym.starts_with("...") {
                    insert_symbol_binding(
                        &sym[3..],
                        &names[i].range(),
                        Expr::array(&values[i..]),
                        context,
                    )?;
                } else {
                    insert_symbol_binding(
                        sym,
                        &name.range(),
                        expr_clone(values.get(i).unwrap()),
                        context,
                    )?;
                }
            }
        }
        Expr::Map(items) => {
            // map destructuring.
            // (let {:name name :age age} user)
            // (let {:name _ :age _} user)

            // #todo temp, nasty code.
            // #todo Add example usage!
            // #todo Add unit tests!
            // ensure that the values are also a Map.
            let Some(values) = value.as_map() else {
                // #todo better error message.
                // #todo annotate the value.
                // #todo add multiple notes to the error.
                return Err(Error::invalid_arguments(
                    "malformed destructuring bind, the value should be a Map",
                    name.range(),
                ));
            };
            // #todo check if the item count matches, report mismatches.

            let items = try_lock_read(items, name.range())?;

            for (key, name) in items.iter() {
                // #todo could use as_stringable here.
                // #todo could use the real expression here.
                let Some(sym) = name.as_symbol() else {
                    return Err(Error::invalid_arguments(
                        "malformed destructuring bind, map pattern should contain symbols",
                        name.range(),
                    ));
                };

                // (let {:name _ :age _} user)
                let sym = if sym == "_" { key } else { sym };

                if !values.contains_key(key) {
                    return Err(Error::invalid_arguments(
                        "malformed destructuring bind, map does not contain key",
                        name.range(),
                    ));
                }

                // #insight no need for ellipsis/rest here!
                insert_symbol_binding(
                    sym,
                    &name.range(),
                    values.get(key).unwrap().clone(),
                    context,
                )?;
            }
        }
        _ => {
            return Err(Error::invalid_arguments(
                &format!("malformed binding: `${name}`"),
                name.range(),
            ));
        }
    }

    Ok(())
}

// #todo pass &[Expr] instead of Vec<Expr>
pub fn invoke(invocable: &Expr, args: Vec<Expr>, context: &mut Context) -> Result<Expr, Error> {
    // #todo Support more invocable expressions, e.g. indexing!
    let result = match invocable.unpack() {
        Expr::Func(..) => invoke_func(invocable, args, context),
        Expr::ForeignFunc(fn_ref) => {
            // #todo Consider having 3 ForeignFunc variants to avoid an extra check?
            match fn_ref {
                ForeignFnRef::NoContext(func) => func(&args),
                ForeignFnRef::Context(func) => func(&args, context),
                ForeignFnRef::MutContext(func) => func(&args, context),
            }
            // foreign_function(&args, context)
        }
        _ => {
            // #todo return NonInvocable error!
            Err(Error::invalid_arguments(
                &format!("not invocable: `{invocable}`"),
                invocable.range(),
            ))
        }
    };

    match result {
        Err(ref error) => {
            if let ErrorVariant::Panic(msg) = &error.variant {
                // #todo #hack this is a temp solution, maybe the Repl/Runner should install a custom panic handler?
                // #todo maybe put a flag in Context to stop further evaluation and unwind the stack?
                panic!("{}", msg);
            }
            result
        }
        _ => result,
    }
}

// #todo rename to eval_func?
// #todo use this function in eval, later.
// #todo pass &[Expr] instead of Vec<Expr>
// #todo rethink this and the non-inner function above.
pub fn invoke_func(func: &Expr, args: Vec<Expr>, context: &mut Context) -> Result<Expr, Error> {
    // #insight args are intentionally not evaluated!

    let Expr::Func(params, body, func_scope, file_path) = func.unpack() else {
        // #todo what to do here?
        return Err(Error::invalid_arguments("should be a Func", func.range()));
    };

    // #todo should set the current-module somehow?

    // #todo ultra-hack to kill shared ref to `env`.
    let params = params.clone();

    // #insight
    // actually we implement static (lexical) scoping here, as we base the new
    // scope on the lexical function scope.

    // #todo avoid the func_scope.clone()
    let prev_scope = context.scope.clone();
    // #insight notice we use func_scope here!
    // #insight #IMPORTANT make sure the scope is restored before all exit points of this function!!!
    // #todo need a push_scope helper on context that uses Drop to emulate defer?
    // #todo e.g. it could return a prev_scope ScopeGuard!
    // #insight notice we use func_scope here!
    // let prev_scope = std::mem::replace(&mut context.scope, func_scope.clone());
    context.scope = Arc::new(Scope::new(func_scope.clone()));
    // #todo consider args.into_iter();

    let mut args = args.into_iter();

    for param in params {
        let Some(param_name) = param.as_symbol() else {
            let mut error = Error::invalid_arguments("parameter is not a symbol", param.range());
            if !error.has_file_path() {
                error.file_path.clone_from(file_path);
            }
            // #insight Restore the scope (and current_file_path) after creating the error.
            context.scope = prev_scope;
            return Err(error);
        };

        // #todo consider other syntax, e.g. `&rest` like Clojure.

        // check for 'rest' parameter.
        if is_ellipsis(param_name) {
            let rest_args = Expr::array(args.collect::<Vec<Expr>>());
            // remove the ellipsis prefix from the parameter name.
            let param_name = &param_name[3..];
            context.scope.insert(param_name, rest_args);
            break;
        }

        // #todo consider making missing parameters an error!
        // #todo or maybe just a warning?
        // let Some(arg) = args.next() else {
        // #todo correctly restore the scope!
        //     return Err(Error::invalid_arguments(
        //         &format!("no argument for parameter `{param}`"),
        //         param.range(),
        //     ));
        // };

        if let Some(arg) = args.next() {
            context.scope.insert(param_name, arg);
        } else {
            break;
        }
    }

    // #todo this code is the same as in the (do ..) block, extract.

    // #todo do should be 'monadic', propagate Eff (effect) wrapper.
    let mut value = Expr::None;

    for expr in body {
        // #todo what happens on `return` statement! should exit this loop and not evaluate the rest!
        // #todo should inspect the error and add the file_path?

        match eval(expr, context) {
            Ok(v) => value = v,
            Err(mut error) => {
                match error.variant {
                    ErrorVariant::ReturnCF(v) => {
                        // A return 'statement' encountered, stop evaluating more
                        // expressions and return the value.
                        value = v;
                        break;
                    }
                    _ => {
                        // #todo find better ways for reporting the file, this is a temp solution.
                        // annotate errors thrown by function evaluation with the
                        // function file_path, for more precise error reporting.
                        // #todo Why is this needed here?
                        if !error.has_file_path() {
                            error.file_path.clone_from(file_path);
                        }
                        context.scope = prev_scope;
                        return Err(error);
                    }
                }
            }
        }
    }

    // #todo what happens to this if an error is thrown??!!
    context.scope = prev_scope;
    Ok(value)
}

// #insight Having granular eval functions allows optimization by calling the directly when we have context.
// #note The passed expression should be unpacked!
pub fn eval_symbol(expr: &Expr, context: &mut Context) -> Result<Expr, Error> {
    // #todo Try to avoid this unpack!
    let Expr::Symbol(symbol) = expr.unpack() else {
        unreachable!()
    };

    // #todo differentiate between evaluating symbol in 'op' position.
    // #todo combine/optimize check for reserved_symbol and dynamically_scoped, move as much as possible to static-time resolver.
    if is_reserved_symbol(symbol) {
        return Ok(expr.clone());
    }

    // #todo #IMPORTANT now that we don't use the method annotation, we don't support Tan functions with types?

    // #todo handle 'PathSymbol'
    // #todo #IMPORTANT this is currently handled in the parser, here is a better place I think.
    // #todo try to populate "method"/"signature" annotations during resolving
    // #todo this is missing now that we don't have the resolve stage.
    // #todo maybe resolve or optimize should already have placed the method in the AST?

    let value = context
        .get(symbol, is_dynamically_scoped(symbol))
        .ok_or_else::<Error, _>(|| {
            let mut error = Error::undefined_symbol(
                symbol,
                &format!("symbol not defined: `{symbol}`"),
                expr.range(),
            );

            // Custom hints for common errors.

            // #todo Add unit test for ',', '`' hints.
            if symbol.contains(',') {
                error.push_note("you added a comma by mistake?", None)
            }

            if symbol.contains('`') {
                // #todo Better hint needed.
                // #todo Mark as hint, not a general note?
                error.push_note("you used ` instead of ' by mistake?", None)
            }

            error
        })?;

    // #todo hm, can we somehow work with references?
    // #hint this could help: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.unwrap_or_clone

    Ok(expr_clone(&value))
}

#[inline]
pub fn eval_key_symbol(expr: &Expr) -> Result<Expr, Error> {
    // #todo Handle 'PathSymbol', nah how can it be handled here? it can't.
    // #todo strip annotation?

    // #todo Lint '::' etc.
    // #todo Check that if there is a leading ':' there is only one ':', make this a lint warning!

    // #todo #IMPORTANT Avoid the clone here.

    // A `Symbol` that starts with `:` is a so-called `KeySymbol`. Key
    // symbols evaluate to themselves, and are convenient to use as Map keys,
    // named (keyed) function parameter, enum variants, etc.
    Ok(expr.clone())
}

#[inline]
pub fn eval_type(expr: &Expr) -> Result<Expr, Error> {
    // #todo Remove this clone.
    Ok(expr.clone())
}

// #todo needs better conversion to Expr::Annotated

/// Evaluates via expression rewriting. The expression `expr` evaluates to
/// a fixed point. In essence this is a 'tree-walk' interpreter.
pub fn eval(expr: &Expr, context: &mut Context) -> Result<Expr, Error> {
    let result = match expr.unpack() {
        // #todo are you sure?
        // Expr::Annotated(..) => eval(expr.unpack(), env),
        // #todo should pass `symbol_expr` to eval_symbol.
        _symbol_expr @ Expr::Symbol(..) => eval_symbol(expr, context),
        Expr::KeySymbol(..) => eval_key_symbol(expr),
        Expr::Type(..) => eval_type(expr),
        // #todo if is unquotable!!
        // #todo is this ever actually used at the moment?
        Expr::If(predicate, true_clause, false_clause) => {
            let predicate = eval(predicate, context)?;

            let Some(predicate) = predicate.as_bool() else {
                return Err(Error::invalid_arguments(
                    "the if predicate is not a boolean value",
                    predicate.range(),
                ));
            };

            if predicate {
                eval(true_clause, context)
            } else if let Some(false_clause) = false_clause {
                eval(false_clause, context)
            } else {
                // #todo what should we return if there is no false-clause? Zero/Never?
                Ok(Expr::None)
            }
        }
        // #insight Operator invocation.
        Expr::List(list) => {
            // #todo No need for dynamic invocable, can use (apply f ...) / (invoke f ...) instead.
            // #todo Replace head/tail with first/rest

            if list.is_empty() {
                // () == None (== One, Unit)
                // This is handled statically, in the parser, but an extra, dynamic
                // check is needed in the evaluator to handle the case where the
                // expression is constructed programmatically (e.g. self-modifying code,
                // dynamically constructed expression, homoiconicity, etc).
                return Ok(Expr::None);
            }

            // The unwrap here is safe.
            let op = list.first().unwrap();
            // let args = &list[1..];
            // #todo Try to avoid Vec::from()
            let mut args = Vec::from(&list[1..]);

            // #todo Could check special forms before the eval

            // #todo This is an ULTRA-HACK! SUPER NASTY/UGLY CODE, refactor!

            // Resolve and evaluate the head, try to find dynamic signature.

            let head = if let Some(name) = op.as_symbolic() {
                if !is_reserved_symbol(name) {
                    // #todo super nasty hack!!!!

                    // #insight The un-mangled operator name is still needed.
                    // #todo Add module.insert_op helper that automatically creates it!
                    // #todo we don't support dynamic scoping in this position, reconsider
                    if let Some(value) = context.scope.get(name) {
                        match value.unpack() {
                            Expr::Func(params, ..) => {
                                // 'Cache' the evaluated args, to avoid double evaluation.
                                args = eval_args(&args, context)?;

                                // #todo Extract utility function to invoke a function.
                                // #todo Ultra-hack to kill shared ref to `env`.
                                let params = params.clone();

                                let prev_scope = context.scope.clone();
                                context.scope = Arc::new(Scope::new(prev_scope.clone()));

                                for (param, arg) in params.iter().zip(&args) {
                                    let Some(param) = param.as_symbol() else {
                                        return Err(Error::invalid_arguments(
                                            "parameter is not a symbol",
                                            param.range(),
                                        ));
                                    };

                                    context.scope.insert(param, arg.clone());
                                }

                                // #todo Optimize the resolve_op_method.
                                let head = resolve_op_method(op, name, &args, context)?;

                                context.scope = prev_scope;

                                head
                            }
                            Expr::ForeignFunc(_) => {
                                args = eval_args(&args, context)?;
                                // #todo Optimize the resolve_op_method.
                                resolve_op_method(op, name, &args, context)?
                            }
                            _ => {
                                // #insight The lookup yields other invocables,
                                // e.g. Map, Array (Indexable), Type, etc.
                                args = eval_args(&args, context)?;
                                eval(op, context)?
                            }
                        }
                    } else {
                        // #todo What is this case?
                        // #insight No need to eval_args here!
                        eval(op, context)?
                    }
                } else {
                    // The operator is a reserved symbol.
                    // #todo !?!?
                    eval(op, context)?
                }
            } else {
                // The operator is not a symbol, it's 'derreferenced'?
                eval(op, context)?
            };

            // Evaluate the whole list expression with the resolved head/op.

            // #todo Use op instead of head here.
            // #todo Move special forms to prelude, as Expr::Macro or Expr::Special

            match head.unpack() {
                Expr::Func(..) => {
                    // #todo #fails library::html::tests::html_from_expr_usage
                    // #insight The args are already evaluated here!
                    // #todo call invoke_func directly?
                    anchor_error(invoke(&head, args, context), expr)
                }
                Expr::ForeignFunc(..) => {
                    // #todo do NOT pre-evaluate args for ForeignFunc, allow to implement 'macros'.
                    // Foreign Functions do NOT change the environment, hmm...
                    // #todo use RefCell / interior mutability instead, to allow for changing the environment (with Mutation Effect)

                    // #todo call directly?
                    // #insight The args are already evaluated here!
                    anchor_error(invoke(&head, args, context), expr)
                }
                Expr::Array(arr) => {
                    // #todo What about dynamic type here?
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(&args, context)?;

                    // #todo optimize this!
                    // #todo error checking, one arg, etc.
                    let index = &args[0];
                    // #todo we need UInt, USize, Nat type
                    let Some(index) = index.as_int() else {
                        return Err(Error::invalid_arguments(
                            "invalid array index, expecting Int",
                            index.range(),
                        ));
                    };
                    let index = index as usize;

                    let arr = try_lock_read(arr, expr.range())?;

                    if let Some(value) = arr.get(index) {
                        // #todo replace the clone with the custom expr::copy/ref
                        Ok(value.clone())
                    } else {
                        // #todo introduce Maybe { Some, None }
                        Ok(Expr::None)
                    }
                }
                Expr::Map(map) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(&args, context)?;

                    // #todo optimize this!
                    // #todo error checking, one arg, stringable, etc.

                    // #insight no need to unpack, format_value sees-through.
                    let key = format_value(&args[0]);

                    let map = try_lock_read(map, expr.range())?;

                    if let Some(value) = map.get(&key) {
                        Ok(value.clone())
                    } else {
                        // #todo Introduce Maybe { Some, None }
                        Ok(Expr::None)
                    }
                }
                // #todo move all 'type-constructors' to external files.
                Expr::Type(s) => match s.as_str() {
                    "List" => {
                        let args = eval_args(&args, context)?;
                        Ok(Expr::List(args))
                    }
                    "Trait" => {
                        // Example:
                        // (let Semigroup (Trait T
                        //   #(Func [T T] T) combine
                        // ))
                        //
                        // #insight Just accepts the syntax for the moment.
                        // #todo Implement me!
                        // #todo We need a Trait Expr?
                        Ok(Expr::None)
                    }
                    "Func" => {
                        let Some(params) = args.first() else {
                            return Err(Error::invalid_arguments(
                                "malformed func definition, missing function parameters",
                                expr.range(),
                            ));
                        };

                        let body = &args[1..];

                        // #todo move handling of Expr::None to as_list?

                        // #todo Remove the clones!
                        // #todo should check both for list and array (i.e. as_iterable)
                        let params = if let Some(params) = params.as_array() {
                            params.clone()
                        } else if params.is_symbol() {
                            // Also allow a single parameter without the array.
                            // #todo For the moment we just convert it to an array here, we should leave it as is though.
                            vec![params.clone()]
                        } else if params.is_none() {
                            // #insight None == One == Unit
                            Vec::new()
                        } else {
                            println!("=== {params:?}");
                            return Err(Error::invalid_arguments(
                                "malformed func parameters definition",
                                params.range(),
                            ));
                        };

                        // #insight captures the static (lexical scope)

                        let func_file_path = get_current_file_path(context);
                        let func_scope = context.scope.clone();
                        func_scope.insert(CURRENT_FILE_PATH, Expr::string(&func_file_path));

                        // #todo optimize
                        Ok(Expr::Func(
                            params,
                            body.into(),
                            func_scope,
                            func_file_path, // #todo is this really needed here?
                        ))
                    }
                    // #todo lookup constructor function
                    _ => Err(Error::not_invocable(
                        &format!("not invocable constructor `{head}`, the type is `{s}`"),
                        head.range(),
                    )),
                },
                // #todo add handling of 'high-level', compound expressions here.
                // #todo Expr::If
                // #todo Expr::Let
                // #todo Expr::Do
                // #todo Expr::..
                Expr::Symbol(s) => {
                    match s.as_str() {
                        "eval" => {
                            // #todo also support eval-all/eval-many? (auto wrap with do?)
                            let Some(expr) = args.first() else {
                                return Err(Error::invalid_arguments(
                                    "missing expression to be evaluated",
                                    expr.range(),
                                ));
                            };

                            // #todo consider naming this `form`?
                            let expr = eval(expr, context)?;

                            eval(&expr, context)
                        }
                        "return" => {
                            let value = args.first().unwrap_or(&Expr::None);
                            let value = eval(value, context)?;
                            Err(Error::return_cf(value))
                        }
                        // #todo is there a way to avoid having continue in the language?
                        // #todo consider a different name?
                        // #todo consider continue without parentheses?
                        // #todo maybe should return some kind of Nothing/Never/Zero value?
                        "continue" => Err(Error::continue_cf()),
                        // #todo is there a way to avoid having break in the language?
                        // #todo consider break without parentheses?
                        // #todo maybe should return some kind of Nothing/Never/Zero value?
                        "break" => {
                            let value = args.first().unwrap_or(&Expr::None);
                            let value = eval(value, context)?;
                            Err(Error::break_cf(value))
                        }
                        "quot" => {
                            // #insight not obvious how to move to static/comptime phase.
                            // #todo doesn't quote all exprs, e.g. the if expression.
                            // #todo optimize with custom exprs, e.g Expr::Quot, Expr::QuasiQuot, etc.

                            let Some(value) = args.first() else {
                                return Err(Error::invalid_arguments(
                                    "missing quote target",
                                    expr.range(),
                                ));
                            };

                            // #todo transform_mut is not the correct traversal, it's depth first it should be breadth first.
                            // #todo expr.quote() is a temp hack.
                            Ok(value.clone().quot(context))
                        }
                        // special term
                        // #todo the low-level handling of special forms should use the above high-level cases.
                        // #todo use the `optimize`/`raise` function, here to prepare high-level expression for evaluation, to avoid duplication.
                        "do" => anchor_error(eval_do(&args, context), expr),
                        // #insight `head` seems to have range info, that `expr` lacks.
                        // #todo add range info to expr (no unpack) and use it instead!!!
                        "panic!" => anchor_error(eval_panic(&args, context), &head),
                        "for" => anchor_error(eval_for(&args, context), expr),
                        // #todo consider the name `for*` or something similar?
                        "for->list" => anchor_error(eval_for_list(&args, context), expr),
                        "while" => anchor_error(eval_while(&args, context), expr),
                        "if" => anchor_error(eval_if(&args, context), expr),
                        // #todo #temp Implement with macro.
                        "unless" => anchor_error(eval_unless(&args, context), expr),
                        // #todo #fix else has no range here, wtf!
                        "else" => anchor_error(eval_else(&args, context), expr),
                        "cond" => anchor_error(eval_cond(&args, context), expr),
                        "when" => anchor_error(eval_when(&args, context), expr),
                        "|>" => anchor_error(eval_pipe(&args, context), expr),
                        // #todo #temp temporary solution.
                        "assert" => anchor_error(eval_assert(op, &args, context), expr),
                        "assert-eq" => anchor_error(eval_assert_eq(op, &args, context), expr),
                        "is-defined?" => anchor_error(eval_is_defined(&args, context), expr),
                        // #todo for-each or overload for?
                        "for-each" => anchor_error(eval_for_each(&args, context), expr),
                        "assign" => anchor_error(eval_assign(&args, context), expr),
                        // #insight operator alias for assign
                        "<-" => anchor_error(eval_assign(&args, context), expr),
                        // #todo, investigate, find a better name.
                        "scope-update" => anchor_error(eval_scope_update(&args, context), expr),
                        // #insight `op` seems to have range info, that `expr` lacks.
                        // #todo add range info to expr (no unpack) and use it instead!!!
                        "use" => anchor_error(eval_use(&args, context), expr),
                        "def" => anchor_error(eval_def(&head, &args, context), expr),
                        "let-ds" => anchor_error(eval_let_ds(&args, context), expr),
                        "let" => anchor_error(eval_let(&head, &args, context), expr),
                        "and" => {
                            // #insight `and` _is_ short-circuiting and cannot be implemented with a function
                            // #todo what about binary and?
                            // #todo consider operator form? `&&` or `*`
                            // #todo optimize case with 2 arguments.
                            // #todo make a macro
                            // #todo should these 'special forms' get added in scope/env?

                            for arg in args {
                                let value = eval(&arg, context)?;
                                let Some(predicate) = value.as_bool() else {
                                    return Err(Error::invalid_arguments(
                                        "`and` argument should be boolean",
                                        expr.range(),
                                    ));
                                };

                                if !predicate {
                                    return Ok(Expr::Bool(false));
                                }
                            }

                            Ok(Expr::Bool(true))
                        }
                        "or" => {
                            // #insight `or` is short-circuiting so it cannot be implemented as a function
                            // #todo what about binary or?
                            // #todo consider operator form? `||` or `+`
                            // #todo make a macro.

                            for arg in args {
                                let value = eval(&arg, context)?;
                                let Some(predicate) = value.as_bool() else {
                                    return Err(Error::invalid_arguments(
                                        "`or` argument should be boolean",
                                        expr.range(),
                                    ));
                                };

                                if predicate {
                                    return Ok(Expr::Bool(true));
                                }
                            }

                            Ok(Expr::Bool(false))
                        }
                        _ => Err(Error::not_invocable(
                            &format!("symbol `{head}`"),
                            head.range(),
                        )),
                    }
                }
                _ => Err(Error::not_invocable(
                    // #todo add a more descriptive error!
                    &format!("expression `{head}`"),
                    head.range(),
                )),
            }
        }
        Expr::Array(items) => {
            // #insight [...] => (Array ...) => it's like a function.
            // #todo can this get pre-evaluated statically in some cases?
            let mut evaled_items = Vec::new();
            let items = try_lock_read(items, expr.range())?;
            for item in items.iter() {
                evaled_items.push(eval(item, context)?);
            }
            Ok(Expr::array(evaled_items))
        }
        Expr::Map(map) => {
            // #insight evaluates the values.
            // #insight [...] => (Map ...) => it's like a function.
            // #todo nasty code, improve.
            // #todo can this get pre-evaluated statically in some cases?
            let mut evaled_map = HashMap::new();
            // #todo use pairs or items instead of map?
            let map = try_lock_read(map, expr.range())?;
            for (k, v) in map.iter() {
                evaled_map.insert(k.clone(), eval(v, context)?);
            }
            Ok(Expr::map(evaled_map))
        }
        _ => {
            // #todo hm, maybe need to report an error here? or even select the desired behavior? -> NO ERROR
            // #todo can we avoid the clone?
            // Unhandled expression variants evaluate to themselves.
            Ok(expr.clone())
        }
    };
    // #hint keep this for debugging.
    // if let Err(ref error) = result {
    //     match error.variant {
    //         // #ignore 'pseudo' errors (control-flow)
    //         ErrorVariant::ContinueCF | ErrorVariant::BreakCF(..) | ErrorVariant::ReturnCF(..) => (),
    //         _ => {
    //             println!("-----> {error:?}");
    //             // println!("{}", std::backtrace::Backtrace::force_capture());
    //         }
    //     }
    // }
    result
}
