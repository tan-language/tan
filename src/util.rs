// #Insight
// What we call reserved_symbol is maybe similar to lisp's 'special form'?

// #TODO consider using `name` instead of `symbol`?
// #TODO better organize the util module.

pub mod fmt;
pub mod put_back_iterator;
pub mod standard_names;

/// Returns true if `sym` is reserved.
pub fn is_reserved_symbol(sym: &str) -> bool {
    // #TODO think about `Func`.
    matches!(
        sym,
        "do" | "ann"
            | "let"
            | "if"
            | "for"
            | "for-each"
            | "map"
            | "eval"
            | "quot"
            | "use" // #TODO consider `using`
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
