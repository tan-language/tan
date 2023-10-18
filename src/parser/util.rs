use crate::{api::parse_string, error::Error, expr::Expr};

// #insight `recognize_` is used instead of e.g. `parse_` to avoid confusion with `parse_string` and other helpers.

// #todo add unit-tests for these functions.

// #todo refactor
// #ai-generated
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

// #ai-generated
// #todo cleanup the implementation.
// #todo move to another file.
/// Parses a range string: start..end/step.
pub fn recognize_range(range_str: &str) -> Option<Expr> {
    let parts: Vec<&str> = range_str.split('/').collect();

    if parts.len() > 2 {
        // If there are more than two parts, the input format is invalid
        return None;
    }

    // Parse the start and end values from the first part of the split
    let start_end: Vec<&str> = parts[0].split("..").collect();
    if start_end.len() != 2 {
        // If the start-end part doesn't have exactly two elements, the input format is invalid
        return None;
    }

    if start_end[0].contains(".") {
        let start: f64 = start_end[0].parse().ok()?;
        let end: f64 = start_end[1].parse().ok()?;

        // Default step value is 1.0 if not provided
        let step: f64 = if parts.len() == 2 {
            parts[1].parse().ok()?
        } else {
            1.0
        };

        Some(Expr::FloatRange(start, end, step))
    } else {
        let start: i64 = start_end[0].parse().ok()?;
        let end: i64 = start_end[1].parse().ok()?;

        // Default step value is 1 if not provided
        let step: i64 = if parts.len() == 2 {
            parts[1].parse().ok()?
        } else {
            1
        };

        Some(Expr::IntRange(start, end, step))
    }
}

/// A key is considered a `KeySymbol` (aka 'keyword') if it contains a collon. A collon
/// can be at the end or at the beginning, or even in the middle of the lexeme.
/// A `KeySymbol` always evaluates to itself.
#[inline(always)]
pub fn is_key_symbol(lexeme: &str) -> bool {
    lexeme.contains(':')
}
