use std::sync::Arc;

use crate::{
    context::Context,
    error::{Error, ErrorVariant},
    expr::Expr,
    scope::Scope,
};

use super::{eval, insert_binding, iterator::try_iterator_from};

// #insight
// `while` is a generalization of `if`
// `for` is a generalization of `let`
// `for` is related with `do`
// `for` is monadic

// #todo check racket.
// #todo implement for->list, for->map, for->fold, etc.

// #todo what should happen if variable source is nil?

// (for [x 10] (writeln x))
pub fn eval_for(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo reuse code from let
    // #todo the resolver should handle this.

    if args.len() < 2 {
        // #todo add more structural checks.
        // #todo proper error!
        return Err(Error::invalid_arguments("missing for arguments", None));
    }

    let binding = args.first().unwrap();
    let body = &args[1..];

    // #todo should check both for list and array (i.e. as_iterable)
    let Some(binding_parts) = binding.as_array() else {
        // #todo proper error!
        return Err(Error::invalid_arguments(
            "invalid for binding",
            binding.range(),
        ));
    };

    // #todo support _multiple_ bindings.
    let [var, value] = &binding_parts[..] else {
        return Err(Error::invalid_arguments(
            "invalid for binding",
            binding.range(),
        ));
    };

    // #insight for the ListIterator
    let value = eval(value, context)?;

    // #todo also handle (Range start end step)
    // #todo maybe step should be external to Range, or use SteppedRange, or (Step-By (Range T))
    let Some(iterator) = try_iterator_from(&value) else {
        // #todo proper error!
        return Err(Error::invalid_arguments(
            &format!("invalid for binding, `{value}` is not iterable"),
            value.range(),
        ));
    };

    let prev_scope = context.scope.clone();
    context.scope = Arc::new(Scope::new(prev_scope.clone()));

    let mut iterator = iterator.borrow_mut();

    'outer_loop: while let Some(value) = iterator.next() {
        insert_binding(var, value, context)?;
        'inner_loop: for expr in body {
            match eval(expr, context) {
                Err(Error {
                    variant: ErrorVariant::BreakCF(_value),
                    ..
                }) => {
                    // #todo for the moment we ignore break with value, should think some more about it.
                    break 'outer_loop;
                }
                Err(Error {
                    variant: ErrorVariant::ContinueCF,
                    ..
                }) => {
                    break 'inner_loop;
                }
                Err(error) => {
                    // #todo add unit test to catch for-error regression.
                    // Propagate all other errors. This is very ..error-prone code, think how
                    // to refactor.
                    return Err(error);
                }
                _ => {
                    // #insight plain `for` is useful only for the side-effects, ignore the value.
                    // #todo maybe it should return the last value?
                }
            }
        }
    }

    // #todo what happens to this if an error is thrown?
    context.scope = prev_scope;

    Ok(Expr::None)
}
