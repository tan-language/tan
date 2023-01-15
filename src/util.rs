pub fn is_reserved_symbol(sym: &str) -> bool {
    // #TODO think about `Func`.
    matches!(sym, "do" | "ann" | "let" | "if" | "quot" | "Func")
}
