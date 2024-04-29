use crate::{api::parse_string, error::Error, expr::Expr};

// #insight `recognize_` is used instead of e.g. `parse_` to avoid confusion with `parse_string` and other helpers.

// #todo add unit-tests for these functions.

// #insight `$` is not enough and we need the braces `{}`, as symbols can use almost all characters.

// #todo allow whitespace and some non-symbol characters to delineate the interpolation to allow e.g. "$name hello!!".
// #todo support 'toggle' interpolation with %{..}

// #todo refactor, this is a temp implementation.
// #todo could use (String ...) constructor instead of (format ...)? (format <> scan)
// #ai-generated
// Parses string templates, e.g. "name: ${name}, age: ${age}."
pub fn recognize_string_template(input: &str) -> Result<Expr, Vec<Error>> {
    let mut exprs = vec![Expr::symbol("format")];

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
            exprs.push(parse_string(&input[(start + 2)..(end - 1)])?);

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

    let start_expr = parse_range_component(start_end[0])?;
    let end_expr = parse_range_component(start_end[1])?;

    // Default step value is 1.0 if not provided
    let step_expr = if parts.len() == 2 {
        parse_range_component(parts[1])?
    } else {
        // #todo think more about start == end
        if let Expr::Float(..) = start_expr {
            let start = start_expr.as_float().unwrap();
            let end = end_expr.as_float().unwrap();
            if end >= start {
                Expr::Float(1.0)
            } else {
                Expr::Float(-1.0)
            }
        } else {
            let start = start_expr.as_int().unwrap();
            let end = end_expr.as_int().unwrap();
            if end >= start {
                Expr::Int(1)
            } else {
                Expr::Int(-1)
            }
        }
    };

    exprs.push(start_expr);
    exprs.push(end_expr);
    exprs.push(step_expr);

    Some(Expr::List(exprs))
}

// #todo force the `:` at the beginning

/// A key is considered a `KeySymbol` (aka 'keyword') if it contains a collon. A collon
/// can be at the end or at the beginning, or even in the middle of the lexeme.
/// A `KeySymbol` always evaluates to itself.
#[inline(always)]
pub fn is_key_symbol(lexeme: &str) -> bool {
    lexeme.contains(':')
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

#[cfg(test)]
mod tests {
    use crate::parser::util::is_potential_number;

    #[test]
    fn is_potential_number_usage() {
        assert!(is_potential_number("1"));
        assert!(is_potential_number("1.0"));
        assert!(is_potential_number("-1.2"));
        assert!(!is_potential_number("random-symbol"));
    }
}
