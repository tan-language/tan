use crate::{context::Context, error::Error, expr::Expr};

use super::eval;

// #insight
// this is not the same as let, it also traverses the scope stack to find bindings to
// update in parent scopes.

pub fn eval_set(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo find other name: poke, mut, mutate
    // #todo this is a temp hack
    // #todo write unit tests
    // #todo support mutating multiple variables.

    let [name, value] = args else {
        return Err(Error::invalid_arguments("malformed `set!`", None));
    };

    let Some(name) = name.as_stringable() else {
        return Err(Error::invalid_arguments(
            "`set!` requires a symbol as the first argument",
            name.range(),
        ));
    };

    let value = eval(value, context)?;

    // #todo should we check that the symbol actually exists?
    context.scope.update(name, value.clone());

    // #todo what should this return? One/Unit (i.e. nothing useful) or the actual value?
    Ok(Expr::Nil)
}
