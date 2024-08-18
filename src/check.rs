use crate::{error::Error, expr::Expr};

// #todo Not the best name, too general, confusing with the upcoming `unchecked` concept.
// #todo Interesting name: vet! (Golang)
// #todo Should be one pass in resolve, just use the check_fn

// #insight
// The check pass/stage performs static analysis and reports errors.

pub fn check_fn(expr: Expr) -> Result<Expr, Error> {
    match expr.unpack() {
        Expr::List(ref terms) => {
            if !terms.is_empty() {
                // #todo Validate let arguments!
                // #todo This may become Expr::typ
                if let Some(s) = terms[0].as_symbol() {
                    // #todo it's weird that we are special-handling "Map" here.
                    if s == "Map" {
                        // Check that the Map constructor has an even number of arguments.
                        // #insight Should be odd, including the op.
                        if terms.len() % 2 == 0 {
                            // #todo Investigate why expr has no range here!
                            return Err(Error::invalid_arguments(
                                "missing argument in Map constructor",
                                terms[0].range(),
                            ));
                        }
                        // #todo Check that '_' inference is correct.
                        return Ok(expr);
                    }
                }
            }
            // #insight No annotations stripped.
            Ok(expr)
        }
        _ => Ok(expr),
    }
}

pub fn check(expr: Expr) -> Result<Expr, Error> {
    expr.try_transform(&check_fn)
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_validates_array_expressions() {
        // #todo
    }
}
