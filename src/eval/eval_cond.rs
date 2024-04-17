use crate::{context::Context, error::Error, expr::Expr};

use super::eval;

// #todo is this different enough from `if`?
// #todo replace `cond` with `if`?
// #todo introduce `when` and `unless`?

// (cond
//   (> i 5) (...)
//   (> i 15) (...)
//   else (...)
// )
pub fn eval_cond(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let mut i = 0;

    loop {
        if i >= args.len() {
            // #todo what should we return here? probably Never/Zero?
            break Ok(Expr::Nil);
        }

        let Some(predicate) = args.get(i) else {
            return Err(Error::invalid_arguments("malformed cond predicate", None));
        };

        let Some(clause) = args.get(i + 1) else {
            return Err(Error::invalid_arguments("malformed cond clause", None));
        };

        // #todo `else` should not be annotated.
        // #todo should NOT annotate symbols and keysymbols!
        // #todo introduce a helper to check for specific symbol.

        if let Expr::Symbol(sym) = predicate.unpack() {
            if sym == "else" {
                break eval(clause, context);
            }
        }

        let predicate = eval(predicate, context)?;

        let Some(predicate) = predicate.as_bool() else {
            return Err(Error::invalid_arguments(
                "the cond predicate is not a boolean value",
                predicate.range(),
            ));
        };

        if predicate {
            break eval(clause, context);
        }

        i += 2;
    }
}
