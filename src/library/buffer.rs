// #todo consider other names: `Buf`, `Byte-Buffer`, etc.

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

// #insight
// size -> in bytes (maybe size-in-bytes ?)
// length/count -> in items/elements (maybe size ?)

pub fn buffer_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // // #todo make some of the arguments optional, e.g. step.
    // let [start, end, ..] = args else {
    //     return Err(Error::invalid_arguments(
    //         "requires `start`, `end` arguments",
    //         None,
    //     ));
    // };

    // // #todo create a helper.
    // let Some(start) = start.as_int() else {
    //     return Err(Error::invalid_arguments(
    //         &format!("start=`{start}` is not Int"),
    //         start.range(),
    //     ));
    // };

    // let Some(end) = end.as_int() else {
    //     return Err(Error::invalid_arguments(
    //         &format!("end=`{end}` is not Int"),
    //         end.range(),
    //     ));
    // };

    // let step = if let Some(step) = args.get(2) {
    //     let Some(step) = step.as_int() else {
    //         return Err(Error::invalid_arguments(
    //             &format!("step=`{step}` is not Int"),
    //             step.range(),
    //         ));
    //     };
    //     step
    // } else if end >= start {
    //     1
    // } else {
    //     -1
    // };

    // // #todo use Expr::ForeignStruct
    // Ok(Expr::IntRange(start, end, step))

    Ok(Expr::Nil)
}

pub fn setup_lib_buffer(context: &mut Context) {
    // #todo put in 'buffer' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);
}
