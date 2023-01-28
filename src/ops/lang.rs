use crate::{ann::Ann, error::Error, eval::env::Env, expr::Expr, range::Ranged};

pub fn ann(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
    if args.len() != 1 {
        return Err(Error::invalid_arguments("`ann` requires one argument").into());
    }

    // #TODO support multiple arguments.

    let _expr = args.first().unwrap();

    // #TODO aargh, no access to annotations!

    Ok(Expr::One.into())
}
