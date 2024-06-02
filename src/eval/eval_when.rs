use crate::{context::Context, error::Error, expr::Expr, util::args::unpack_arg};

use super::eval;

// #todo IMPLEMENT ME

// #todo is this different enough from `if`?
// #todo replace `cond` with `if`?
// #todo introduce `when` and `unless`?

// #insight
// The `when` keyword is inspired by Kotlin. The more common `match`, `switch`,
// and `case` keywords are not used to avoid reserving useful nouns.

// #todo consider (else ...) clause.

// (when value
//   (Int n) (...)
//   (Float n) (...)
//   _ (...)
// )
pub fn eval_when(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let value = unpack_arg(args, 0, "value")?;

    let mut i = 1;

    // #todo use args as iterator.

    // #insight
    // All cases should be handled, at least with a 'wildcard' `_` clause.
    // Otherwise return Never/Zero or actually panic!

    loop {
        if i >= args.len() {
            // #todo add a static, comp-time check for this!
            // #insight
            // All cases _must_ be handled, at least with a 'wildcard' `_` clause.
            // If the execution ended here, a case is missing, panic!
            // return Err(Error::panic("unhandled `when` case"));
            // #todo for the moment we don't panic, revisit this!
            return Ok(Expr::Never);
        }

        let pattern = unpack_arg(args, i, "pattern")?;
        let clause = unpack_arg(args, i + 1, "clause")?;

        // #todo handle else close!

        // if let Expr::Symbol(sym) = predicate.unpack() {
        //     if sym == "else" {
        //         break eval(clause, context);
        //     }
        // }

        // let predicate = eval(predicate, context)?;

        // let Some(predicate) = predicate.as_bool() else {
        //     return Err(Error::invalid_arguments(
        //         "the cond predicate is not a boolean value",
        //         predicate.range(),
        //     ));
        // };

        let predicate = true;

        if predicate {
            break eval(clause, context);
        }

        i += 2;
    }
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    // #insight `+<-` and other assignment operators are expanded in macro_expand.
    // #insight `plus` is a more general name than `add` for the operator.
    #[test]
    fn eval_assign_plus_usage() {}
}
