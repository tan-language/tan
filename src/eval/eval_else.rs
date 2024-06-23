use crate::{context::Context, error::Error, expr::Expr, util::args::unpack_arg};

use super::{eval, eval_do::eval_do};

pub fn eval_else(_args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    todo!()
}
