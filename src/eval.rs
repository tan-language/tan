pub mod env;
pub mod prelude;

use std::collections::HashMap;

use crate::{
    ann::Ann,
    error::Error,
    expr::{format_value, Expr},
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
fn eval_args(args: &[Ann<Expr>], env: &mut Env) -> Result<Vec<Expr>, Error> {
    args.iter()
        .map(|x| eval(x, env))
        .collect::<Result<Vec<_>, _>>()
}

/// Evaluates via expression rewriting. The expression `expr` evaluates to
/// a fixed point. In essence this is a 'tree-walk' interpreter.
pub fn eval(expr: impl AsRef<Expr>, env: &mut Env) -> Result<Expr, Error> {
    let expr = expr.as_ref();

    match expr {
        Expr::Symbol(sym) => {
            if is_reserved_symbol(sym) {
                return Ok(expr.clone());
            }

            // #TODO handle 'PathSymbol'

            let result = env.get(sym);

            // #TODO ULTRA-HACK until we properly resolve types
            let result = if result.is_none() {
                if let Some((sym, _)) = sym.split_once("$$") {
                    env.get(sym)
                } else {
                    result
                }
            } else {
                result
            };

            let Some(Ann(expr, ..)) = result else {
                return Err(Error::UndefinedSymbol(sym.clone()));
            };

            // #TODO hm, can we somehow work with references?
            Ok(expr.clone())
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
        Expr::If(predicate, true_clause, false_clause) => {
            let predicate = eval(predicate, env)?;

            let Expr::Bool(predicate) = predicate else {
                // #TODO can we range this error?
                return Err(Error::InvalidArguments("the if predicate is not a boolean value".to_owned()));
            };

            if predicate {
                eval(true_clause, env)
            } else if let Some(false_clause) = false_clause {
                eval(false_clause, env)
            } else {
                // #TODO what should we return if there is no false-clause? Zero/Never?
                Ok(Expr::One)
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
                return Ok(Expr::One);
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
                                return Err(Error::invalid_arguments("parameter is not a symbol"));
                            };

                        env.insert(param, arg);
                    }

                    let result = eval(body, env);

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
                    let Expr::Int(index) = &args[0] else {
                        return Err(Error::InvalidArguments("invalid array index, expecting Int".to_string()));
                    };
                    let index = *index as usize;
                    if let Some(value) = arr.get(index) {
                        Ok(value.clone())
                    } else {
                        // #TODO introduce Maybe { Some, None }
                        Ok(Expr::One)
                    }
                }
                Expr::Dict(dict) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, env)?;

                    // #TODO optimize this!
                    // #TODO error checking, one arg, stringable, etc.
                    let key = format_value(&args[0]);
                    if let Some(value) = dict.get(&key) {
                        Ok(value.clone())
                    } else {
                        // #TODO introduce Maybe { Some, None }
                        Ok(Expr::One)
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
                            let mut value = Expr::One;
                            for expr in tail {
                                value = eval(expr, env)?;
                            }
                            Ok(value)
                        }
                        "ann" => {
                            // #Insight implemented as special-form because it applies to Ann<Expr>.
                            // #TODO try to implement as ForeignFn

                            if tail.len() != 1 {
                                return Err(Error::invalid_arguments(
                                    "`ann` requires one argument",
                                ));
                            }

                            // #TODO support multiple arguments.

                            let expr = tail.first().unwrap();

                            if let Some(ann) = expr.1.clone() {
                                Ok(Expr::Dict(ann))
                            } else {
                                Ok(Expr::Dict(HashMap::new()))
                            }
                        }
                        "quot" => {
                            let [value] = tail else {
                                return Err(Error::invalid_arguments("missing quote target"));
                            };

                            // #TODO hm, that clone, maybe `Rc` can fix this?
                            Ok(value.0.clone())
                        }
                        "for" => {
                            // #Insight
                            // `for` is a generalization of `if`.
                            // `for` is also related with `do`.
                            let [predicate, body] = tail else {
                                // #TODO proper error!
                                return Err(Error::invalid_arguments("missing for arguments"));
                            };

                            let mut value = Expr::One;

                            loop {
                                let predicate = eval(predicate, env)?;

                                let Expr::Bool(predicate) = predicate else {
                                    // #TODO can we range this error?
                                    return Err(Error::invalid_arguments("the for predicate is not a boolean value"));
                                };

                                if !predicate {
                                    break;
                                }

                                value = eval(body, env)?;
                            }

                            Ok(value)
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
                                    return Err(Error::invalid_arguments(format!("`{}` is not a Symbol", sym)));
                                };

                                if is_reserved_symbol(s) {
                                    return Err(Error::invalid_arguments(format!(
                                        "let cannot shadow the reserved symbol `{s}`"
                                    )));
                                }

                                let value = eval(value, env)?;

                                // #TODO notify about overrides? use `set`?
                                env.insert(s, value);
                            }

                            // #TODO return last value!
                            Ok(Expr::One)
                        }
                        "Func" => {
                            let [args, body] = tail else {
                                return Err(Error::invalid_arguments("malformed func invocation"));
                            };

                            let Ann(Expr::List(params), ..) = args else {
                                return Err(Error::invalid_arguments("malformed func invocation parameters"));
                            };

                            // #TODO optimize!
                            Ok(Expr::Func(params.clone(), Box::new(body.clone())))
                        }
                        _ => {
                            return Err(Error::NotInvocable(format!("{}", head)));
                        }
                    }
                }
                _ => {
                    return Err(Error::NotInvocable(format!("{}", head)));
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
