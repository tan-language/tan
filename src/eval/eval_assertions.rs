use crate::{context::Context, error::Error, expr::Expr};

use super::{eval, util::get_current_file_path};

pub fn eval_assert(op: &Expr, args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let [assert_expr] = args else {
        return Err(Error::invalid_arguments(
            "requires `predicate` argument",
            None,
        ));
    };

    // #todo don't throw the error, include in failures!
    let predicate = eval(assert_expr, context)?;

    let Some(predicate) = predicate.as_bool() else {
        return Err(Error::invalid_arguments(
            &format!("`{}` is not a Bool", predicate.unpack()),
            predicate.range(),
        ));
    };

    if predicate {
        Ok(Expr::Bool(true))
    } else {
        if let Some(value) = context.get("*test-failures*", true) {
            if let Some(mut failures) = value.as_array_mut() {
                let file_path = get_current_file_path(context);
                let location = if let Some(range) = op.range() {
                    format!(":{}:{}", range.start.line + 1, range.start.col + 1)
                } else {
                    String::new()
                };
                failures.push(Expr::string(format!(
                    "assertion failed: {}\n  at {}{}",
                    assert_expr, file_path, location
                )));
            }
        }
        Ok(Expr::Bool(false))
    }
}
