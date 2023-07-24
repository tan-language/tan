use crate::{context::Context, error::Error, expr::Expr};

// #todo extract *_impl function.
pub fn ann(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    if args.len() != 1 {
        return Err(Error::invalid_arguments(
            "`ann` requires one argument",
            None,
        ));
    }

    // #todo support multiple arguments.

    let expr = args.first().unwrap();

    if let Some(ann) = expr.annotations() {
        Ok(Expr::Dict(ann.clone()))
    } else {
        // #todo what to return here?
        Ok(Expr::One.into())
    }
}
