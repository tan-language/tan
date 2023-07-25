// #Insight
// What we call reserved_symbol is maybe similar to lisp's 'special form'?

// #todo consider using `name` instead of `symbol`?
// #todo better organize the util module.

pub mod fmt;
pub mod put_back_iterator;
pub mod standard_names;

/// Returns true if `sym` is reserved.
pub fn is_reserved_symbol(sym: &str) -> bool {
    // #todo think about `Func`.
    matches!(
        sym,
        "do" | "ann" // #todo consider `meta`?
            | "def"
            | "let"
            | "if"
            | "for"
            | "while"
            | "for-each" // #todo extract as seq function
            | "map"
            | "eval"
            | "quot"
            | "use" // #todo consider `using`
            | "Char"
            | "Func"
            | "Macro"
            | "List"
            | "Array"
            | "Dict"
    )
}

/// The`Break` is thrown when a pass processor cannot synchronize
/// to continue processing to detect more errors. Processing is stopped immediately.
/// Typically signals non-recoverable errors or end of input.
#[derive(Debug)]
pub struct Break {}

impl std::error::Error for Break {}

impl std::fmt::Display for Break {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Break")
    }
}
