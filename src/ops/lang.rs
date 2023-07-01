use crate::{error::Error, eval::env::Env, expr::Expr};

// #TODO extract *_impl function.
pub fn ann(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    if args.len() != 1 {
        return Err(Error::invalid_arguments(
            "`ann` requires one argument",
            None,
        ));
    }

    // #TODO support multiple arguments.

    let expr = args.first().unwrap();

    if let Some(ann) = expr.annotations() {
        Ok(Expr::Dict(ann.clone()))
    } else {
        // #TODO what to return here?
        Ok(Expr::One.into())
    }
}
