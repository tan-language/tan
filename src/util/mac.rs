// #todo Find a better name for this macro.
#[macro_export]
macro_rules! magic_name {
    ($input:expr) => {
        // #todo Find a better name convention.
        concat!("!", $input, "!")
    };
}
