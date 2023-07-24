pub mod arithmetic;
pub mod eq;
pub mod fs;
pub mod io;
pub mod lang;
pub mod math;
pub mod prelude;
pub mod process;
pub mod rng;
pub mod seq;
pub mod string;

use crate::context::Context;

use self::{
    fs::setup_std_fs, prelude::setup_std_prelude, process::setup_std_process, rng::setup_std_rand,
};

// #todo add unit test for the foreign-functions.

// #todo consider extracting builtins.

// #todo helper function or macro for arithmetic operations!
// #todo also eval 'if', 'do', 'for' and other keywords here!
// #todo use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
// #todo support overloading,
// #todo make equality a method of Expr?
// #todo support non-Int types
// #todo support multiple arguments.
// #todo helper function or macro for arithmetic operations!

// #todo primitives, builtins
// #todo cannot be implemented with Expr::ForeignFunc as the args are pre-evaluated.

// #Insight special-form != primitive ? special form requires special case in the interpreter?

// #todo consider removing the `std` prefix from module paths, like haskell.
// #todo find a better prefix than setup_
// #todo use Rc/Arc consistently
// #todo some helpers are needed here, to streamline the code.

pub fn setup_std(context: &mut Context) {
    setup_std_fs(context);
    setup_std_process(context);
    setup_std_rand(context);
    setup_std_prelude(context);
}
