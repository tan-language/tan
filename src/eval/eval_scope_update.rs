use crate::{
    context::Context,
    error::Error,
    expr::{expr_clone, Expr},
};

use super::eval;

// #todo add unit-test
// #todo this name conflicts with scope.update()
// #todo consider a function that nests a new scope.
// #todo maybe it should be some kind of let? e.g. (`let-all` ?? or `let*` or `let..`)
// #todo this is a temp hack.

/// Updates the scope with bindings of the given map.
/// (scope-update ...)
pub fn eval_scope_update(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let [map] = args else {
        return Err(Error::invalid_arguments("malformed `scope-update`", None));
    };

    let map = eval(map, context)?;

    let Some(map) = map.as_map() else {
        // #todo report what was found!
        return Err(Error::invalid_arguments(
            "malformed `scope-update`, expects Map argument",
            None,
        ));
    };

    for (name, value) in map.iter() {
        // #todo remove clone.
        context.scope.insert(name, expr_clone(value));
    }

    Ok(Expr::None)
}
