use crate::context::Context;

use super::arithmetic::setup_lib_arithmetic;
use super::cmp::setup_lib_cmp;
use super::dict::setup_lib_dict;
use super::eq::setup_lib_eq;
use super::io::setup_lib_io;
use super::seq::setup_lib_seq;
use super::string::setup_lib_string;

// #todo instead of evaluating in prelude maybe it's better to use the functions from the actual modules?
pub fn setup_lib_prelude(context: &mut Context) {
    // #todo maybe context.require_module(path) instead?

    setup_lib_arithmetic(context);
    setup_lib_eq(context);
    setup_lib_cmp(context);
    setup_lib_io(context);
    setup_lib_string(context);
    setup_lib_seq(context);
    setup_lib_dict(context);
}
