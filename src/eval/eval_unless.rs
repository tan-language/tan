use crate::{context::Context, error::Error, expr::Expr, util::args::unpack_arg};

use super::{eval, eval_do::eval_do};

// #todo Somehow mark that this is lazy evaluation.
pub fn eval_unless(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #insight If is not comp-time.
    // #insight Cannot use unpack_bool_arg, this has lazy evaluation.
    // #todo Is the name `predicate` relevant here?
    let predicate = unpack_arg(args, 0, "predicate")?;

    let predicate = eval(predicate, context)?;

    let Some(predicate) = predicate.as_bool() else {
        return Err(Error::invalid_arguments(
            "the predicate is not a boolean value",
            predicate.range(),
        ));
    };

    let body = &args[1..];

    // #todo Make (else ...) raise error if not in the last position of if/else.

    let else_clause = if let Some(last_clause) = body.last() {
        if let Some(last_clause) = last_clause.as_list() {
            if last_clause.len() > 1 {
                if let Some(op) = last_clause.first().unwrap().as_symbol() {
                    if op == "else" {
                        Some(last_clause)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let body = if else_clause.is_some() {
        // Remove the else_clause from the main_clause.
        &body[..(body.len() - 1)]
    } else {
        body
    };

    let value = if !predicate {
        // #todo Extract common code between this, do, for, etc.
        eval_do(body, context)?
    } else if let Some(else_clause) = else_clause {
        // #insight Note that (else ...) is like (do ...)!
        eval_do(&else_clause[1..], context)?
    } else {
        // #todo What should be the return value?
        Expr::None
    };

    Ok(value)
}
