pub mod env;
pub mod error;

use crate::expr::Expr;

use self::{env::Env, error::EvalError};

// tree-walk interpreter

// #TODO interpret or eval or execute?
// #TODO alternative names: Processor, Runner

// #TODO accept AsRef<Expr>
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
        Expr::List(list) => {
            // #TODO replace head/tail with first/rest
            // #TODO empty list should also be found in read/parse phase
            // #TODO could this arise in self-modifying code?
            let head = list.first().ok_or(EvalError::UnknownError)?;
            let tail = &list[1..];

            let Expr::Symbol(s) = head.as_ref() else {
                return Err(EvalError::UnknownError);
            };

            // Evaluate the arguments before calling the function.
            let mut args = Vec::new();
            for x in tail {
                // #Insight cannot use map() because of the `?` operator.
                args.push(eval(x, env)?);
            }

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
