pub mod arithmetic;
pub mod bool;
pub mod buffer;
pub mod char;
pub mod chrono;
pub mod cmp;
pub mod css_expr;
pub mod dec;
pub mod eq;
pub mod error;
pub mod float;
pub mod fs;
pub mod int;
pub mod io;
pub mod json;
pub mod lang;
pub mod map;
pub mod math;
pub mod maybe;
pub mod path;
pub mod prelude;
pub mod process;
pub mod range;
pub mod seq;
pub mod set;
pub mod string;
pub mod testing;
pub mod u8;

use dec::setup_lib_dec;

use crate::context::Context;

use self::{
    chrono::setup_lib_chrono, css_expr::setup_lib_css_expr, fs::setup_lib_fs,
    json::setup_lib_codec_json, math::setup_lib_math, path::setup_lib_path,
    prelude::setup_lib_prelude, process::setup_lib_process, set::setup_lib_set,
};

// #todo consider extracting as a (temporary?) crate, e.g. tan-stdlib-native, tan-native-lib, tan-runtime
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

// #insight special-form != primitive ? special form requires special case in the interpreter?

// #todo consider removing the `std` prefix from module paths, like haskell.
// #todo find a better prefix than setup_
// #todo use Rc/Arc consistently
// #todo some helpers are needed here, to streamline the code.

// #todo only setup non-prelude libs on demand!pub mod http_server;

// #todo call the foreign setup from the actual tan module file.

pub fn setup_lib(context: &mut Context) {
    setup_lib_fs(context);
    setup_lib_process(context);
    setup_lib_css_expr(context);
    setup_lib_codec_json(context);
    setup_lib_math(context);
    setup_lib_chrono(context);
    // setup_lib_testing(context);
    setup_lib_path(context);
    setup_lib_set(context);
    setup_lib_dec(context);

    setup_lib_prelude(context);
}
