// #todo why rust implements assert as a macro?
// #todo also provide 'debug' version of assert that is stripped in prod builds
// #todo easier to implement with Tan code?
// #todo no need for #test annotation, at least initially, just scan for *.test.tan extension and explicitly call the test functions

// #todo assert
// #todo assert-eq
// #todo assert-not-eq
// #todo assert-is-matching
// #todo assert-is-error
// #todo assert-is-panic

// #todo support optional message?

use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

pub fn assert_eq(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo need to implement method dispatching here!

    // #todo for the moment only supports int!
    let name = "=$$Int$$Int";

    let func = context.scope.get(name).unwrap();
    let func = func.unpack();
    let Expr::ForeignFunc(func) = func else {
        panic!("unexpected error");
    };

    // #insight args are pre-evaluated, no need for eval_args.
    let result = func(args, context);

    let Ok(result) = result else {
        return result;
    };

    let Expr::Bool(b) = result else {
        panic!("unexpected error");
    };

    if !b {
        return Err(Error::general("assertion failed"));
    }

    // #todo how to report the assertion? no panic in test mode.

    Ok(result)
}

pub fn setup_lib_testing(context: &mut Context) {
    let module = require_module("testing", context);

    module.insert("assert-eq", Expr::ForeignFunc(Arc::new(assert_eq)));
}

#[cfg(test)]
mod tests {
    #[test]
    fn assert_eq_usage() {
        // #todo
    }
}
