use std::{fs::File, io::BufWriter, sync::Arc};

use png::ColorType;

use crate::{
    context::Context,
    error::Error,
    expr::{annotate_type, expr_clone, Expr},
    util::{
        args::{
            unpack_buffer_arg, unpack_foreign_struct_arg, unpack_int_arg, unpack_stringable_arg,
        },
        expect_lock_write,
        module_util::require_module,
    },
};

// #ref https://github.com/image-rs/image-png

struct PngCoderData {
    pub width: u32,
    pub height: u32,
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

    // #todo verify the type of the writable.

    // let writable = unpack_foreign_struct_arg(args, 3, "writable", "Writable")?;
    // let s = expect_lock_write(writable);
    // let Some(mut file) = s.downcast_ref::<std::fs::File>() else {
    //     return Err(Error::invalid_arguments("invalid File", None));
    // };

    let data = PngCoderData {
        width: width as u32,
        height: height as u32,
        // #todo set color_type based on encoding.
        color_type: ColorType::Grayscale,
        writable: expr_clone(writable),
    };

    let expr = Expr::ForeignStruct(Arc::new(data));

    Ok(annotate_type(expr, "Coder"))
}

pub fn png_coder_write(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let coder = unpack_foreign_struct_arg(args, 0, "coder", "Coder")?;
    let Some(coder) = coder.downcast_ref::<PngCoderData>() else {
        return Err(Error::invalid_arguments("invalid Coder", args[0].range()));
    };

    let Expr::ForeignStructMut(writable) = coder.writable.unpack() else {
        return Err(Error::invalid_arguments("invalid Writable", None));
    };
    let writable = expect_lock_write(writable);
    // #todo temporarily forcing downcast to File! Should force to Write?
    let Some(writable) = writable.downcast_ref::<File>() else {
        return Err(Error::invalid_arguments(
            "invalid Writable",
            args[0].range(),
        ));
    };

    let data = unpack_buffer_arg(args, 1, "data")?;

    let writer = &mut BufWriter::new(writable);
    let mut encoder = png::Encoder::new(writer, coder.width, coder.height);
    encoder.set_color(coder.color_type);
    encoder.set_depth(png::BitDepth::Eight);

    // #todo handle the errors without unwrap!
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&data).unwrap();

    Ok(Expr::Nil)
}

pub fn setup_lib_image_png(context: &mut Context) {
    // #todo consider other paths?
    let module = require_module("image/png", context);

    module.insert("Coder", Expr::ForeignFunc(Arc::new(png_coder_new)));
    module.insert(
        "write$$Coder$$Buffer",
        Expr::ForeignFunc(Arc::new(png_coder_write)),
    );
}
