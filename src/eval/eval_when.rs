use crate::{
    context::Context,
    error::Error,
    expr::{has_dyn_type, Expr},
    util::args::unpack_arg,
};

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

    let value = eval(value, context)?;

    let mut i = 1;

    // #todo use args as iterator.
    // #todo #IMPORTANT should create nested context!

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

        match pattern {
            Expr::Symbol(sym) => {
                if sym == "_" {
                    break eval(clause, context);
                }
            }
            Expr::Type(type_name) => {
                if has_dyn_type(&value, type_name, context) {
                    break eval(clause, context);
                }
            }
            Expr::List(terms) => {
                // #todo pattern matching needs to be recursive!
                // #todo #temp manual, dummy implementation.
                // #todo check () -> None!
                match value.unpack() {
                    Expr::Int(n) => {
                        // #todo extract as function.
                        if let Some(typ) = terms[0].as_stringable() {
                            if typ == "Int" {
                                if let Some(name) = terms[1].as_stringable() {
                                    if name != "_" {
                                        // #todo #IMPORTANT should create nested context.
                                        context.scope.insert(name, Expr::Int(*n));
                                    }
                                } // #todo raise error.
                                break eval(clause, context);
                            }
                        }
                    }
                    Expr::Float(_n) => todo!(),
                    Expr::Error(reason) => {
                        // #todo extract as function.
                        if let Some(typ) = terms[0].as_stringable() {
                            if typ == "Error" {
                                if let Some(name) = terms[1].as_stringable() {
                                    if name != "_" {
                                        // #todo #IMPORTANT should create nested context.
                                        context.scope.insert(name, Expr::string(reason));
                                    }
                                } // #todo raise error.
                                break eval(clause, context);
                            }
                        }
                    }
                    _ => {}
                }
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

// #todo also add tan tests.

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

        let input = r#"
        (let value ())
        (when value
            None "nothing"
            _    "unknown"
        )
        "#;
        let value = eval_string(input, &mut context).unwrap();
        assert_eq!(format_value(&value), "nothing");

        let input = r#"
        (let value 5)
        (when value
            (Int n) "integer: ${n}"
                  _ "unknown" ; <-- #insight interesting formatting!
        )
        "#;
        let value = eval_string(input, &mut context).unwrap();
        assert_eq!(format_value(&value), "integer: 5");

        let input = r#"
        (let result (Error "invalid value"))
        (when result
            (Error reason)
                "error: ${reason}"
            _   "OK"
        )
        "#;
        let value = eval_string(input, &mut context).unwrap();
        assert_eq!(format_value(&value), "error: invalid value");
    }
}
