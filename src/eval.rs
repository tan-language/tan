pub mod env;
pub mod error;

use crate::{ann::Annotated, expr::Expr};

use self::{env::Env, error::EvalError};

// tree-walk interpreter

// #TODO interpret or eval or execute?
// #TODO alternative names: Processor, Runner

pub fn eval(expr: impl AsRef<Expr>, env: &mut Env) -> Result<Expr, EvalError> {
    let expr = expr.as_ref();
    let result = match expr {
        Expr::Do(list) => {
            let mut result = Ok(Expr::One);
            for expr in list {
                result = eval(expr, env)
            }
            result
        }
        Expr::Symbol(s) => {
            let result = env.get(s);

            let Some(Annotated(expr, ..)) = result else {
                // #TODO proper error!
                return Err(EvalError::UnknownError);
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

            let Expr::Symbol(s) = head.as_ref() else {
                return Err(EvalError::UnknownError);
            };

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
                "let" => {
                    let mut args = args.into_iter();

                    loop {
                        let Some(sym) = args.next() else {
                            break;
                        };
                        let Some(value) = args.next() else {
                            // #TODO error?
                            break;
                        };
                        let Expr::Symbol(s) = sym else {
                            // #TODO proper error!
                            return Err(EvalError::UnknownError);
                        };
                        // #TODO notify about overrides? use `set`?
                        env.insert(s, value);
                    }

                    // #TODO return last value!
                    Ok(Expr::One)
                }
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
                    return Err(EvalError::UnknownError);
                }
            }
        }
        _ => {
            // Unhandled expression variants evaluate to themselves.
            return Ok(expr.clone());
        }
    };

    result
}
