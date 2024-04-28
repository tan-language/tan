// #todo implement with Tan
// #todo consider putting in Prelude.
// #todo complex numbers, implement just enough to render mandelbrot.

// (let z (Complex 1.0 0.3))
// (let r (* z z))

use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::{annotate_type, has_type_annotation, Expr},
    util::{expect_lock_read, module_util::require_module},
};

// #todo how to extend to quaternions and octonions.

fn try_complex(expr: &Expr) -> Option<(f64, f64)> {
    if !has_type_annotation(expr, "Complex") {
        return None;
    }

    let Expr::Array(v) = expr.unpack() else {
        return None;
    };

    let v = expect_lock_read(v);

    // #todo use ForeignStruct.
    let re = v[0].as_float().unwrap();
    let im = v[1].as_float().unwrap();

    Some((re, im))
}

fn make_complex(re: impl Into<Expr>, im: impl Into<Expr>) -> Expr {
    let expr = Expr::array(vec![re.into(), im.into()]);
    annotate_type(expr, "Complex")
}

// (Complex)
// (Complex re)
// (Complex re im)
pub fn complex_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo consider a ForeignStruct.

    let re = args.first().unwrap_or_else(|| &Expr::Float(0.0));
    let im = args.get(1).unwrap_or_else(|| &Expr::Float(0.0));

    // #todo optimize the clones.
    Ok(make_complex(re.clone(), im.clone()))
}

pub fn complex_add(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let mut re_sum = 0.0;
    let mut im_sum = 0.0;

    for arg in args {
        let Some((re, im)) = try_complex(arg) else {
            return Err(Error::invalid_arguments(
                &format!("{arg} is not a Complex"),
                arg.range(),
            ));
        };
        re_sum += re;
        im_sum += im;
    }

    Ok(make_complex(re_sum, im_sum))
}

pub fn setup_lib_math_complex(context: &mut Context) {
    // #todo skip the `math/` prefix?
    let module = require_module("math/complex", context);

    // #todo make type-paremetric.
    // #todo better name?
    // (let z (Complex 1.0 0.3))
    module.insert("Complex", Expr::ForeignFunc(Arc::new(complex_new)));

    module.insert(
        "+$$Complex$$Complex",
        Expr::ForeignFunc(Arc::new(complex_add)),
    );

    // #todo `Complex/one`
    // #todo `Complex/zero`
    // #todo `Complex/re`
    // #todo `Complex/im`
    // #todo `(re c)`, `(re-of c)`, `(get-re c)`
    // #todo `(im c)`, `(im-of c)`, `(get-im c)`
    // #todo (* c z)
    // #todo (+ c z)
    // #todo (abs c)
}

// #todo add unit tests.
