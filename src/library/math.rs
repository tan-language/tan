use crate::context::Context;

use self::complex::setup_lib_math_complex;

pub mod complex;

pub fn setup_lib_math(context: &mut Context) {
    setup_lib_math_complex(context);
}
