pub mod arithmetic;
pub mod chrono;
pub mod cmark;
pub mod cmp;
pub mod css_expr;
pub mod dict;
pub mod eq;
pub mod fs;
pub mod html;
pub mod io;
pub mod json;
pub mod lang;
pub mod math;
pub mod network;
pub mod prelude;
pub mod process;
pub mod rng;
pub mod seq;
pub mod string;

use crate::context::Context;

use self::{
    chrono::setup_lib_chrono, cmark::setup_lib_text_cmark, css_expr::setup_lib_css_expr,
    fs::setup_lib_fs, html::setup_lib_html, json::setup_lib_codec_json,
    network::http::setup_lib_http, prelude::setup_lib_prelude, process::setup_lib_process,
    rng::setup_lib_rand,
};

// #todo consider extracting as a (temporary?) crate, e.g. tan-stdlib-native, tan-native-lib, tan-runtime
// #todo find
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

pub fn setup_lib(context: &mut Context) {
    setup_lib_fs(context);
    setup_lib_process(context);
    setup_lib_rand(context);
    setup_lib_html(context);
    setup_lib_text_cmark(context);
    setup_lib_css_expr(context);
    setup_lib_codec_json(context);
    setup_lib_http(context);
    setup_lib_chrono(context);

    setup_lib_prelude(context);
}
