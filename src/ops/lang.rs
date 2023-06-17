use crate::{ann::Ann, error::Error, eval::env::Env, expr::Expr};

pub fn ann(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
    if args.len() != 1 {
        return Err(Error::invalid_arguments(
            "`ann` requires one argument",
            None,
        ));
    }

    // #TODO support multiple arguments.

    let _expr = args.first().unwrap();

    // #TODO aargh, no access to annotations!

    Ok(Expr::One.into())
}
