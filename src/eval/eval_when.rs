use crate::{context::Context, error::Error, expr::Expr, util::args::unpack_arg};

use super::eval;

// #todo should reserve `when` for one-leg if? (inverse of `unless`)?

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
    // #todo can use returns instead of breaks.

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

        println!("===== {pattern}");

        match pattern {
            Expr::Symbol(sym) => {
                println!("--1 Symbol");
                println!("--2");
                if sym == "_" {
                    break eval(clause, context);
                }
            }
            Expr::List(terms) => {
                println!("--1 List");
                println!("--2");
            }
            _ => {
                // #todo what is the correct error to return?
                return Err(Error::panic("unhandled `when` case"));
            }
        }

        // #todo handle else clause!

        i += 2;
    }
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn eval_when_usage() {
        let mut context = Context::new();

        let input = r#"
        (let value "not-int")
        (when value
            (Int n) "integer: ${n}"
            _       "unknown"
        )
        "#;
        let value = eval_string(input, &mut context).unwrap();
        assert_eq!(format_value(&value), "unknown");
    }
}
