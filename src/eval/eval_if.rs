use crate::{context::Context, error::Error, expr::Expr};

use super::eval;

pub fn eval_if(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo this is a temp hack!
    let Some(predicate) = args.first() else {
        return Err(Error::invalid_arguments("malformed if predicate", None));
    };

    let Some(true_clause) = args.get(1) else {
        return Err(Error::invalid_arguments("malformed if true clause", None));
    };

    // #todo don't get false_clause if not required?
    let false_clause = args.get(2);

    let predicate = eval(predicate, context)?;

    let Some(predicate) = predicate.as_bool() else {
        return Err(Error::invalid_arguments(
            "the if predicate is not a boolean value",
            predicate.range(),
        ));
    };

    if predicate {
        eval(true_clause, context)
    } else if let Some(false_clause) = false_clause {
        eval(false_clause, context)
    } else {
        // #insight In the Curry–Howard correspondence, an empty type corresponds to falsity.
        // #insight
        // Zero / Never disallows this:
        // (let flag (if predicate (+ 1 2))) ; compile error: cannot assign Never
        Ok(Expr::Never)
    }
}
