use crate::{ann::Ann, error::Error, eval::env::Env, expr::Expr, range::Ranged};

/// Returns a char iterable for the chars in the string.
pub fn chars(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
    todo!();
}
