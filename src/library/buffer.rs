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
    // #todo allow for custom initial value.
    let buf: Vec<u8> = vec![0; length];

    Ok(Expr::Buffer(length, Arc::new(RwLock::new(buf))))
}

// (put buf index value)
pub fn buffer_put(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [buffer, index, value] = args else {
        return Err(Error::invalid_arguments(
            "requires `index` and `value` arguments",
            None,
        ));
    };

    let Some(mut buffer) = buffer.as_buffer_mut() else {
        return Err(Error::invalid_arguments(
            &format!("buffer=`{buffer}` is not a Buffer"),
            buffer.range(),
        ));
    };

    let Some(index) = index.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("index=`{index}` is not Int"),
            index.range(),
        ));
    };

    let Some(value) = value.as_u8() else {
        return Err(Error::invalid_arguments(
            &format!("value=`{value}` is not U8"),
            value.range(),
        ));
    };

    buffer[index as usize] = value;

    // #todo what should we return?
    Ok(Expr::Nil)
}

pub fn setup_lib_buffer(context: &mut Context) {
    // #todo put in 'buffer' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    // #todo consider `Buf`.
    module.insert("Buffer", Expr::ForeignFunc(Arc::new(buffer_new)));

    // #todo also provide a put$$Int

    module.insert(
        "put$$Buffer$$Int$$U8",
        Expr::ForeignFunc(Arc::new(buffer_put)),
    );
}
