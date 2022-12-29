pub mod env;
pub mod error;

use crate::{ann::Annotated, expr::Expr};

use self::{env::Env, error::EvalError};

// #TODO Stack-trace is needed!

// #Insight
// _Not_ a pure evaluator, performs side-effects.

// #TODO encode effects in the type-system.
// #TODO interpret or eval or execute?
// #TODO alternative names: Processor, Runner
// #TODO check that eval accepts plain Expr.

/// Evaluates via expression rewriting. The expression `expr` evaluates to
/// a fixed point. In essence this is a 'tree-walk' interpreter.
pub fn eval(expr: impl AsRef<Expr>, env: &mut Env) -> Result<Expr, EvalError> {
    let expr = expr.as_ref();
    let result = match expr {
        Expr::Symbol(sym) => {
            let result = env.get(sym);

            let Some(Annotated(expr, ..)) = result else {
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
            // #TODO empty list should also be found in read/parse phase
            // #TODO could this arise in self-modifying code?
            // #TODO also eval the head?
            let head = list.first().ok_or(EvalError::UnknownError)?;
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
                                let Annotated(Expr::Symbol(s), ..) = sym else {
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

                            let Annotated(Expr::List(params), ..) = args else {
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
                                "write" => {
                                    // #TODO for some reason, "\n" is not working.
                                    let output = args.iter().fold(String::new(), |mut str, x| {
                                        str.push_str(&format!("{}", x));
                                        str
                                    });

                                    // #TODO shenanigans to handle `\n` in string, how can we do this better?
                                    for line in output.split_inclusive("\\n") {
                                        if line.ends_with("\\n") {
                                            let mut line: String = line.to_owned();
                                            line.pop();
                                            line.pop();
                                            println!("{line}");
                                        } else {
                                            print!("{line}");
                                        }
                                    }

                                    Ok(Expr::One)
                                }
                                "+" => {
                                    let mut sum = 0;

                                    for arg in args {
                                        let Expr::Int(n) = arg else {
                                            // #TODO proper error!
                                            return Err(EvalError::UnknownError);
                                        };
                                        sum += n;
                                    }

                                    Ok(Expr::Int(sum))
                                }
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

                                    let Annotated(Expr::Func(params, body), ..) = op else {
                                        // #TODO non-callable error!
                                        return Err(EvalError::UnknownError);
                                    };

                                    // #TODO ultra-hack to kill shared ref to `env`.
                                    let params = params.clone();
                                    let body = body.clone();

                                    // Dynamic scoping

                                    env.push_new_scope();

                                    for (param, arg) in params.iter().zip(args) {
                                        let Annotated(Expr::Symbol(param), ..) = param else {
                                            // #TODO non-callable error!
                                            return Err(EvalError::UnknownError);
                                        };

                                        env.insert(param, arg);
                                    }

                                    let result = eval(body, env);

                                    // let result = Ok(Expr::One);

                                    env.pop();

                                    result
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Err(EvalError::UnknownError);
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
