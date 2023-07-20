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

// #TODO helper function or macro for arithmetic operations!
// #TODO also eval 'if', 'do', 'for' and other keywords here!
// #TODO use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
// #TODO support overloading,
// #TODO make equality a method of Expr?
// #TODO support non-Int types
// #TODO support multiple arguments.
// #TODO helper function or macro for arithmetic operations!

// #TODO primitives, builtins
// #TODO cannot be implemented with Expr::ForeignFunc as the args are pre-evaluated.

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
