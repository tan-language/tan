pub fn is_reserved_symbol(sym: &str) -> bool {
    matches!(sym, "do" | "let" | "if" | "quot")
}
