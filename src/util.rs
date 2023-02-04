// #Insight
// What we call reserved_symbol is maybe similar to lisp's 'special form'?

// #TODO consider using `name` instead of `symbol`?

/// Returns true if `sym` is reserved.
pub fn is_reserved_symbol(sym: &str) -> bool {
    // #TODO think about `Func`.
    matches!(
        sym,
        "do" | "ann" | "let" | "if" | "for" | "for_each" | "eval" | "quot" | "Func" | "Macro"
    )
}
