use crate::{context::Context, error::Error, expr::Expr};

use super::eval;

// #todo pass-through the annotations.

// #warning still researching the final state between def and let.

pub fn eval_def(op: &Expr, args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo there is currently no resolver, duh.
    // #todo actually some resolving is happening in macro_expand, e.g. checking for binding values.
    // #todo this is already parsed statically by resolver, no need to duplicate the tests here?
    // #todo also report some of these errors statically, maybe in a sema phase?
    // #todo use 'location' or 'lvalue' instead of name?

    // #insight 'pass-through' let annotations, only for ...def.

    // #insight def does not support multiple definitions
    // #todo consider supporting multiple definitions with [...] syntax.
    // #todo align `def` with `use`.

    // loop {

    let Some(name_expr) = args.first() else {
        return Err(Error::invalid_arguments("missing binding name", None));
    };

    let Some(value) = args.get(1) else {
        return Err(Error::invalid_arguments("missing binding value", None));
    };

    if args.len() > 2 {
        return Err(Error::invalid_arguments(
            "malformed def with multiple bindings",
            None,
        ));
    }

    let value = Expr::maybe_annotated(eval(value, context)?, op.annotations());

    // #todo insert the binding into the current module/namespace, not the current scope!
    // #todo maybe current scope is good though?

    // #insight def does not allow destructuring
    // #todo reconsider this? like Scheme define?

    let Some(name) = name_expr.as_stringable() else {
        return Err(Error::invalid_arguments(
            "malformed def: name must be Stringable",
            name_expr.range(),
        ));
    };

    if context.scope.contains_name(name) {
        // #insight
        // One important difference between `def` and `let` is that `def`
        // does not allow shadowing.
        // #todo use a custom Error variant.
        return Err(Error::invalid_arguments(
            &format!("`{name_expr}` is already defined"),
            name_expr.range(),
        ));
    }

    context.scope.insert(name, value);

    // }

    // #todo return last value, it would require some cloning currently.
    Ok(Expr::None)
}

// #todo add unit tests!
