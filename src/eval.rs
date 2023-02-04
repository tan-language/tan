pub mod env;
pub mod prelude;

use std::collections::HashMap;

use crate::{
    ann::Ann,
    error::Error,
    expr::{format_value, Expr},
    range::Ranged,
    util::is_reserved_symbol,
};

use self::env::Env;

// #Insight
// _Not_ a pure evaluator, performs side-effects.

// #Insight
// I don't like the name `interpreter`.

// #TODO move excessive error-checking/linting to the resolve/typecheck pass.
// #TODO encode effects in the type-system.
// #TODO alternative names: Processor, Runner, Interpreter
// #TODO split eval_special, eval_func -> not needed if we put everything uniformly in prelude.
// #TODO Stack-trace is needed!

// #TODO give more 'general' name.
fn eval_args(args: &[Ann<Expr>], env: &mut Env) -> Result<Vec<Ann<Expr>>, Ranged<Error>> {
    args.iter()
        .map(|x| eval(x, env))
        .collect::<Result<Vec<_>, _>>()
}

/// Evaluates via expression rewriting. The expression `expr` evaluates to
/// a fixed point. In essence this is a 'tree-walk' interpreter.
pub fn eval(expr: &Ann<Expr>, env: &mut Env) -> Result<Ann<Expr>, Ranged<Error>> {
    // let expr = expr.as_ref();

    match expr {
        Ann(Expr::Symbol(sym), _) => {
            // #TODO differentiate between evaluating symbol in 'op' position.

            if is_reserved_symbol(sym) {
                return Ok(expr.clone());
            }

            // #TODO handle 'PathSymbol'

            let value = if let Some(Expr::Symbol(method)) = expr.get_annotation("method") {
                // If the symbol is annotated with a method, it's in 'operator' position.
                if let Some(value) = env.get(method) {
                    value
                } else {
                    // #TODO ultra-hack, if the method is not found, try to lookup the function symbol, fall-through.
                    // #TODO should do proper type analysis here.
                    env.get(sym).ok_or::<Ranged<Error>>(
                        Error::UndefinedFunction(sym.to_owned(), method.to_owned()).into(),
                    )?
                }
            } else {
                env.get(sym)
                    .ok_or::<Ranged<Error>>(Error::UndefinedSymbol(sym.clone()).into())?
            };

            // #TODO hm, can we somehow work with references?
            Ok(value.clone())
        }
        Ann(Expr::KeySymbol(..), ..) => {
            // #TODO handle 'PathSymbol'

            // #TODO lint '::' etc.
            // #TODO check that if there is a leading ':' there is only one ':', make this a lint warning!
            // #TODO consider renaming KeywordSymbol to KeySymbol.

            // A `Symbol` that starts with `:` is a so-called `KeywordSymbol`. Keyword
            // symbols evaluate to themselves, and are convenient to use as Map keys,
            // named (keyed) function parameter, enum variants, etc.
            Ok(expr.clone())
        }
        // #TODO argh, if is unquotable!!
        Ann(Expr::If(predicate, true_clause, false_clause), ..) => {
            let predicate = eval(predicate, env)?;

            let Ann(Expr::Bool(predicate), ..) = predicate else {
                // #TODO can we range this error?
                return Err(Error::InvalidArguments("the if predicate is not a boolean value".to_owned()).into());
            };

            if predicate {
                eval(true_clause, env)
            } else if let Some(false_clause) = false_clause {
                eval(false_clause, env)
            } else {
                // #TODO what should we return if there is no false-clause? Zero/Never?
                Ok(Expr::One.into())
            }
        }
        Ann(Expr::List(list), ..) => {
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

            // Evaluate the head
            let head = eval(head, env)?;

            // #TODO move special forms to prelude, as Expr::Macro or Expr::Special

            match head.as_ref() {
                Expr::Func(params, body) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, env)?;

                    // #TODO ultra-hack to kill shared ref to `env`.
                    let params = params.clone();
                    let body = body.clone();

                    // Dynamic scoping, #TODO convert to lexical.

                    env.push_new_scope();

                    for (param, arg) in params.iter().zip(args) {
                        let Ann(Expr::Symbol(param), ..) = param else {
                                return Err(Error::invalid_arguments("parameter is not a symbol").into());
                            };

                        env.insert(param, arg);
                    }

                    let result = eval(&body, env);

                    env.pop();

                    result
                }
                Expr::ForeignFunc(foreign_function) => {
                    // #TODO do NOT pre-evaluate args for ForeignFunc, allow to implement 'macros'.
                    // Foreign Functions do NOT change the environment, hmm...
                    // #TODO use RefCell / interior mutability instead, to allow for changing the environment (with Mutation Effect)

                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, env)?;

                    foreign_function(&args, env)
                }
                Expr::Array(arr) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, env)?;

                    // #TODO optimize this!
                    // #TODO error checking, one arg, etc.
                    let Ann(Expr::Int(index), ..) = &args[0] else {
                        return Err(Error::InvalidArguments("invalid array index, expecting Int".to_string()).into());
                    };
                    let index = *index as usize;
                    if let Some(value) = arr.get(index) {
                        Ok(value.clone().into())
                    } else {
                        // #TODO introduce Maybe { Some, None }
                        Ok(Expr::One.into())
                    }
                }
                Expr::Dict(dict) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, env)?;

                    // #TODO optimize this!
                    // #TODO error checking, one arg, stringable, etc.
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

                            env.push_new_scope();

                            for expr in tail {
                                value = eval(expr, env)?;
                            }

                            env.pop();

                            Ok(value)
                        }
                        "ann" => {
                            // #Insight implemented as special-form because it applies to Ann<Expr>.
                            // #TODO try to implement as ForeignFn

                            if tail.len() != 1 {
                                return Err(Error::invalid_arguments(
                                    "`ann` requires one argument",
                                )
                                .into());
                            }

                            // #TODO support multiple arguments.

                            let expr = tail.first().unwrap();

                            if let Some(ann) = expr.1.clone() {
                                Ok(Expr::Dict(ann).into())
                            } else {
                                Ok(Expr::Dict(HashMap::new()).into())
                            }
                        }
                        "eval" => {
                            let [expr] = tail else {
                                return Err(Error::invalid_arguments("missing expression to be evaluated").into());
                            };

                            // #TODO consider naming this `form`?
                            let expr = eval(expr, env)?;

                            eval(&expr, env)
                        }
                        // #TODO can move to static/comptime phase.
                        // #TODO doesn't quote all exprs, e.g. the if expression.
                        "quot" => {
                            let [value] = tail else {
                                return Err(Error::invalid_arguments("missing quote target").into());
                            };

                            // #TODO hm, that clone, maybe `Rc` can fix this?
                            Ok(value.0.clone().into())
                        }
                        "for" => {
                            // #Insight
                            // `for` is a generalization of `if`.
                            // `for` is also related with `do`.
                            let [predicate, body] = tail else {
                                // #TODO proper error!
                                return Err(Error::invalid_arguments("missing for arguments").into());
                            };

                            let mut value = Expr::One.into();

                            loop {
                                let predicate = eval(predicate, env)?;

                                let Ann(Expr::Bool(predicate), ..) = predicate else {
                                    // #TODO can we range this error?
                                    return Err(Error::invalid_arguments("the for predicate is not a boolean value").into());
                                };

                                if !predicate {
                                    break;
                                }

                                value = eval(body, env)?;
                            }

                            Ok(value)
                        }
                        "if" => {
                            // #TODO this is a temp hack!
                            let Some(predicate) = tail.get(0) else {
                                return Err(Error::invalid_arguments("malformed if predicate").into());
                            };

                            let Some(true_clause) = tail.get(1) else {
                                return Err(Error::invalid_arguments("malformed if true clause").into());
                            };


                            let false_clause = tail.get(2);

                            let predicate = eval(predicate, env)?;

                            let Ann(Expr::Bool(predicate), ..) = predicate else {
                                // #TODO can we range this error?
                                return Err(Error::InvalidArguments("the if predicate is not a boolean value".to_owned()).into());
                            };

                            if predicate {
                                eval(true_clause, env)
                            } else if let Some(false_clause) = false_clause {
                                eval(false_clause, env)
                            } else {
                                // #TODO what should we return if there is no false-clause? Zero/Never?
                                Ok(Expr::One.into())
                            }
                        }
                        "for_each" => {
                            // #TODO this is a temp hack!
                            let [seq, var, body] = tail else {
                                return Err(Error::invalid_arguments("malformed `for_each`").into());
                            };

                            let seq = eval(seq, env)?;

                            let Ann(Expr::Array(arr), ..) = seq else {
                                // #TODO can we range this error?
                                return Err(Error::invalid_arguments("`for_each` requires a `Seq` as the first argument").into());
                            };

                            let Ann(Expr::Symbol(sym), _) = var else {
                                // #TODO can we range this error?
                                return Err(Error::invalid_arguments("`for_each` requires a symbol as the second argument").into());
                            };

                            env.push_new_scope();

                            for x in arr {
                                // #TODO array should have Ann<Expr> use Ann<Expr> everywhere, avoid the clones!
                                env.insert(sym, Ann::new(x.clone()));
                                eval(body, env)?;
                            }

                            env.pop();

                            // #TODO intentionally don't return a value, reconsider this?
                            Ok(Expr::One.into())
                        }
                        "let" => {
                            // #TODO also report some of these errors statically, maybe in a sema phase?
                            let mut args = tail.iter();

                            loop {
                                let Some(sym) = args.next() else {
                                    break;
                                };

                                let Some(value) = args.next() else {
                                    // #TODO error?
                                    break;
                                };

                                let Ann(Expr::Symbol(s), ..) = sym else {
                                    return Err(Error::invalid_arguments(format!("`{sym}` is not a Symbol")).into());
                                };

                                if is_reserved_symbol(s) {
                                    return Err(Error::invalid_arguments(format!(
                                        "let cannot shadow the reserved symbol `{s}`"
                                    ))
                                    .into());
                                }

                                let value = eval(value, env)?;

                                // #TODO notify about overrides? use `set`?
                                env.insert(s, value);
                            }

                            // #TODO return last value!
                            Ok(Expr::One.into())
                        }
                        "Func" => {
                            let [args, body] = tail else {
                                return Err(Error::invalid_arguments("malformed func definition").into());
                            };

                            let Ann(Expr::List(params), ..) = args else {
                                return Err(Error::invalid_arguments("malformed func parameters definition").into());
                            };

                            // #TODO optimize!
                            Ok(Expr::Func(params.clone(), Box::new(body.clone())).into())
                        }
                        // #TODO macros should be handled at a separate, comptime, macroexpand pass.
                        // #TODO actually two passes, macro_def, macro_expand
                        "Macro" => {
                            let [args, body] = tail else {
                                return Err(Error::invalid_arguments("malformed macro definition").into());
                            };

                            let Ann(Expr::List(params), ..) = args else {
                                return Err(Error::invalid_arguments("malformed macro parameters definition").into());
                            };

                            // #TODO optimize!
                            Ok(Expr::Macro(params.clone(), Box::new(body.clone())).into())
                        }
                        _ => {
                            return Err(Error::NotInvocable(format!("{head}")).into());
                        }
                    }
                }
                _ => {
                    return Err(Error::NotInvocable(format!("{head}")).into());
                }
            }
        }
        _ => {
            // #TODO hm, maybe need to report an error here? or even select the desired behavior? -> NO ERROR
            // #TODO can we avoid the clone?
            // Unhandled expression variants evaluate to themselves.
            Ok(expr.clone())
        }
    }
}
