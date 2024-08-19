use crate::{
    context::Context,
    error::Error,
    expr::{expr_clone, Expr},
    util::args::unpack_arg,
};

use super::{eval, invoke};

// #insight Function pipes are related to function composition.

// #insight C++ reuses >> and << both for bitwise shift and the stream insertion operator.

// - Composition: f . g . h (x) is equivalent to f(g(h(x)))
// - Pipes: h |> g |> f (x) is equivalent to f(g(h(x)))

// Function composition implementation
// const compose = (...fns) => x => fns.reduceRight((v, f) => f(v), x);

// Function piping implementation
// const pipe = (...fns) => x => fns.reduce((v, f) => f(v), x);

// #todo #think How can one input, one output help here?

// #todo Also implement composition.

// #todo Actually, `pipe` can be a function.
// #todo Implement as standard function.

// (|> 5 add1 add2 add3) -> (add3 (add2 (add1 5)))
// ((|> add1 add2 add3) 5) -> (add3 (add2 (add1 5)))

// #todo Consider using >> as the operator, and use something different for bitwise shift operators.

// #ai
// const pipe = (value, ...fns) => fns.reduce((v, f) => f(v), value);
pub fn eval_pipe(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo Optimize, for the moment it's just a demo.
    // #todo What would be a good name for `value`?
    let value = unpack_arg(args, 0, "value")?;
    let funcs = &args[1..];

    let mut value = expr_clone(value);

    // let value = funcs
    //     .into_iter()
    //     .fold(value, |acc, func| invoke(func, vec![value], context));

    // #insight Cannot use fold due to error propagation.

    for func in funcs.iter() {
        // #todo Can we remove this clone somehow?
        let func = eval(func, context)?;
        let args = vec![expr_clone(&value)];
        value = invoke(&func, args, context)?;
    }

    Ok(value)
}
