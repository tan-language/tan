use crate::context::Context;

use self::png::setup_lib_image_png;

pub mod png;

// #todo make image/png an optional feature?
// #todo consider other names, e.g. `raster`, `graphics`, ...

pub fn setup_lib_image(context: &mut Context) {
    setup_lib_image_png(context);
}
