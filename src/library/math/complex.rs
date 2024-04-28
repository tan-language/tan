// #warning #wip not working yet!

// #todo complex numbers, implement just enough to render mandelbrot.

// (let z (Complex 1.0 0.3))
// (let r (* z z))

use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::{annotate_type, Expr},
    util::module_util::require_module,
};

// #todo how to extend to quaternions and octonions.

// (Complex)
// (Complex re)
// (Complex re im)
pub fn complex_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo consider a ForeignStruct.

    let re = args.first().unwrap_or_else(|| &Expr::Float(0.0));
    let im = args.get(1).unwrap_or_else(|| &Expr::Float(0.0));

    // #todo optimize.
    let expr = Expr::array(vec![re.clone(), im.clone()]);
    Ok(annotate_type(expr, "Complex"))
}

pub fn setup_lib_math_complex(context: &mut Context) {
    // #todo skip the `math/` prefix?
    let module = require_module("math/complex", context);

    // #todo make type-paremetric.
    // #todo better name?
    // (let z (Complex 1.0 0.3))
    module.insert("Complex", Expr::ForeignFunc(Arc::new(complex_new)));

    // #todo `Complex/one`
    // #todo `Complex/zero`
    // #todo `Complex/re`
    // #todo `Complex/im`
}
