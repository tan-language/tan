pub mod env;
pub mod error;
pub mod prelude;

use crate::{
    ann::Ann,
    expr::{format_value, Expr},
};

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
                return Err(EvalError::UndefinedSymbol(sym.clone()));
            };

            // #TODO hm, can we somehow work with references?
            Ok(expr.clone())
        }
        Expr::If(predicate, true_clause, false_clause) => {
            let predicate = eval(predicate, env)?;

            let Expr::Bool(predicate) = predicate else {
                // #TODO can we range this error?
                return Err(EvalError::InvalidArguments("the if predicate is not a boolean value".to_owned()));
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
                                return Err(EvalError::InvalidArguments("missing quote target".to_owned()));
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
                                return Err(EvalError::Unknown);
                            };

                            let mut value = Expr::One;

                            loop {
                                let predicate = eval(predicate, env)?;

                                let Expr::Bool(predicate) = predicate else {
                                    // #TODO can we range this error?
                                    return Err(EvalError::InvalidArguments("the for predicate is not a boolean value".to_owned()));
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
                                    return Err(EvalError::Unknown);
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
                                return Err(EvalError::Unknown);
                            };

                            let Ann(Expr::List(params), ..) = args else {
                                // #TODO proper error!
                                return Err(EvalError::Unknown);
                            };

                            // #TODO optimize!
                            Ok(Expr::Func(params.clone(), Box::new(body.clone())))
                        }
                        _ => {
                            // non-special term -> application.

                            // #TODO maybe delay evaluation to see if there is an actual invocable?
                            // #TODO also delay to see if the invocable is a macro or other special form?

                            // Evaluate the arguments before calling the function.
                            let args = tail
                                .iter()
                                .map(|x| eval(x, env))
                                .collect::<Result<Vec<_>, _>>()?;

                            // Try to apply an op!
                            // #Insight `op` = 'callable` (func, macro, collection, actor, etc)

                            let Some(op) = env.get(s) else {
                                return Err(EvalError::UndefinedSymbol(s.clone()));
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
                                                return Err(EvalError::Unknown);
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
                                Ann(Expr::Array(arr), ..) => {
                                    // #TODO optimize this!
                                    // #TODO error checking, one arg, etc.
                                    let Expr::Int(index) = &args[0] else {
                                        return Err(EvalError::InvalidArguments("invalid array index, expecting Int".to_string()));
                                    };
                                    let index = *index as usize;
                                    if let Some(value) = arr.get(index) {
                                        Ok(value.clone())
                                    } else {
                                        // #TODO introduce Maybe { Some, None }
                                        Ok(Expr::One)
                                    }
                                }
                                Ann(Expr::Dict(dict), ..) => {
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
                                _ => {
                                    return Err(EvalError::NotInvocable(format!("{}", head.0)));
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Err(EvalError::NotInvocable(format!("{}", head.0)));
                }
            }
        }
        _ => {
            // #TODO hm, maybe need to report an error here? or even select the desired behavior? -> NO ERROR
            // #TODO can we avoid the clone?
            // Unhandled expression variants evaluate to themselves.
            return Ok(expr.clone());
        }
    };

    result
}
