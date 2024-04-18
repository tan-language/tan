use crate::{context::Context, error::Error, expr::Expr};

use super::eval;

// #todo add unit test

// #insight
// `while` is a generalization of `if`
// `for` is a generalization of `let`
// `for` is related with `do`
// `for` is monadic

pub fn eval_while(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo
    // try to merge `while` with `for` (maybe `for` is implemented on top of `while`?)

    let [predicate, body] = args else {
        // #todo proper error!
        return Err(Error::invalid_arguments("missing for arguments", None));
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
