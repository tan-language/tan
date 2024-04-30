// #todo consider other names: `Buf`, `Byte-Buffer`, etc.

use std::sync::{Arc, RwLock};

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

// #insight
// size -> in bytes (maybe size-in-bytes ?)
// length/count -> in items/elements (maybe size ?)

// #todo use array instead of vec? can we have dynamic array, probably a slice.

pub fn buffer_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo also support a default-element/fill option.

    let [length] = args else {
        return Err(Error::invalid_arguments("requires `length` argument", None));
    };

    // #todo create a helper.
    let Some(length) = length.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("length=`{length}` is not Int"),
            length.range(),
        ));
    };

    let length = length as usize;
    let buf: Vec<u8> = Vec::with_capacity(length);

    Ok(Expr::Buffer(length, Arc::new(RwLock::new(buf))))
}

pub fn setup_lib_buffer(context: &mut Context) {
    // #todo put in 'buffer' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    module.insert("Buffer", Expr::ForeignFunc(Arc::new(buffer_new)));
}
