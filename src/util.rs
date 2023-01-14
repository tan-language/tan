pub fn is_reserved_symbol(sym: &str) -> bool {
    // #TODO think about `Func`.
    matches!(sym, "do" | "let" | "if" | "quot" | "Func")
}
