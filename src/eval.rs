mod eval_assertions;
mod eval_do;
mod eval_for;
mod eval_if;
mod eval_let;
mod eval_let_ds;
mod eval_panic;
mod eval_use;
pub mod iterator;
pub mod util;

use std::{collections::HashMap, sync::Arc};

use crate::{
    context::Context,
    error::{Error, ErrorVariant},
    expr::{annotate, expr_clone, format_value, Expr},
    range::Range,
    resolver::compute_dyn_signature,
    scope::Scope,
    util::{is_dynamically_scoped, is_ellipsis, is_reserved_symbol, try_lock_read},
};

use self::{
    eval_assertions::{eval_assert, eval_assert_eq},
    eval_do::eval_do,
    eval_for::eval_for,
    eval_if::eval_if,
    eval_let::eval_let,
    eval_let_ds::eval_let_ds,
    eval_panic::eval_panic,
    eval_use::eval_use,
    iterator::try_iterator_from,
    util::{anchor, get_current_file_path},
};

// #insight
// _Not_ a pure evaluator, performs side-effects.

// #insight
// I don't like the name `interpreter`.

// #todo move excessive error-checking/linting to the resolve/typecheck pass.
// #todo encode effects in the type-system.
// #todo alternative names: Processor, Runner, Interpreter
// #todo split eval_special, eval_func -> not needed if we put everything uniformly in prelude.
// #todo Stack-trace is needed!
// #todo https://clojure.org/reference/evaluation

// #todo try to remove non-needed .into()s <--

// #todo give more 'general' name -> `eval_all` or `eval_vec`?
// #todo what about if a required argument is not passed to a function? currently we report undefined symbol.
pub fn eval_args(args: &[Expr], context: &mut Context) -> Result<Vec<Expr>, Error> {
    // #todo should report ALL errors!

    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        values.push(eval(arg, context)?);
    }
    Ok(values)
}

fn insert_symbol_binding(
    sym: &str,
    range: &Option<Range>,
    value: Expr,
    context: &mut Context,
) -> Result<(), Error> {
    // #todo also is_reserved_symbol is slow, optimize.
    // #todo do we really want this? Maybe convert to a lint?
    if is_reserved_symbol(sym) {
        return Err(Error::invalid_arguments(
            &format!("cannot shadow the reserved symbol `{sym}`"),
            range.clone(),
        ));
    }

    // #todo notify about overrides? use `set`?
    context.scope.insert(sym, value);

    Ok(())
}

// #todo find a better name.
fn insert_binding(name: &Expr, value: Expr, context: &mut Context) -> Result<(), Error> {
    // #todo consider special op/syntax for destructuring?

    // #todo handle potential relevant annotations.

    match name.unpack() {
        // #todo Type/Symbol duplication needs to be resolved, separate Types from Symbols.
        Expr::Type(sym) => {
            // #todo report error if sym == _ or ...
            insert_symbol_binding(sym, &name.range(), value, context)?;
        }
        Expr::Symbol(sym) => {
            // #todo report error if sym == _ or ...
            insert_symbol_binding(sym, &name.range(), value, context)?;
        }
        Expr::List(names) => {
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
                if sym == "..." {
                    break;
                }

                // #todo support "...", "...rest"
                insert_symbol_binding(
                    sym,
                    &name.range(),
                    expr_clone(values.get(i).unwrap()),
                    context,
                )?;
            }
        }
        Expr::Map(items) => {
            // #todo temp, nasty code.
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
                // // #todo what todo about  '_'?
                // if sym == "_" {
                //     continue;
                // }
                // // #todo what todo about '...'?
                // // #insight '...' is called `ellipsis`.
                // if sym == "..." {
                //     break;
                // }
                // #todo support "...", "...rest"
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

// #todo rename to eval_func?
// #todo a version where the arguments are pre-evaluated.
// #todo use this function in eval, later.
pub fn invoke_func(func: &Expr, args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let Expr::Func(params, body, func_scope, file_path) = func.unpack() else {
        // #todo what to do here?
        return Err(Error::invalid_arguments("should be a Func", func.range()));
    };

    // #todo should set the current-module somehow?

    // Evaluate the arguments before calling the function.
    let args = eval_args(args, context)?;

    // #todo ultra-hack to kill shared ref to `env`.
    let params = params.clone();

    // #insight
    // actually we implement static (lexical) scoping here, as we base the new
    // scope on the lexical function scope.

    let prev_scope = context.scope.clone();
    context.scope = Arc::new(Scope::new(func_scope.clone())); // #insight notice we use func_scope here!

    // #todo consider args.into_iter();

    let mut args = args.into_iter();

    for param in params {
        let Some(param_name) = param.as_symbol() else {
            return Err(Error::invalid_arguments(
                "parameter is not a symbol",
                param.range(),
            ));
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
    let mut value = Expr::Nil;

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
                        error.file_path = file_path.clone();
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

// #todo needs better conversion to Expr::Annotated

/// Evaluates via expression rewriting. The expression `expr` evaluates to
/// a fixed point. In essence this is a 'tree-walk' interpreter.
pub fn eval(expr: &Expr, context: &mut Context) -> Result<Expr, Error> {
    let result = match expr.unpack() {
        // #todo are you sure?
        // Expr::Annotated(..) => eval(expr.unpack(), env),
        Expr::Symbol(symbol) => {
            // #todo differentiate between evaluating symbol in 'op' position.
            // #todo combine/optimize check for reserved_symbol and dynamically_scoped, move as much as possible to static-time resolver.
            if is_reserved_symbol(symbol) {
                return Ok(expr.clone());
            }

            // #todo handle 'PathSymbol'

            // #todo try to populate "method"/"signature" annotations during resolving
            // #todo this is missing now that we don't have the resolve stage.
            // #todo maybe resolve or optimize should already have placed the method in the AST?

            let value = if let Some(Expr::String(method)) = expr.annotation("method") {
                // If the symbol is annotated with a `method`, it's in 'operator' position.
                // `method` is just one of the variants of a multi-method-function.
                // #hint: currently dynamically_scope is not supported in this position.
                if let Some(value) = context.scope.get(method) {
                    value
                } else {
                    // #todo leave this trace on in some kind of debug mode.
                    // println!("--> method-fallback");
                    // #todo ultra-hack, if the method is not found, try to lookup the function symbol, fall-through.
                    // #todo should do proper type analysis here.
                    // #todo maybe use a custom Expr::DSSymbol expression to move the detection to read/static time?

                    context
                        .get(symbol, is_dynamically_scoped(symbol))
                        .ok_or::<Error>(Error::undefined_function(
                            symbol,
                            method,
                            &format!("undefined function `{symbol}` with signature `{method}"),
                            expr.range(),
                        ))?
                }
            } else {
                context
                    .get(symbol, is_dynamically_scoped(symbol))
                    .ok_or_else::<Error, _>(|| {
                        Error::undefined_symbol(
                            symbol,
                            &format!("symbol not defined: `{symbol}`"),
                            expr.range(),
                        )
                    })?
            };

            // #todo hm, can we somehow work with references?
            // #hint this could help: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.unwrap_or_clone

            Ok(expr_clone(&value))
        }
        Expr::KeySymbol(..) => {
            // #todo handle 'PathSymbol'
            // #todo strip annotation?

            // #todo lint '::' etc.
            // #todo check that if there is a leading ':' there is only one ':', make this a lint warning!
            // #todo consider renaming KeywordSymbol to KeySymbol.

            // A `Symbol` that starts with `:` is a so-called `KeywordSymbol`. Keyword
            // symbols evaluate to themselves, and are convenient to use as Map keys,
            // named (keyed) function parameter, enum variants, etc.
            Ok(expr.clone())
        }
        // #todo remove this clone.
        Expr::Type(..) => Ok(expr.clone()),
        // #todo if is unquotable!!
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
                Ok(Expr::Nil)
            }
        }
        Expr::List(list) => {
            // #todo no need for dynamic invocable, can use (apply f ...) / (invoke f ...) instead.
            // #todo replace head/tail with first/rest

            if list.is_empty() {
                // () == One (Unit)
                // This is handled statically, in the parser, but an extra, dynamic
                // check is needed in the evaluator to handle the case where the
                // expression is constructed programmatically (e.g. self-modifying code,
                // dynamically constructed expression, homoiconicity, etc).
                return Ok(Expr::Nil);
            }

            // The unwrap here is safe.
            let op = list.first().unwrap();
            let args = &list[1..];

            // #todo could check special forms before the eval

            // #todo this is an ULTRA-HACK! SUPER NASTY/UGLY CODE, refactor!

            // Evaluate the head, try to find dynamic signature
            let head = if let Some(name) = op.as_symbolic() {
                if !is_reserved_symbol(name) {
                    // #todo super nasty hack!!!!
                    let args = eval_args(args, context)?;

                    // #odo we don't support dynamic scoping in this position, reconsider
                    if let Some(value) = context.scope.get(name) {
                        if let Expr::Func(params, ..) = value.unpack() {
                            // #todo extract utility function to invoke a function.
                            // #todo ultra-hack to kill shared ref to `env`.
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

                            let signature = compute_dyn_signature(&args, context);
                            let head = annotate(
                                // #todo #hack think about this!!!!!
                                // #insight we don't use .clone() here, so that Expr::Type is converted to Expr::Symbol()
                                Expr::symbol(name),
                                "method",
                                Expr::String(format!("{name}$${signature}")),
                            );
                            let head = eval(&head, context)?;

                            context.scope = prev_scope;

                            head
                        } else if let Expr::ForeignFunc(_) = value.unpack() {
                            let signature = compute_dyn_signature(&args, context);
                            let head = annotate(
                                // #todo #hack think about this!!!!!
                                // #insight we don't use .clone() here, so that Expr::Type is converted to Expr::Symbol()
                                Expr::symbol(name),
                                "method",
                                Expr::String(format!("{name}$${signature}")),
                            );
                            eval(&head, context)?
                        } else {
                            eval(op, context)?
                        }
                    } else {
                        eval(op, context)?
                    }
                } else {
                    // #todo !?!?
                    eval(op, context)?
                }
            } else {
                eval(op, context)?
            };

            // #todo move special forms to prelude, as Expr::Macro or Expr::Special

            match head.unpack() {
                Expr::Func(..) => invoke_func(&head, args, context),
                Expr::ForeignFunc(foreign_function) => {
                    // #todo extract as `invoke_foreign_function`
                    // #todo do NOT pre-evaluate args for ForeignFunc, allow to implement 'macros'.
                    // Foreign Functions do NOT change the environment, hmm...
                    // #todo use RefCell / interior mutability instead, to allow for changing the environment (with Mutation Effect)

                    // Evaluate the arguments before calling the function.
                    let args = eval_args(args, context)?;

                    anchor(foreign_function(&args, context), expr)
                }
                Expr::Array(arr) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(args, context)?;

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
                        Ok(Expr::Nil)
                    }
                }
                Expr::Map(map) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(args, context)?;

                    // #todo optimize this!
                    // #todo error checking, one arg, stringable, etc.

                    // #insight no need to unpack, format_value sees-through.
                    let key = format_value(&args[0]);

                    let map = try_lock_read(map, expr.range())?;

                    if let Some(value) = map.get(&key) {
                        Ok(value.clone())
                    } else {
                        // #todo introduce Maybe { Some, None }
                        Ok(Expr::Nil)
                    }
                }
                Expr::Type(s) => match s.as_str() {
                    "Char" => {
                        // #todo report more than 1 arguments.

                        let Some(arg) = args.first() else {
                            return Err(Error::invalid_arguments(
                                "malformed Char constructor, missing argument",
                                expr.range(),
                            ));
                        };

                        let Some(c) = arg.as_string() else {
                            return Err(Error::invalid_arguments(
                                "malformed Char constructor, expected String argument",
                                expr.range(),
                            ));
                        };

                        if c.len() != 1 {
                            // #todo better error message.
                            return Err(Error::invalid_arguments(
                                "the Char constructor requires a single-char string",
                                expr.range(),
                            ));
                        }

                        let c = c.chars().next().unwrap();

                        Ok(Expr::Char(c))
                    }
                    "List" => {
                        let args = eval_args(args, context)?;
                        Ok(Expr::List(args))
                    }
                    "Func" => {
                        let Some(params) = args.first() else {
                            return Err(Error::invalid_arguments(
                                "malformed func definition, missing function parameters",
                                expr.range(),
                            ));
                        };

                        let body = &args[1..];

                        // #todo move handling of Expr::One to as_list?

                        // #todo should check both for list and array (i.e. as_iterable)
                        let params = if let Some(params) = params.as_array() {
                            params.clone()
                        } else if params.is_one() {
                            // #insight is_one as in is_unit
                            Vec::new()
                        } else {
                            return Err(Error::invalid_arguments(
                                "malformed func parameters definition",
                                params.range(),
                            ));
                        };

                        // #insight captures the static (lexical scope)

                        // #todo optimize
                        Ok(Expr::Func(
                            params,
                            body.into(),
                            context.scope.clone(),
                            get_current_file_path(context),
                        ))
                    }
                    // #todo lookup constructor function
                    _ => Err(Error::not_invocable(
                        &format!("not invocable constructor `{head}`"),
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
                        // special term
                        // #todo the low-level handling of special forms should use the above high-level cases.
                        // #todo use the `optimize`/`raise` function, here to prepare high-level expression for evaluation, to avoid duplication.
                        "do" => anchor(eval_do(args, context), expr),
                        // #insight `head` seems to have range info, that `expr` lacks.
                        // #todo add range info to expr (no unpack) and use it instead!!!
                        "panic!" => anchor(eval_panic(args, context), &head),
                        "eval" => {
                            // #todo also support eval-all/eval-many? (auto wrap with do?)
                            let [expr] = args else {
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
                            let value = args.first().unwrap_or(&Expr::Nil);
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
                            let value = args.first().unwrap_or(&Expr::Nil);
                            let value = eval(value, context)?;
                            Err(Error::break_cf(value))
                        }
                        "quot" => {
                            // #insight not obvious how to move to static/comptime phase.
                            // #todo doesn't quote all exprs, e.g. the if expression.
                            // #todo optimize with custom exprs, e.g Expr::Quot, Expr::QuasiQuot, etc.

                            let [value] = args else {
                                return Err(Error::invalid_arguments(
                                    "missing quote target",
                                    expr.range(),
                                ));
                            };

                            // #todo transform_mut is not the correct traversal, it's depth first it should be breadth first.
                            // #todo expr.quote() is a temp hack.
                            Ok(value.clone().quot(context))
                        }
                        "for" => anchor(eval_for(args, context), expr),
                        // #todo consider the name `for*` or something similar?
                        "for->list" => {
                            // #insight
                            // `while` is a generalization of `if`
                            // `for` is a generalization of `let`
                            // `for` is related with `do`
                            // `for` is monadic

                            // (for (x 10) (writeln x))

                            // #todo solve duplication between for and for->list
                            // #todo reuse code from let
                            // #todo the resolver should handle this.

                            if args.len() < 2 {
                                // #todo add more structural checks.
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "missing for->list arguments",
                                    expr.range(),
                                ));
                            }

                            let mut values = Vec::new();

                            let binding = args.first().unwrap();
                            let body = &args[1..];

                            // #todo should be as_array to match `for`.
                            // #todo should check both for list and array.
                            let Some(binding_parts) = binding.as_array() else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "invalid for->list binding, not an array",
                                    binding.range(),
                                ));
                            };

                            let [var, value] = &binding_parts[..] else {
                                return Err(Error::invalid_arguments(
                                    "invalid for->list binding",
                                    binding.range(),
                                ));
                            };

                            let Some(var) = var.as_symbol() else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "invalid for->list binding, malformed variable",
                                    var.range(),
                                ));
                            };

                            // #insight for the ListIterator
                            let value = eval(value, context)?;

                            // #todo also handle (Range start end step)
                            // #todo maybe step should be external to Range, or use SteppedRange, or (Step-By (Range T))
                            let Some(iterator) = try_iterator_from(&value) else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "invalid for binding, the value is not iterable",
                                    value.range(),
                                ));
                            };

                            let prev_scope = context.scope.clone();
                            context.scope = Arc::new(Scope::new(prev_scope.clone()));

                            let mut iterator = iterator.borrow_mut();

                            while let Some(value) = iterator.next() {
                                context.scope.insert(var, value);
                                for expr in body {
                                    values.push(eval(expr, context)?);
                                }
                            }

                            context.scope = prev_scope;

                            Ok(Expr::array(values))
                        }
                        "while" => {
                            // #insight
                            // `while` is a generalization of `if`
                            // `for` is a generalization of `let`
                            // `for` is related with `do`
                            // `for` is monadic

                            // #todo
                            // try to merge `while` with `for` (maybe `for` is implemented on top of `while`?)

                            let [predicate, body] = args else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "missing for arguments",
                                    expr.range(),
                                ));
                            };

                            let mut value = Expr::Nil;

                            loop {
                                let predicate = eval(predicate, context)?;

                                let Some(predicate) = predicate.as_bool() else {
                                    return Err(Error::invalid_arguments(
                                        "the `while` predicate is not a boolean value",
                                        predicate.range(),
                                    ));
                                };

                                if !predicate {
                                    break;
                                }

                                value = eval(body, context)?;
                            }

                            Ok(value)
                        }
                        "if" => anchor(eval_if(args, context), expr),
                        // #todo is this different enough from `if`?
                        // (cond
                        //   (> i 5) (...)
                        //   (> i 15) (...)
                        //   else (...)
                        // )
                        "cond" => {
                            let mut i = 0;

                            loop {
                                if i >= args.len() {
                                    // #todo what should we return here? probably Never/Zero?
                                    break Ok(Expr::Nil);
                                }

                                let Some(predicate) = args.get(i) else {
                                    return Err(Error::invalid_arguments(
                                        "malformed cond predicate",
                                        expr.range(),
                                    ));
                                };

                                let Some(clause) = args.get(i + 1) else {
                                    return Err(Error::invalid_arguments(
                                        "malformed cond clause",
                                        expr.range(),
                                    ));
                                };

                                // #todo `else` should not be annotated.
                                // #todo should NOT annotate symbols and keysymbols!
                                // #todo introduce a helper to check for specific symbol.

                                if let Expr::Symbol(sym) = predicate.unpack() {
                                    if sym == "else" {
                                        break eval(clause, context);
                                    }
                                }

                                let predicate = eval(predicate, context)?;

                                let Some(predicate) = predicate.as_bool() else {
                                    return Err(Error::invalid_arguments(
                                        "the cond predicate is not a boolean value",
                                        predicate.range(),
                                    ));
                                };

                                if predicate {
                                    break eval(clause, context);
                                }

                                i += 2;
                            }
                        }
                        // #todo #temp temporary solution.
                        "assert" => eval_assert(op, args, context),
                        "assert-eq" => eval_assert_eq(op, args, context),
                        // #todo for-each or overload for?
                        "for-each" => {
                            // #todo this is a temp hack!
                            let [seq, var, body] = args else {
                                return Err(Error::invalid_arguments(
                                    "malformed `for-each`",
                                    expr.range(),
                                ));
                            };

                            let seq = eval(seq, context)?;

                            let Some(arr) = seq.as_array() else {
                                return Err(Error::invalid_arguments(
                                    "`for-each` requires a `Seq` as the first argument",
                                    seq.range(),
                                ));
                            };

                            let Some(sym) = var.as_symbol() else {
                                return Err(Error::invalid_arguments(
                                    "`for-each` requires a symbol as the second argument",
                                    var.range(),
                                ));
                            };

                            let prev_scope = context.scope.clone();
                            context.scope = Arc::new(Scope::new(prev_scope.clone()));

                            for x in arr.iter() {
                                // #todo array should have Ann<Expr> use Ann<Expr> everywhere, avoid the clones!
                                // #todo replace the clone with custom expr::ref/copy?
                                context.scope.insert(sym, x.clone());
                                eval(body, context)?;
                            }

                            context.scope = prev_scope;

                            // #todo intentionally don't return a value, reconsider this?
                            Ok(Expr::Nil)
                        }
                        "set!" => {
                            // #insight
                            // this is not the same as let, it also traverses the scope stack to find bindings to
                            // update in parent scopes.

                            // #todo find other name: poke, mut, mutate
                            // #todo this is a temp hack
                            // #todo write unit tests
                            // #todo support mutating multiple variables.

                            let [name, value] = args else {
                                return Err(Error::invalid_arguments(
                                    "malformed `set!`",
                                    expr.range(),
                                ));
                            };

                            let Some(name) = name.as_stringable() else {
                                return Err(Error::invalid_arguments(
                                    "`set!` requires a symbol as the first argument",
                                    name.range(),
                                ));
                            };

                            let value = eval(value, context)?;

                            // #todo should we check that the symbol actually exists?
                            context.scope.update(name, value.clone());

                            // #todo what should this return? One/Unit (i.e. nothing useful) or the actual value?
                            Ok(Expr::Nil)
                        }
                        "scope-update" => {
                            // #todo this name conflicts with scope.update()
                            // #todo consider a function that nests a new scope.
                            // #todo maybe it should be some kind of let? e.g. (`let-all` ?? or `let*` or `let..`)
                            // #todo this is a temp hack.

                            // Updates the scope with bindings of the given map.

                            let [map] = args else {
                                return Err(Error::invalid_arguments(
                                    "malformed `scope-update`",
                                    expr.range(),
                                ));
                            };

                            let map = eval(map, context)?;

                            let Some(map) = map.as_map() else {
                                // #todo report what was found!
                                return Err(Error::invalid_arguments(
                                    "malformed `scope-update`, expects Map argument",
                                    expr.range(),
                                ));
                            };

                            for (name, value) in map.iter() {
                                // #todo remove clone.
                                context.scope.insert(name, expr_clone(value));
                            }

                            Ok(Expr::Nil)
                        }
                        // #insight `head` seems to have range info, that `expr` lacks.
                        // #todo add range info to expr (no unpack) and use it instead!!!
                        "use" => anchor(eval_use(args, context), expr),
                        "let-ds" => anchor(eval_let_ds(args, context), expr),
                        "let" => anchor(eval_let(args, context), expr),
                        "and" => {
                            // #insight `and` _is_ short-circuiting and cannot be implemented with a function
                            // #todo what about binary and?
                            // #todo consider operator form? `&&` or `*`
                            // #todo optimize case with 2 arguments.
                            // #todo make a macro
                            // #todo should these 'special forms' get added in scope/env?

                            for arg in args {
                                let value = eval(arg, context)?;
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
                                let value = eval(arg, context)?;
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
