use std::{fs::File, sync::Arc};

use png::ColorType;

use crate::{
    context::Context,
    error::Error,
    expr::{expr_clone, Expr},
    util::{
        args::{unpack_foreign_struct_arg, unpack_int_arg, unpack_stringable_arg},
        expect_lock_write,
        module_util::require_module,
    },
};

// #ref https://github.com/image-rs/image-png

struct PngCoderData {
    pub width: i64,
    pub height: i64,
    pub color_type: ColorType,
    pub writable: Expr,
}

pub fn png_coder_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let width = unpack_int_arg(args, 0, "width")?;
    let height = unpack_int_arg(args, 1, "height")?;
    let _encoding = unpack_stringable_arg(args, 2, "encoding")?;
    let Some(writable) = args.get(3) else {
        return Err(Error::invalid_arguments(
            "missing `writeable` argument",
            None,
        ));
    };

    // let writable = unpack_foreign_struct_arg(args, 3, "writable", "Writable")?;
    // let s = expect_lock_write(writable);
    // let Some(mut file) = s.downcast_ref::<std::fs::File>() else {
    //     return Err(Error::invalid_arguments("invalid File", None));
    // };

    let data = PngCoderData {
        width,
        height,
        color_type: ColorType::Grayscale,
        writable: expr_clone(writable),
    };

    Ok(Expr::ForeignStruct(Arc::new(data)))
}

pub fn setup_lib_image_png(context: &mut Context) {
    // #todo consider other paths?
    let module = require_module("image/png", context);

    module.insert("Coder", Expr::ForeignFunc(Arc::new(png_coder_new)));
}
