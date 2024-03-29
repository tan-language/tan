// #insight
// What we call reserved_symbol is maybe similar to lisp's 'special form'?

// #todo consider using `name` instead of `symbol`?
// #todo better organize the util module.

pub mod constants;
pub mod fmt;
pub mod module_util;
pub mod put_back_iterator;
pub mod standard_names;

// #todo this looks SLOW, maybe just use a HashSet here?
/// Returns true if `sym` is reserved.
pub fn is_reserved_symbol(sym: &str) -> bool {
    // #todo think about `Func`.
    // #todo currently it include even non-reserved symbols, fix!!!
    matches!(
        sym,
        "do"
            | "def"
            | "and"
            | "or"
            | "not"
            | "let"
            | "let-ds"
            | "set!"
            | "if"
            | "cond"
            | "return"
            | "continue"
            | "break"
            | "for"
            | "for->list" // #todo reconsider the name!
            | "while"
            | "for-each" // #todo extract as seq function
            | "map"
            | "eval"
            | "panic!"
            | "quot"
            | "scope-update" // #todo rename to `let*` or `let..` or `let-all` or something.
            | "use" // #todo consider `using`
            | "Char"
            | "Func"
            | "Macro"
            | "List"
            | "Array"
            | "Map"
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

/// Returns true if the input can be a path literal.
pub fn is_path_literal(input: &str) -> bool {
    input.contains('/') || input.contains('@') || input == ".." // || input == "."
}

// #todo move to utils file?
// #todo should consider n/step a 0..n/step range?
pub fn is_range_literal(input: &str) -> bool {
    // #todo should be more precise, e.g. report 0...1, 0......3, etc.
    (!is_path_literal(input)) && input.contains("..") && input != "..."
}

// #todo consider is_type_symbol, is_type_literal.
// A type starts with an uppercase character.
pub fn is_type(input: &str) -> bool {
    let mut chars = input.chars();
    if let Some(c) = chars.next() {
        // #todo is_ascii_uppercase is used for performance, but we should consider is_uppercase in the future.
        // c.is_uppercase()
        c.is_ascii_uppercase()
    } else {
        false
    }
}

// #todo maybe use a custom Expr::DSSymbol expression to move the detection to read/static time?
pub fn is_dynamically_scoped(name: &str) -> bool {
    // #todo don't allow `**`
    // #todo replace with regex? I am wondering which is faster.
    name.starts_with('*') && name.ends_with('*') // && name.len() > 2;
}

#[cfg(test)]
mod tests {
    use crate::util::{is_dynamically_scoped, is_type};

    #[test]
    fn is_type_should_detect_type_symbols() {
        assert!(is_type("User"));
        assert!(is_type("Ledger-Account"));
        assert!(!is_type("user"));
    }

    #[test]
    fn is_dynamically_scoped_should_detect_special_names() {
        assert!(is_dynamically_scoped("*context*"));
        assert!(is_dynamically_scoped("*my-context*"));
        assert!(!is_dynamically_scoped("*context"));
        assert!(!is_dynamically_scoped("my-context"));
        // #todo fix the next two assertions
        // assert!(!is_dynamically_scoped("**"));
        // assert!(!is_dynamically_scoped("****"));
    }
}
