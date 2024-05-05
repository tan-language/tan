// #todo find a better filename.
// #todo maybe these are 'special' names, not 'standard' names.
// #todo maybe use a suffix? e.g. `_NAME` or `_VAR`?
// #todo find another naming convention, don't like !...! much.
// #todo should these be dynamically-scoped variables?

// #todo merge into constants.rs

// pub enum Profile {
//     Debug,
//     Release,
//     Test,
//     Benchmark,
// }

// #todo consider other names: execution-profile, eval-profile, ...
/// The execution profile
pub const PROFILE: &str = "!PROFILE!";

pub const CURRENT_MODULE_PATH: &str = "!CURRENT-MODULE-PATH!";

pub const CURRENT_FILE_PATH: &str = "!CURRENT-FILE-PATH!";
