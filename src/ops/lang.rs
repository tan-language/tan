use crate::{ann::Ann, error::Error, eval::env::Env, expr::Expr, range::Range};

pub fn ann(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
    if args.len() != 1 {
        return Err(Error::invalid_arguments(
            "`ann` requires one argument",
            Range::default(),
        )); // #TODO set in caller.
    }

    // #TODO support multiple arguments.

    let _expr = args.first().unwrap();

    // #TODO aargh, no access to annotations!

    Ok(Expr::One.into())
}
