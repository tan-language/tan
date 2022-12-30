pub mod env;
pub mod error;
pub mod prelude;

use crate::{ann::Ann, expr::Expr};

use self::{env::Env, error::EvalError};

// #Insight
// _Not_ a pure evaluator, performs side-effects.

// #Insight
// I don't like the name `interpreter`.

// #TODO encode effects in the type-system.
// #TODO alternative names: Processor, Runner, Interpreter
// #TODO split eval_special, eval_func
// #TODO Stack-trace is needed!

/// Evaluates via expression rewriting. The expression `expr` evaluates to
/// a fixed point. In essence this is a 'tree-walk' interpreter.
pub fn eval(expr: impl AsRef<Expr>, env: &mut Env) -> Result<Expr, EvalError> {
    let expr = expr.as_ref();
    let result = match expr {
        Expr::Symbol(sym) => {
            let result = env.get(sym);

            let Some(Ann(expr, ..)) = result else {
                return Err(EvalError::UndefinedSymbolError(sym.clone()));
            };

            // #TODO hm, can we somehow work with references?
            Ok(expr.clone())
        }
        Expr::If(predicate, true_clause, false_clause) => {
            let predicate = eval(predicate, env)?;

            let Expr::Bool(predicate) = predicate else {
                // #TODO can we range this error?
                return Err(EvalError::ArgumentError("the if predicate is not a boolean value".to_owned()));
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
            // #TODO replace head/tail with first/rest
            // #TODO also eval the head?

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

            match head.as_ref() {
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
                            return Ok(value);
                        }
                        "quot" => {
                            let [value] = tail else {
                                return Err(EvalError::ArgumentError("missing quote target".to_owned()));
                            };

                            // #TODO hm, that clone, maybe `Rc` can fix this?
                            return Ok(value.0.clone());
                        }
                        "for" => {
                            // #Insight
                            // `for` is a generalization of `if`.
                            // `for` is also related with `do`.
                            let [predicate, body] = tail else {
                                // #TODO proper error!
                                return Err(EvalError::UnknownError);
                            };

                            let mut value = Expr::One;

                            loop {
                                let predicate = eval(predicate, env)?;

                                let Expr::Bool(predicate) = predicate else {
                                    // #TODO can we range this error?
                                    return Err(EvalError::ArgumentError("the for predicate is not a boolean value".to_owned()));
                                };

                                if !predicate {
                                    break;
                                }

                                value = eval(body, env)?;
                            }

                            return Ok(value);
                        }
                        "let" => {
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
                                    // #TODO proper error!
                                    return Err(EvalError::UnknownError);
                                };

                                let value = eval(value, env)?;

                                // #TODO notify about overrides? use `set`?
                                env.insert(s, value);
                            }

                            // #TODO return last value!
                            return Ok(Expr::One);
                        }
                        "Func" => {
                            let [args, body] = tail else {
                                // #TODO proper error!
                                return Err(EvalError::UnknownError);
                            };

                            let Ann(Expr::List(params), ..) = args else {
                                // #TODO proper error!
                                return Err(EvalError::UnknownError);
                            };

                            // #TODO optimize!
                            Ok(Expr::Func(params.clone(), Box::new(body.clone())))
                        }
                        _ => {
                            // non-special term -> application.

                            // Evaluate the arguments before calling the function.
                            let args = tail
                                .iter()
                                .map(|x| eval(x, env))
                                .collect::<Result<Vec<_>, _>>()?;

                            match s.as_str() {
                                // #TODO also eval 'if', 'do', 'for' and other keywords here!
                                "-" => {
                                    // #TODO support multiple arguments.
                                    let [a, b] = &args[..] else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    let Expr::Int(a) = a else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    let Expr::Int(b) = b else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    Ok(Expr::Int(a - b))
                                }
                                "*" => {
                                    // #TODO optimize!
                                    let mut prod = 1;

                                    for arg in args {
                                        let Expr::Int(n) = arg else {
                                            // #TODO proper error!
                                            return Err(EvalError::UnknownError);
                                        };
                                        prod *= n;
                                    }

                                    Ok(Expr::Int(prod))
                                }
                                ">" => {
                                    // #TODO support multiple arguments.
                                    let [a, b] = &args[..] else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    let Expr::Int(a) = a else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    let Expr::Int(b) = b else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    Ok(Expr::Bool(a > b))
                                }
                                // #TODO helper function or macro for arithmetic operations!
                                "<" => {
                                    // #TODO support multiple arguments.
                                    let [a, b] = &args[..] else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    let Expr::Int(a) = a else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    let Expr::Int(b) = b else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    Ok(Expr::Bool(a < b))
                                }
                                "=" => {
                                    // Use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
                                    // #TODO support overloading,
                                    // #TODO make equality a method of Expr?
                                    // #TODO support non-Int types
                                    // #TODO support multiple arguments.
                                    let [a, b] = &args[..] else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    let Expr::Int(a) = a else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    let Expr::Int(b) = b else {
                                        // #TODO proper error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    Ok(Expr::Bool(a == b))
                                }
                                _ => {
                                    // Try to apply an op!

                                    // #Insight `op` = 'callable` (func, macro, collection, actor, etc)

                                    let Some(op) = env.get(s) else {
                                        return Err(EvalError::UndefinedSymbolError(s.clone()));
                                    };

                                    match op {
                                        Ann(Expr::Func(params, body), ..) => {
                                            // #TODO ultra-hack to kill shared ref to `env`.
                                            let params = params.clone();
                                            let body = body.clone();

                                            // Dynamic scoping

                                            env.push_new_scope();

                                            for (param, arg) in params.iter().zip(args) {
                                                let Ann(Expr::Symbol(param), ..) = param else {
                                                    // #TODO non-callable error!
                                                    return Err(EvalError::UnknownError);
                                                };

                                                env.insert(param, arg);
                                            }

                                            let result = eval(body, env);

                                            env.pop();

                                            result
                                        }
                                        Ann(Expr::ForeignFunc(foreign_function), ..) => {
                                            // Foreign Functions do NOT change the environment, hmm...
                                            // #TODO use RefCell / interior mutability instead, to allow for changing the environment (with Mutation Effect)
                                            foreign_function(&args, env)
                                        }
                                        _ => {
                                            // #TODO non-callable error!
                                            return Err(EvalError::UnknownError);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Err(EvalError::NotInvocableError(format!("{}", head.0)));
                }
            }
        }
        _ => {
            // #TODO hm, maybe need to report an error here? or even select the desired behavior?
            // Unhandled expression variants evaluate to themselves.
            return Ok(expr.clone());
        }
    };

    result
}
