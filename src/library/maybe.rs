use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::{expr_clone, Expr},
    util::{
        args::{unpack_arg, unpack_stringable_arg},
        module_util::require_module,
    },
};

// #todo implement all these with Tan.

// #todo rename to nil?/is-nil?
// #insight with dynamic typing you don't strictly need a Maybe type?
pub fn is_none(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [expr] = args else {
        // #todo better error
        return Err(Error::invalid_arguments("one argument expected", None));
    };

    Ok(Expr::Bool(expr.is_none()))
}

pub fn is_some(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [expr] = args else {
        // #todo better error
        return Err(Error::invalid_arguments("one argument expected", None));
    };

    Ok(Expr::Bool(!expr.is_none()))
}

// #todo #IMPORTANT this should be a special form, not evaluate the default value if not needed (short-circuit).
// #todo implement with tan!
pub fn some_or(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let expr = unpack_arg(args, 0, "expr")?;

    if expr.is_none() {
        let fallback = unpack_arg(args, 1, "fallback")?;
        // #todo #optimize remove the fucking clone.
        // #todo consider ForeignFunction variant that returns Result<&Expr, Error>
        // #todo or maybe a macro can consume expressions?
        return Ok(expr_clone(fallback));
    }

    // #todo the nasty clone again.
    Ok(expr_clone(expr))
}

pub fn expect(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let expr = unpack_arg(args, 0, "expr")?;

    // #todo Should handle proper Maybe type.
    if expr.is_none() {
        let text = unpack_stringable_arg(args, 1, "text")?;
        let text = format!("Failed expect: {text}");
        // #todo Improve reporting of failed expect.s
        // #todo Introduce throw_panic helper.
        return Err(Error::panic_with_context(&text, context));
    }

    // #todo #fixme the nasty clone again.
    Ok(expr_clone(expr))
}

pub fn setup_lib_maybe(context: &mut Context) {
    //. #todo move to `maybe` namespace?
    let module = require_module("prelude", context);

    // #insight Use `is-some?` instead of `some?` to make it a verb, `is` is a linking verb.
    // (if (is-some? user) ...)
    module.insert("is-some?", Expr::ForeignFunc(Arc::new(is_some)));
    module.insert("is-none?", Expr::ForeignFunc(Arc::new(is_none)));
    module.insert("some-or", Expr::ForeignFunc(Arc::new(some_or)));
    module.insert("expect", Expr::ForeignFunc(Arc::new(expect)));
}

// #todo add unit tests!

// #info test are added to root/@std/maybe/maybe.test.tan

// #[cfg(test)]
// mod tests {

//     #[test]
//     fn some_or_usage() {}
// }
