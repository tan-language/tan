use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::args::{unpack_arg, unpack_bool_arg},
};

use super::eval;

// #todo Somehow mark that this is lazy evaluation.
pub fn eval_unless(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #insight If is not comp-time.
    // #insight Cannot use unpack_bool_arg, this has lazy evaluation.
    // #todo Is the name `predicate` relevant here?
    let predicate = unpack_arg(args, 0, "predicate")?;

    let predicate = eval(predicate, context)?;

    let Some(predicate) = predicate.as_bool() else {
        return Err(Error::invalid_arguments(
            "the predicate is not a boolean value",
            predicate.range(),
        ));
    };

    // #todo check for else clause.

    if !predicate {
        let body = &args[1..];
        // #todo eval all exprs!
        // #todo support return?
        for expr in body {
            eval(expr, context)?;
        }
        // #todo What should be the return value?
        Ok(Expr::Never)
    } else {
        // #todo check for else clause.
        Ok(Expr::Never)
    }
}
