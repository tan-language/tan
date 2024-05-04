// #todo consider other names: `Buf`, `Byte-Buffer`, etc.

use std::sync::{Arc, RwLock};

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

// #insight
// size -> in bytes (maybe size-in-bytes ?)
// length/count -> in items/elements (maybe size ?)

// #todo use array instead of vec? can we have dynamic array, probably a slice.

// #todo make buffer Iterable/Iterate

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
    // #todo enforce bounds!!

    let [buffer, index, value] = args else {
        return Err(Error::invalid_arguments(
            "requires `index` and `value` arguments",
            None,
        ));
    };

    let Some((length, mut buffer)) = buffer.as_buffer_mut() else {
        return Err(Error::invalid_arguments(
            &format!("buffer=`{buffer}` is not a Buffer"),
            buffer.range(),
        ));
    };

    let Some(i) = index.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("index=`{index}` is not Int"),
            index.range(),
        ));
    };

    if i < 0 {
        // #todo use specialized error variant? e.g. invalid_argument_out_of_bounds?
        return Err(Error::invalid_arguments(
            &format!("buffer index=`{i}` cannot be negative"),
            index.range(),
        ));
    }

    let i = i as usize;

    if i >= length {
        // #todo separate error message for <0, >= length, give length in the later.
        return Err(Error::invalid_arguments(
            &format!("buffer index=`{i}` must be less than the buffer length"),
            index.range(),
        ));
    }

    let Some(value) = value.as_u8() else {
        return Err(Error::invalid_arguments(
            &format!("value=`{value}` is not U8"),
            value.range(),
        ));
    };

    buffer[i] = value;

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

// #todo push with Int, reuse push with U8
// #todo support 5u8

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn buffer_put_usage() {
        let mut context = Context::new();

        let input = r#"
            (let buf (Buffer 4))
            (put buf 2 (U8 9))
            buf
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = r#"[0 0 9 0]"#;
        assert_eq!(value, expected);

        let input = r#"
            (let buf (Buffer 4))
            (put buf -4 (U8 9))
        "#;
        let result = eval_string(input, &mut context);
        assert!(result.is_err());
        let error = &result.unwrap_err()[0];
        assert_eq!("buffer index=`-4` cannot be negative", error.notes[0].text);

        let input = r#"
            (let buf (Buffer 4))
            (put buf 15 (U8 9))
        "#;
        let result = eval_string(input, &mut context);
        assert!(result.is_err());
        let error = &result.unwrap_err()[0];
        assert_eq!(
            "buffer index=`15` must be less than the buffer length",
            error.notes[0].text
        );
    }
}
