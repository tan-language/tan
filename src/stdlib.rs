pub mod fs;
pub mod io;
pub mod math;
pub mod prelude;
pub mod process;
pub mod rng;

use crate::context::Context;

use self::{
    fs::setup_std_fs, prelude::setup_std_prelude, process::setup_std_process, rng::setup_std_rand,
};

// #todo add unit test for the foreign-functions.

// #todo consider removing the `std` prefix from module paths, like haskell.
// #todo find a better prefix than setup_
// #todo use Rc/Arc consistently
// #todo some helpers are needed here, to streamline the code.

pub fn setup_std(context: &mut Context) {
    setup_std_fs(context);
    setup_std_process(context);
    setup_std_rand(context);
    setup_std_prelude(context);
}
