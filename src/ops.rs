pub mod arithmetic;
pub mod eq;
pub mod lang;
pub mod seq;
pub mod string;

// #todo organize within stdlib or something.

// #TODO helper function or macro for arithmetic operations!
// #TODO also eval 'if', 'do', 'for' and other keywords here!
// #TODO use macros to monomorphise functions? or can we leverage Rust's generics? per viariant? maybe with cost generics?
// #TODO support overloading,
// #TODO make equality a method of Expr?
// #TODO support non-Int types
// #TODO support multiple arguments.
// #TODO helper function or macro for arithmetic operations!

// #TODO primitives, builtins
// #TODO cannot be implemented with Expr::ForeignFunc as the args are pre-evaluated.

// #Insight special-form != primitive ? special form requires special case in the interpreter?
