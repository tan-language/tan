use crate::{api::parse_string_with_position, error::Error, expr::Expr, range::Position};

pub const STRING_INTERPOLATION_FUNC: &str = "String";

// #insight `recognize_` is used instead of e.g. `parse_` to avoid confusion with `parse_string` and other helpers.

// #todo add unit-tests for these functions.

// #insight `$` is not enough and we need the braces `{}`, as symbols can use almost all characters.

// #todo Reconsider the pub visibility of functions here.

// #todo allow whitespace and some non-symbol characters to delineate the interpolation to allow e.g. "$name hello!!".
// #todo support 'toggle' interpolation with %{..}

// #todo Refactor, this is a temp implementation.
// #todo Don't use the template name here.
// #todo Could use (String ...) constructor instead of (format ...)? (format <> scan)
// #ai-generated
// Parses string templates, e.g. "name: ${name}, age: ${age}."
pub fn recognize_string_template(
    input: &str,
    start_position: Position,
) -> Result<Expr, Vec<Error>> {
    let mut exprs = vec![Expr::symbol(STRING_INTERPOLATION_FUNC)];

    let mut previous_end = 0;

    while let Some(start) = input[previous_end..].find("${") {
        let start = previous_end + start;

        if let Some(end) = input[start..].find('}') {
            let end = start + end + 1;

            // Add text before the interpolation.
            exprs.push(Expr::string(&input[previous_end..start]));

            // #todo make sure that interpolation error contains correct range!
            // #todo what happens if the interpolation contains an }?
            // Add the interpolation itself.
            exprs.push(parse_string_with_position(
                &input[(start + 2)..(end - 1)],
                start_position,
            )?);

            previous_end = end;
        } else {
            // If '}' is not found after '${', break the loop.
            break;
        }
    }

    if previous_end < input.len() {
        // Add the remaining text after the last interpolation.
        exprs.push(Expr::string(&input[previous_end..]));
    }

    Ok(Expr::List(exprs))
}

// pub fn recognize_range_old(range_str: &str) -> Option<Expr> {
//     // #todo should convert to (Range start, end, step)
//     let parts: Vec<&str> = range_str.split('|').collect();

//     if parts.len() > 2 {
//         // If there are more than two parts, the input format is invalid
//         return None;
//     }

//     // #todo support parameters in range, e.g. start..end, 0..end, start..end, etc.

//     // Parse the start and end values from the first part of the split
//     let start_end: Vec<&str> = parts[0].split("..").collect();
//     if start_end.len() != 2 {
//         // If the start-end part doesn't have exactly two elements, the input format is invalid
//         return None;
//     }

//     if start_end[0].contains('.') {
//         let start: f64 = start_end[0].parse().ok()?;
//         let end: f64 = start_end[1].parse().ok()?;

//         // Default step value is 1.0 if not provided
//         let step: f64 = if parts.len() == 2 {
//             parts[1].parse().ok()?
//         } else {
//             // #todo think more about start == end
//             if end >= start {
//                 1.0
//             } else {
//                 -1.0
//             }
//         };

//         Some(Expr::FloatRange(start, end, step))
//     } else {
//         let start: i64 = start_end[0].parse().ok()?;
//         let end: i64 = start_end[1].parse().ok()?;

//         // Default step value is 1 if not provided
//         let step: i64 = if parts.len() == 2 {
//             parts[1].parse().ok()?
//         } else {
//             // #todo think more about start == end
//             if end >= start {
//                 1
//             } else {
//                 -1
//             }
//         };

//         Some(Expr::IntRange(start, end, step))
//     }
// }

// #todo should return Result.
fn parse_range_component(lexeme: &str) -> Option<Expr> {
    // if let Ok(n) = lexeme.parse::<i64>() {
    //     return Some(Expr::Int(n));
    // }
    // if let Ok(n) = lexeme.parse::<f64>() {
    //     return Some(Expr::Float(n));
    // }
    let expr = if is_potential_number(lexeme) {
        if lexeme.contains('.') {
            Expr::Float(lexeme.parse().ok()?)
        } else {
            Expr::Int(lexeme.parse().ok()?)
        }
    } else {
        // #todo check that this is a valid symbol, reuse symbol parsing functionality!
        Expr::symbol(lexeme)
    };

    Some(expr)
}

// #todo better return Result, so that exact error details can be emitted!
// #todo cleanup the implementation.
// #todo move to another file.
// #todo the `/step` part is conflicting with paths, e.g. in `use` statements, consider other separator.
// #todo consider not supporting the step in the literal, instead: (with-step 3..10 2) <- THIS
// #todo support 3..=10 literal
// #todo consider swift-like range literals, closed: 2...3 and open 2..<3
/// Parses a range string: start..end|step.
pub fn recognize_range(range_str: &str) -> Option<Expr> {
    // #insight don't build a custom range expression to support dyn-time ranges.

    // #todo #cool if bounds are static, build the specialized expression in parser or static-time pass.

    // #todo should convert to (Range start, end, step)
    let parts: Vec<&str> = range_str.split('|').collect();

    if parts.len() > 2 {
        // If there are more than two parts, the input format is invalid
        return None;
    }

    // #todo support parameters in range, e.g. start..end, 0..end, start..end, etc.
    // #todo support open intervals, e.g. start.. ..end

    // Parse the start and end values from the first part of the split
    let start_end: Vec<&str> = parts[0].split("..").collect();
    if start_end.len() != 2 {
        // If the start-end part doesn't have exactly two elements, the input format is invalid
        return None;
    }

    let mut exprs = vec![Expr::symbol("Range")];

    exprs.push(parse_range_component(start_end[0])?);
    exprs.push(parse_range_component(start_end[1])?);

    if parts.len() == 2 {
        exprs.push(parse_range_component(parts[1])?);
    }

    Some(Expr::List(exprs))
}

// #todo Force the `:` at the beginning
// #todo Consider alternative syntax

// #insight
// Use `:` in the middle as 'key-path' syntax-sugar, e.g.
// user:data:name -> ((user :data) :name)

/// A key is considered a `KeySymbol` (aka 'keyword') if it contains a collon. A collon
/// can be at the end or at the beginning, or the end of the lexeme. Lexemes with
/// the colon in the middle are considered key-paths.
/// A `KeySymbol` always evaluates to itself.
#[inline(always)]
pub fn is_key_symbol(lexeme: &str) -> bool {
    // #todo the `:=` is not used yet, `<-` is provided instead.
    // // The `:=` operator is a special case.
    // if lexeme == ":=" {
    //     return false;
    // }
    lexeme.starts_with(':') || lexeme.ends_with(':')
}

pub fn is_key_path(lexeme: &str) -> bool {
    // #todo Optimize this.
    lexeme.contains(':') && (!is_key_symbol(lexeme))
}

// #insight it's actually a potential number!
pub fn is_potential_number(lexeme: &str) -> bool {
    let mut chars = lexeme.chars();

    let Some(ch) = chars.next() else {
        return false;
    };
    if ch == '-' {
        let Some(ch1) = chars.next() else {
            return false;
        };
        char::is_numeric(ch1)
    } else {
        char::is_numeric(ch)
    }
}

pub fn is_potential_uint(lexeme: &str) -> bool {
    let mut chars = lexeme.chars();

    let Some(ch) = chars.next() else {
        return false;
    };

    char::is_numeric(ch)
}

// Two kinds of keypaths:
// - (user :info:name:middle) ; type-a
// - user:info:name:middle    ; type-b

// #todo Should return error.
// #todo Handle 'type-a' key-paths.
// #insight The key-path segments can only be strings and ints.
// This desugars 'type-b' key-paths.
pub fn desugar_key_path(lexeme: &str) -> Expr {
    // user:info:name:middle -> (((user :info) name) :middle)
    let mut segments = lexeme.split(':');
    // #insight The unwrap is safe here.
    // #todo Verify subject is not numeric.
    let mut expr = Expr::symbol(segments.next().unwrap());

    for key in segments {
        // #todo Add error handling.
        let key = if is_potential_uint(key) {
            Expr::Int(key.parse().unwrap())
        } else {
            Expr::key_symbol(key)
        };
        expr = Expr::List(vec![expr, key]);
    }

    expr
}

#[cfg(test)]
mod tests {
    use crate::parser::util::{is_key_path, is_key_symbol, is_potential_number};

    use super::desugar_key_path;

    #[test]
    fn is_key_symbol_usage() {
        assert!(is_key_symbol(":key"));
        assert!(is_key_symbol("key:"));
        assert!(!is_key_symbol("key:path:segment"));
    }

    #[test]
    fn is_key_path_usage() {
        assert!(is_key_path("key:path:segment"));
        assert!(!is_key_path(":key"));
        assert!(!is_key_path("key:"));
    }

    #[test]
    fn is_potential_number_usage() {
        assert!(is_potential_number("1"));
        assert!(is_potential_number("1.0"));
        assert!(is_potential_number("-1.2"));
        assert!(!is_potential_number("random-symbol"));
    }

    #[test]
    fn desugar_key_path_usage() {
        let expr = desugar_key_path("user:detail:0:name");
        assert_eq!(format!("{expr}"), "(((user :detail) 0) :name)");
    }
}
