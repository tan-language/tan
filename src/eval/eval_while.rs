use crate::{
    context::Context,
    error::{Error, ErrorVariant},
    expr::Expr,
};

use super::eval;

// #todo add unit tests

// #insight
// `while` is a generalization of `if`
// `for` is a generalization of `let`
// `for` is related with `do`
// `for` is monadic

// #todo #fix It seems that the current version of (while ...) requires a do! <-- ARGH!
pub fn eval_while(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo
    // Try to merge `while` with `for` (maybe `for` is implemented on top of `while`?)

    // #todo Improve error handling here.
    if args.len() < 2 {
        // #todo add more structural checks.
        // #todo proper error!
        return Err(Error::invalid_arguments("missing while arguments", None));
    }

    let predicate = args.first().unwrap();
    let body = &args[1..];

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

        // #todo Extract this code, maybe to (do ...), support break/continue etc?
        // #todo Improve this.
        // #todo Could return the value of the last expression?
        for expr in body {
            match eval(expr, context) {
                Err(Error {
                    variant: ErrorVariant::BreakCF(_value),
                    ..
                }) => {
                    // #todo for the moment we ignore break with value, should think some more about it.
                    break;
                }
                Err(Error {
                    variant: ErrorVariant::ContinueCF,
                    ..
                }) => {
                    continue;
                }
                Err(error) => {
                    // #todo add unit test to catch for-error regression.
                    // Propagate all other errors. This is very ..error-prone code, think how
                    // to refactor.
                    return Err(error);
                }
                _ => {
                    // #insight Plain `for`/`while` is useful only for the side-effects, ignore the value.
                    // #todo Maybe it should return the last value?
                }
            }
        }
    }

    // #todo What should be the return value of `while`?
    // Ok(value)
    Ok(Expr::None)
}

// #todo Add Tan unit-tests.
