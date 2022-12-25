pub mod env;
pub mod error;

use crate::{ann::Annotated, expr::Expr};

use self::{env::Env, error::EvalError};

// #Insight
// _Not_ a pure evaluator, performs side-effects.

// #TODO encode effects in the type-system.
// #TODO interpret or eval or execute?
// #TODO alternative names: Processor, Runner

/// Evaluates via expression rewriting. The expression `expr` evaluates to
/// a fixed point. In essence this is a 'tree-walk' interpreter.
pub fn eval(expr: impl AsRef<Expr>, env: &mut Env) -> Result<Expr, EvalError> {
    let expr = expr.as_ref();
    let result = match expr {
        Expr::Symbol(sym) => {
            let result = env.get(sym);

            let Some(Annotated(expr, ..)) = result else {
                return Err(EvalError::UndefinedSymbol(sym.clone()));
            };

            // #TODO hm, can we somehow work with references?
            Ok(expr.clone())
        }
        Expr::List(list) => {
            // #TODO replace head/tail with first/rest
            // #TODO empty list should also be found in read/parse phase
            // #TODO could this arise in self-modifying code?
            // #TODO also eval the head?
            let head = list.first().ok_or(EvalError::UnknownError)?;
            let tail = &list[1..];

            match head.as_ref() {
                Expr::Do => {
                    // #TODO do should be 'monadic', propagate Eff (effect) wrapper.
                    let mut result = Expr::One;
                    for expr in tail {
                        result = eval(expr, env)?;
                    }
                    return Ok(result);
                }
                Expr::Symbol(s) => {
                    // Special forms

                    #[allow(clippy::single_match)]
                    match s.as_str() {
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
                                // #TODO notify about overrides? use `set`?
                                env.insert(s, value.clone());
                            }

                            // #TODO return last value!
                            return Ok(Expr::One);
                        }
                        _ => (),
                    }

                    // Evaluate the arguments before calling the function.
                    let mut args = Vec::new();
                    for x in tail {
                        // #Insight cannot use map() because of the `?` operator.
                        args.push(eval(x, env)?);
                    }

                    // Functions

                    match s.as_str() {
                        "write" => {
                            let output = args.iter().fold(String::new(), |mut str, x| {
                                str.push_str(&format!("{}", x));
                                str
                            });

                            println!("{output}");

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
                        _ => {
                            return Err(EvalError::UndefinedSymbol(s.clone()));
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
