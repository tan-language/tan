// #warning #wip not working yet!

// #todo complex numbers, implement just enough to render mandelbrot.

// (let z (Complex 1.0 0.3))
// (let r (* z z))

pub fn complex_new(_args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo use an array for the moment.
    let set: HashSet<Expr> = HashSet::new();
    Ok(Expr::set(set))
}

pub fn setup_lib_complex(context: &mut Context) {
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
