use crate::{context::Context, error::Error, expr::Expr, util::args::unpack_symbolic_arg};

// #insight Has to be a special-form (or macro).
// #todo #think What about name vs symbol?
// #todo Consider is-bound?, python-style `locals()`/`globals()`.
pub fn eval_is_defined(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let name = unpack_symbolic_arg(args, 0, "name")?;
    Ok(Expr::Bool(context.contains_name(name)))
}

// #todo add unit tests!
