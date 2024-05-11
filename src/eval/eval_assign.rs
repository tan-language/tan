use crate::{context::Context, error::Error, expr::Expr};

use super::eval;

// #insight
// This is not the same as let, it also traverses the scope stack to find
// bindings to update in parent scopes.

// #insight
// In the past this function was called `set!`. It was renamed to `assign` to
// avoid ambiguity with mathematical sets, and to move away from the (too) noisy
// trailing `!` convention.

// #insight
// Don't use `poke` for this, reseve peek and poke.

// #insight
// Maybe the full `assign` name should be recommended, to add more friction.

// #insight
// Originally we the operator `:=` was used as an alias for assignment, like
// Pascal and Go. However, the `:=` conflicts with key-symbols :(
// So the operator `<-` is used instead, like R, Math, etc. The `<-` is somehow
// related with the `->` map/function operator.

pub fn eval_assign(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo this is a temp hack
    // #todo write unit tests
    // #todo support mutating multiple variables.

    let [name, value] = args else {
        return Err(Error::invalid_arguments("malformed `assign`", None));
    };

    let Some(name) = name.as_stringable() else {
        return Err(Error::invalid_arguments(
            "requires a symbol as the first argument",
            name.range(),
        ));
    };

    let value = eval(value, context)?;

    // #todo should we check that the symbol actually exists?
    context.scope.update(name, value.clone());

    // #todo what should this return? One/Unit (i.e. nothing useful) or the actual value?
    Ok(Expr::Nil)
}
