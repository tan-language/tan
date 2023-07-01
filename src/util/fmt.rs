// #TODO what about other float types?
/// Formats a float number, ensures it always has a decimal separator.
pub fn format_float(n: f64) -> String {
    let s = n.to_string();

    if !s.contains('.') {
        format!("{s}.0")
    } else {
        s
    }
}
