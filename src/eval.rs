pub mod env;
pub mod prelude;
pub mod util;

use std::{collections::HashMap, rc::Rc};

use crate::{
    context::Context,
    error::Error,
    expr::{annotate, format_value, Expr},
    resolver::compute_dyn_signature,
    scope::Scope,
    util::is_reserved_symbol,
};

use self::util::eval_module;

// #Insight
// _Not_ a pure evaluator, performs side-effects.

// #Insight
// I don't like the name `interpreter`.

// #TODO move excessive error-checking/linting to the resolve/typecheck pass.
// #TODO encode effects in the type-system.
// #TODO alternative names: Processor, Runner, Interpreter
// #TODO split eval_special, eval_func -> not needed if we put everything uniformly in prelude.
// #TODO Stack-trace is needed!
// #TODO https://clojure.org/reference/evaluation

// #TODO try to remove non-needed .into()s

// #TODO give more 'general' name.
fn eval_args(args: &[Expr], context: &mut Context) -> Result<Vec<Expr>, Error> {
    args.iter()
        .map(|x| eval(x, context))
        .collect::<Result<Vec<_>, _>>()
}

// #TODO needs better conversion to Expr::Annotated

/// Evaluates via expression rewriting. The expression `expr` evaluates to
/// a fixed point. In essence this is a 'tree-walk' interpreter.
pub fn eval(expr: &Expr, context: &mut Context) -> Result<Expr, Error> {
    match expr.unpack() {
        // #TODO are you sure?
        // Expr::Annotated(..) => eval(expr.unpack(), env),
        Expr::Symbol(symbol) => {
            // #TODO differentiate between evaluating symbol in 'op' position.

            if is_reserved_symbol(symbol) {
                return Ok(expr.clone());
            }

            // #TODO handle 'PathSymbol'

            // #TODO maybe resolve or optimize should already have placed the method in the AST?
            let value = if let Some(Expr::Symbol(method)) = expr.annotation("method") {
                // If the symbol is annotated with a `method`, it's in 'operator' position.
                // `method` is just one of the variants of a multi-method-function.
                if let Some(value) = context.scope.get(method) {
                    value
                } else {
                    // #TODO ultra-hack, if the method is not found, try to lookup the function symbol, fall-through.
                    // #TODO should do proper type analysis here.

                    context
                        .scope
                        .get(symbol)
                        .ok_or::<Error>(Error::undefined_function(
                            symbol,
                            method,
                            &format!("undefined function `{symbol}` with signature `{method}"),
                            expr.range(),
                        ))?
                }
            } else {
                context
                    .scope
                    .get(symbol)
                    .ok_or::<Error>(Error::undefined_symbol(
                        &symbol,
                        &format!("symbol not defined: `{symbol}`"),
                        expr.range(),
                    ))?
            };

            // #TODO hm, can we somehow work with references?
            // #hint this could help: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.unwrap_or_clone
            Ok((*value).clone())
        }
        Expr::KeySymbol(..) => {
            // #TODO handle 'PathSymbol'

            // #TODO lint '::' etc.
            // #TODO check that if there is a leading ':' there is only one ':', make this a lint warning!
            // #TODO consider renaming KeywordSymbol to KeySymbol.

            // A `Symbol` that starts with `:` is a so-called `KeywordSymbol`. Keyword
            // symbols evaluate to themselves, and are convenient to use as Map keys,
            // named (keyed) function parameter, enum variants, etc.
            Ok(expr.clone())
        }
        // #TODO if is unquotable!!
        Expr::If(predicate, true_clause, false_clause) => {
            let predicate = eval(predicate, context)?;

            let Some(predicate) = predicate.as_bool() else {
                return Err(Error::invalid_arguments("the if predicate is not a boolean value", predicate.range()));
            };

            if predicate {
                eval(true_clause, context)
            } else if let Some(false_clause) = false_clause {
                eval(false_clause, context)
            } else {
                // #TODO what should we return if there is no false-clause? Zero/Never?
                Ok(Expr::One.into())
            }
        }
        Expr::List(list) => {
            // #TODO no need for dynamic invocable, can use (apply f ...) / (invoke f ...) instead.
            // #TODO replace head/tail with first/rest

            if list.is_empty() {
                // () == One (Unit)
                // This is handled statically, in the parser, but an extra, dynamic
                // check is needed in the evaluator to handle the case where the
                // expression is constructed programmatically (e.g. self-modifying code,
                // dynamically constructed expression, homoiconicity, etc).
                return Ok(Expr::One.into());
            }

            // The unwrap here is safe.
            let head = list.first().unwrap();
            let tail = &list[1..];

            // #TODO could check special forms before the eval

            // #TODO this is an ULTRA-HACK! SUPER NASTY/UGLY CODE, refactor!

            // Evaluate the head, try to find dynamic signature
            let head = if let Some(name) = head.as_symbol() {
                if !is_reserved_symbol(name) {
                    // #TODO super arghhhh!!!!
                    let args = eval_args(tail, context)?;

                    if let Some(value) = context.scope.get(name) {
                        if let Expr::Func(params, ..) = value.unpack() {
                            // #TODO ultra-hack to kill shared ref to `env`.
                            let params = params.clone();

                            let prev_scope = context.scope.clone();
                            context.scope = Rc::new(Scope::new(prev_scope.clone()));

                            // env.push_new_scope();

                            for (param, arg) in params.iter().zip(&args) {
                                let Some(param) = param.as_symbol() else {
                                        return Err(Error::invalid_arguments("parameter is not a symbol", param.range()));
                                    };

                                context.scope.insert(param, arg.clone());
                            }

                            let signature = compute_dyn_signature(&args, context);
                            let head = annotate(
                                head.clone(),
                                "method",
                                Expr::Symbol(format!("{name}$${signature}")),
                            );
                            let head = eval(&head, context)?;

                            // env.pop();

                            context.scope = prev_scope;

                            head
                        } else if let Expr::ForeignFunc(_) = value.unpack() {
                            let signature = compute_dyn_signature(&args, &context);
                            let head = annotate(
                                head.clone(),
                                "method",
                                Expr::Symbol(format!("{name}$${signature}")),
                            );
                            let head = eval(&head, context)?;

                            head
                        } else {
                            eval(head, context)?
                        }
                    } else {
                        eval(head, context)?
                    }
                } else {
                    eval(head, context)?
                }
            } else {
                eval(head, context)?
            };

            // #TODO move special forms to prelude, as Expr::Macro or Expr::Special

            match head.unpack() {
                Expr::Func(params, body, func_scope) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, context)?;

                    // #TODO ultra-hack to kill shared ref to `env`.
                    let params = params.clone();

                    // Dynamic scoping, #TODO convert to lexical.

                    // env.push_new_scope();

                    let prev_scope = context.scope.clone();
                    context.scope = Rc::new(Scope::new(func_scope.clone()));

                    for (param, arg) in params.iter().zip(args) {
                        let Some(param) = param.as_symbol() else {
                                return Err(Error::invalid_arguments("parameter is not a symbol", param.range()));
                            };

                        context.scope.insert(param, arg);
                    }

                    // #TODO this code is the same as in the (do ..) block, extract.

                    // #TODO do should be 'monadic', propagate Eff (effect) wrapper.
                    let mut value = Expr::One;

                    for expr in body {
                        value = eval(expr, context)?;
                    }

                    // env.pop();

                    context.scope = prev_scope;

                    Ok(value)
                }
                Expr::ForeignFunc(foreign_function) => {
                    // #TODO do NOT pre-evaluate args for ForeignFunc, allow to implement 'macros'.
                    // Foreign Functions do NOT change the environment, hmm...
                    // #TODO use RefCell / interior mutability instead, to allow for changing the environment (with Mutation Effect)

                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, context)?;

                    let result = foreign_function(&args, context);

                    // If the error has no range, try to apply the range of the invocation.
                    if let Err(mut error) = result {
                        if let Some(note) = error.notes.first_mut() {
                            if note.range.is_none() {
                                note.range = expr.range()
                            }
                        };

                        return Err(error);
                    };

                    result
                }
                Expr::Array(arr) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, context)?;

                    // #TODO optimize this!
                    // #TODO error checking, one arg, etc.
                    let index = &args[0];
                    // #TODO we need UInt, USize, Nat type
                    let Some(index) = index.as_int() else {
                        return Err(Error::invalid_arguments("invalid array index, expecting Int", index.range()));
                    };
                    let index = index as usize;
                    if let Some(value) = arr.get(index) {
                        Ok(value.clone().into())
                    } else {
                        // #TODO introduce Maybe { Some, None }
                        Ok(Expr::One.into())
                    }
                }
                Expr::Dict(dict) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, context)?;

                    // #TODO optimize this!
                    // #TODO error checking, one arg, stringable, etc.

                    // #insight no need to unpack, format_value sees-through.
                    let key = format_value(&args[0]);
                    if let Some(value) = dict.get(&key) {
                        Ok(value.clone().into())
                    } else {
                        // #TODO introduce Maybe { Some, None }
                        Ok(Expr::One.into())
                    }
                }
                // #TODO add handling of 'high-level', compound expressions here.
                // #TODO Expr::If
                // #TODO Expr::Let
                // #TODO Expr::Do
                // #TODO Expr::..
                Expr::Symbol(s) => {
                    match s.as_str() {
                        // special term
                        // #TODO the low-level handling of special forms should use the above high-level cases.
                        // #TODO use the `optimize`/`raise` function, here to prepare high-level expression for evaluation, to avoid duplication.
                        "do" => {
                            // #TODO do should be 'monadic', propagate Eff (effect) wrapper.
                            let mut value = Expr::One.into();

                            // env.push_new_scope();

                            let prev_scope = context.scope.clone();
                            context.scope = Rc::new(Scope::new(prev_scope.clone()));

                            for expr in tail {
                                value = eval(expr, context)?;
                            }

                            // env.pop();

                            context.scope = prev_scope;

                            Ok(value)
                        }
                        "ann" => {
                            // #Insight implemented as special-form because it applies to Ann<Expr>.
                            // #TODO try to implement as ForeignFn

                            if tail.len() != 1 {
                                return Err(Error::invalid_arguments(
                                    "`ann` requires one argument",
                                    expr.range(),
                                ));
                            }

                            // #TODO support multiple arguments.

                            let expr = tail.first().unwrap();

                            if let Some(ann) = expr.annotations() {
                                Ok(Expr::Dict(ann.clone()))
                            } else {
                                Ok(Expr::Dict(HashMap::new()))
                            }
                        }
                        "eval" => {
                            let [expr] = tail else {
                                return Err(Error::invalid_arguments("missing expression to be evaluated", expr.range()));
                            };

                            // #TODO consider naming this `form`?
                            let expr = eval(expr, context)?;

                            eval(&expr, context)
                        }
                        // #TODO can move to static/comptime phase.
                        // #TODO doesn't quote all exprs, e.g. the if expression.
                        "quot" => {
                            let [value] = tail else {
                                return Err(Error::invalid_arguments("missing quote target", expr.range()));
                            };

                            // #TODO hm, that clone, maybe `Rc` can fix this?
                            Ok(value.unpack().clone())
                        }
                        "for" => {
                            // #Insight
                            // `for` is a generalization of `if`.
                            // `for` is also related with `do`.
                            let [predicate, body] = tail else {
                                // #TODO proper error!
                                return Err(Error::invalid_arguments("missing for arguments", expr.range()));
                            };

                            let mut value = Expr::One.into();

                            loop {
                                let predicate = eval(predicate, context)?;

                                let Some(predicate) = predicate.as_bool() else {
                                    return Err(Error::invalid_arguments("the for predicate is not a boolean value", predicate.range()));
                                };

                                if !predicate {
                                    break;
                                }

                                value = eval(body, context)?;
                            }

                            Ok(value)
                        }
                        "if" => {
                            // #TODO this is a temp hack!
                            let Some(predicate) = tail.get(0) else {
                                return Err(Error::invalid_arguments("malformed if predicate", expr.range()));
                            };

                            let Some(true_clause) = tail.get(1) else {
                                return Err(Error::invalid_arguments("malformed if true clause", expr.range()));
                            };

                            let false_clause = tail.get(2);

                            let predicate = eval(predicate, context)?;

                            let Some(predicate) = predicate.as_bool() else {
                                return Err(Error::invalid_arguments("the if predicate is not a boolean value", predicate.range()));
                            };

                            if predicate {
                                eval(true_clause, context)
                            } else if let Some(false_clause) = false_clause {
                                eval(false_clause, context)
                            } else {
                                // #TODO what should we return if there is no false-clause? Zero/Never?
                                Ok(Expr::One.into())
                            }
                        }
                        // #TODO for-each or overload for?
                        "for-each" => {
                            // #TODO this is a temp hack!
                            let [seq, var, body] = tail else {
                                return Err(Error::invalid_arguments("malformed `for-each`", expr.range()));
                            };

                            let seq = eval(seq, context)?;

                            let Some(arr) = seq.as_array() else {
                                return Err(Error::invalid_arguments("`for-each` requires a `Seq` as the first argument", seq.range()));
                            };

                            let Some(sym) = var.as_symbol() else {
                                return Err(Error::invalid_arguments("`for-each` requires a symbol as the second argument", var.range()));
                            };

                            // env.push_new_scope();

                            let prev_scope = context.scope.clone();
                            context.scope = Rc::new(Scope::new(prev_scope.clone()));

                            for x in arr {
                                // #TODO array should have Ann<Expr> use Ann<Expr> everywhere, avoid the clones!
                                context.scope.insert(sym, x.clone());
                                eval(body, context)?;
                            }

                            // env.pop();
                            context.scope = prev_scope;

                            // #TODO intentionally don't return a value, reconsider this?
                            Ok(Expr::One.into())
                        }
                        // #TODO extract
                        // #TODO functions implemented here have dynamic dispatch!
                        "map" => {
                            // #TODO this is a temp hack!
                            let [seq, var, body] = tail else {
                                return Err(Error::invalid_arguments("malformed `map`", expr.range()));
                            };

                            let seq = eval(seq, context)?;

                            let Some(arr) = seq.as_array() else {
                                return Err(Error::invalid_arguments("`map` requires a `Seq` as the first argument", seq.range()));
                            };

                            let Some(sym) = var.as_symbol() else {
                                return Err(Error::invalid_arguments("`map` requires a symbol as the second argument", var.range()));
                            };

                            // env.push_new_scope();

                            let prev_scope = context.scope.clone();
                            context.scope = Rc::new(Scope::new(prev_scope.clone()));

                            let mut results: Vec<Expr> = Vec::new();

                            for x in arr {
                                // #TODO array should have Ann<Expr> use Ann<Expr> everywhere, avoid the clones!
                                context.scope.insert(sym, x.clone());
                                let result = eval(body, context)?;
                                results.push(result.unpack().clone());
                            }

                            // env.pop();
                            context.scope = prev_scope.clone();

                            // #TODO intentionally don't return a value, reconsider this?
                            Ok(Expr::Array(results).into())
                        }
                        "use" => {
                            // Import a directory as a module.

                            let Some(term) = tail.get(0) else {
                                return Err(Error::invalid_arguments("malformed use expression", expr.range()));
                            };

                            let Some(module_path) = term.as_string() else {
                                return Err(Error::invalid_arguments("malformed use expression", expr.range()));
                            };

                            // #TODO make sure paths are relative to the current file.
                            let result = eval_module(module_path, context);

                            if let Err(errors) = result {
                                // #TODO precise formating is _required_ here!
                                // eprintln!("{}", format_errors(&errors));
                                // dbg!(errors);
                                return Err(Error::failed_use(&module_path, errors));
                            };

                            // #TODO what could we return here?
                            Ok(Expr::One.into())
                        }
                        "let" => {
                            // #TODO this is already parsed statically by resolver, no need to duplicate the tests here?
                            // #TODO also report some of these errors statically, maybe in a sema phase?
                            let mut args = tail.iter();

                            loop {
                                let Some(name) = args.next() else {
                                    break;
                                };

                                let Some(value) = args.next() else {
                                    // #TODO error?
                                    break;
                                };

                                let Some(s) = name.as_symbol() else {
                                    return Err(Error::invalid_arguments(&format!("`{name}` is not a Symbol"), name.range()));
                                };

                                // #TODO do we really want this? Maybe convert to a lint?
                                if is_reserved_symbol(s) {
                                    return Err(Error::invalid_arguments(
                                        &format!("let cannot shadow the reserved symbol `{s}`"),
                                        name.range(),
                                    ));
                                }

                                let value = eval(value, context)?;

                                // #TODO notify about overrides? use `set`?
                                context.scope.insert(s, value);
                            }

                            // #TODO return last value!
                            Ok(Expr::One.into())
                        }
                        "Char" => {
                            // #TODO report more than 1 arguments.

                            let Some(arg) = tail.get(0) else {
                                return Err(Error::invalid_arguments("malformed Char constructor, missing argument", expr.range()));
                            };

                            let Some(c) = arg.as_string() else {
                                return Err(Error::invalid_arguments("malformed Char constructor, expected String argument", expr.range()));
                            };

                            if c.len() != 1 {
                                // #TODO better error message.
                                return Err(Error::invalid_arguments(
                                    "the Char constructor requires a single-char string",
                                    expr.range(),
                                ));
                            }

                            let c = c.chars().next().unwrap();

                            Ok(Expr::Char(c).into())
                        }
                        "List" => {
                            let args = eval_args(tail, context)?;
                            Ok(Expr::List(args).into())
                        }
                        "Func" => {
                            let Some(params) = tail.first() else {
                                // #TODO seems the range is not reported correctly here!!!
                                return Err(Error::invalid_arguments(
                                    "malformed func definition, missing function parameters",
                                    expr.range(),
                                ));
                            };

                            let body = &tail[1..];

                            let Some(params) = params.as_list() else {
                                return Err(Error::invalid_arguments("malformed func parameters definition", params.range()));
                            };

                            // #insight captures the static (lexical scope)

                            // #TODO optimize!
                            Ok(Expr::Func(
                                params.clone(),
                                body.into(),
                                context.scope.clone(),
                            ))
                        }
                        // #TODO macros should be handled at a separate, comptime, macroexpand pass.
                        // #TODO actually two passes, macro_def, macro_expand
                        // #TODO probably macro handling should be removed from eval, there are no runtime/dynamic macro definitions!!
                        "Macro" => {
                            let Some(params) = tail.first() else {
                                // #TODO seems the range is not reported correctly here!!!
                                return Err(Error::invalid_arguments(
                                    "malformed macro definition, missing function parameters",
                                    expr.range(),
                                ));
                            };

                            let body = &tail[1..];

                            let Some(params) = params.as_list() else {
                                return Err(Error::invalid_arguments("malformed macro parameters definition", params.range()));
                            };

                            // #TODO optimize!
                            Ok(Expr::Macro(params.clone(), body.into()))
                        }
                        _ => {
                            return Err(Error::not_invocable(
                                &format!("symbol `{head}`"),
                                head.range(),
                            ));
                        }
                    }
                }
                _ => {
                    return Err(Error::not_invocable(
                        &format!("expression `{head}`"),
                        head.range(),
                    ));
                }
            }
        }
        Expr::Array(items) => {
            // #insight [...] => (Array ...) => it's like a function.
            // #TODO can this get pre-evaluated statically in some cases?
            let mut evaled_items = Vec::new();
            for item in items {
                evaled_items.push(eval(item, context)?);
            }
            Ok(Expr::Array(evaled_items))
        }
        _ => {
            // #TODO hm, maybe need to report an error here? or even select the desired behavior? -> NO ERROR
            // #TODO can we avoid the clone?
            // Unhandled expression variants evaluate to themselves.
            Ok(expr.clone())
        }
    }
}
