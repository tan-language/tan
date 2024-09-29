// #note #WARNING This is actually not used, look into eval/eval_assertions.

// // #todo why rust implements assert as a macro?
// // #todo also provide 'debug' version of assert that is stripped in prod builds
// // #todo easier to implement with Tan code?
// // #todo no need for #test annotation, at least initially, just scan for *.test.tan extension and explicitly call the test functions
// // #todo have a separate module with 'runtime' asserts, e.g. called just `asserts`.

// // #todo assert
// // #todo assert-eq
// // #todo assert-not-eq
// // #todo assert-is-matching
// // #todo assert-is-error
// // #todo assert-is-panic

// // #todo support optional message?

// use crate::{context::Context, error::Error, expr::Expr};

// // #todo move assert to prelude?

// // #todo should implement as a macro, this is a temp solution.
// // #todo is `predicate` a good name?
// pub fn assert(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
//     let [predicate] = args else {
//         return Err(Error::invalid_arguments(
//             "requires `predicate` argument",
//             None,
//         ));
//     };

//     let Some(predicate) = predicate.as_bool() else {
//         return Err(Error::invalid_arguments(
//             &format!("`{}` is not a Bool", predicate.unpack()),
//             predicate.range(),
//         ));
//     };

//     if predicate {
//         Ok(Expr::Bool(true))
//     } else {
//         if let Some(value) = context.get("*test-failures*", true) {
//             if let Some(mut failures) = value.as_array_mut() {
//                 failures.push(Expr::string(format!("{predicate} failed")));
//             }
//         }
//         Ok(Expr::Bool(false))
//     }
// }

// // #todo we need the call-side position.
// pub fn assert_eq(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
//     // #todo need to implement method dispatching here!

//     // #todo for the moment only supports int!
//     let name = "=$$Int$$Int";

//     let func = context.scope.get(name).unwrap();
//     let func = func.unpack();
//     let Expr::ForeignFunc(func) = func else {
//         panic!("unexpected error");
//     };

//     // #insight args are pre-evaluated, no need for eval_args.
//     let result = func(args, context);

//     let Ok(result) = result else {
//         return result;
//     };

//     let Expr::Bool(b) = result else {
//         panic!("unexpected error");
//     };

//     if !b {
//         // #todo give exact details about that failed!
//         return Err(Error::general("assertion failed"));
//     }

//     // #todo how to report the assertion? no panic in test mode.

//     Ok(result)
// }

// pub fn setup_lib_testing(_context: &mut Context) {
//     // #todo at the moment we use the assert and assert-eq special forms.

//     // let module = require_module("testing", context);

//     // module.insert_invocable("assert", Expr::ForeignFunc(Arc::new(assert)));
//     // module.insert_invocable("assert-eq", Expr::ForeignFunc(Arc::new(assert_eq)));
// }

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn assert_eq_usage() {
//         // #todo
//     }
// }
