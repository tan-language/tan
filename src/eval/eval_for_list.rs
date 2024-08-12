use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, scope::Scope};

use super::{eval, iterator::try_iterator_from};

// #insight
// `while` is a generalization of `if`
// `for` is a generalization of `let`
// `for` is related with `do`
// `for` is monadic

// #todo Unify with eval_for.
// #todo Document what it does.
// #todo consider the name `for*` or something similar?
// #todo solve duplication between for and for->list
// #todo reuse code from let
// #todo the resolver should handle this.
pub fn eval_for_list(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    if args.len() < 2 {
        // #todo add more structural checks.
        // #todo proper error!
        return Err(Error::invalid_arguments(
            "missing for->list arguments",
            None,
        ));
    }

    let mut values = Vec::new();

    let binding = args.first().unwrap();
    let body = &args[1..];

    // #todo should be as_array to match `for`.
    // #todo should check both for list and array.
    let Some(binding_parts) = binding.as_array() else {
        // #todo proper error!
        return Err(Error::invalid_arguments(
            "invalid for->list binding, not an array",
            binding.range(),
        ));
    };

    let [var, value] = &binding_parts[..] else {
        return Err(Error::invalid_arguments(
            "invalid for->list binding",
            binding.range(),
        ));
    };

    let Some(var) = var.as_symbol() else {
        // #todo proper error!
        return Err(Error::invalid_arguments(
            "invalid for->list binding, malformed variable",
            var.range(),
        ));
    };

    // #insight for the ListIterator
    let value = eval(value, context)?;

    // #todo also handle (Range start end step)
    // #todo maybe step should be external to Range, or use SteppedRange, or (Step-By (Range T))
    let Some(iterator) = try_iterator_from(&value) else {
        // #todo proper error!
        return Err(Error::invalid_arguments(
            "invalid for-list binding, the value is not iterable",
            value.range(),
        ));
    };

    let prev_scope = context.scope.clone();
    context.scope = Arc::new(Scope::new(prev_scope.clone()));

    let mut iterator = iterator.borrow_mut();

    while let Some(value) = iterator.next() {
        context.scope.insert(var, value);
        for expr in body {
            values.push(eval(expr, context)?);
        }
    }

    context.scope = prev_scope;

    Ok(Expr::array(values))
}
