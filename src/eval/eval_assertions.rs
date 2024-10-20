use crate::{
    context::Context,
    error::Error,
    expr::{expr_clone, Expr},
};

// #todo Should extract the implementation into an assert foreign library, should make a macro.

use super::{eval, util::get_current_file_path};

// #todo Introduce assert-some, or assert-some?
// #todo Introduce assert-includes/contains? Not really can use (assert (contains? array val))

pub fn eval_assert(op: &Expr, args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let [assert_expr] = args else {
        return Err(Error::invalid_arguments(
            "requires `predicate` argument",
            op.range(),
        ));
    };

    // #todo don't throw the error, include in failures!
    let predicate = eval(assert_expr, context)?;

    let Some(predicate) = predicate.as_bool() else {
        return Err(Error::invalid_arguments(
            &format!("`{}` is not a Bool", predicate.unpack()),
            predicate.range(),
        ));
    };

    if predicate {
        Ok(Expr::Bool(true))
    } else {
        if let Some(value) = context.get("*test-failures*", true) {
            if let Some(mut failures) = value.as_array_mut() {
                let file_path = get_current_file_path(context);
                let location = if let Some(range) = op.range() {
                    format!(":{}:{}", range.start.line + 1, range.start.col + 1)
                } else {
                    String::new()
                };
                failures.push(Expr::string(format!(
                    "{}\n  at {}{}",
                    assert_expr, file_path, location
                )));
            }
        }
        Ok(Expr::Bool(false))
    }
}

// #todo make polymorphic, support Float, etc...
pub fn eval_assert_eq(op: &Expr, args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo skip in debug profile.

    // #todo there is not really left and right in Tan syntax.
    // #todo if we use (left, right), right is the ..'right' (correct/expected) value.
    let [left_expr, right_expr] = args else {
        return Err(Error::invalid_arguments(
            "requires `left` and `right` arguments",
            op.range(),
        ));
    };

    // #todo don't throw the error, include in failures!
    // #todo Rename *test-count* -> *assertion-count*, *test-failures* -> *assertion-failures*

    // #todo Optimize this, have a precreated predicate or something.
    // #todo REALLY get read of excessive clones.
    let predicate = Expr::List(vec![
        Expr::symbol("="),
        expr_clone(left_expr),
        expr_clone(right_expr),
    ]);
    // #todo #optimize No need for full eval here, we should use something specialized!
    // #todo What is the correct name here?
    let predicate = eval(&predicate, context)?;
    let Expr::Bool(predicate) = predicate else {
        unreachable!()
    };

    if predicate {
        // #todo Update *test-count*.
        Ok(Expr::Bool(true))
    } else {
        if let Some(value) = context.get("*test-failures*", true) {
            if let Some(mut failures) = value.as_array_mut() {
                let file_path = get_current_file_path(context);
                let location = if let Some(range) = op.range() {
                    format!(":{}:{}", range.start.line + 1, range.start.col + 1)
                } else {
                    String::new()
                };
                let left_value = eval(left_expr, context)?;
                let right_value = eval(right_expr, context)?;
                let mut msg = format!("{left_expr} == {right_expr}");
                msg.push_str(&format!("\n  at {file_path}{location}"));
                if format!("{left_expr}") != format!("{left_value}") {
                    msg.push_str(&format!("\n    hint: {left_expr} -> {left_value}"));
                }
                if format!("{right_expr}") != format!("{right_value}") {
                    msg.push_str(&format!("\n    hint: {right_expr} -> {right_value}"));
                }
                msg.push_str(&format!("\n>>> {left_value} != {right_value}"));
                failures.push(Expr::string(msg));
            }
        }
        Ok(Expr::Bool(false))
    }
}

// #todo have a combined assert for Ok/Some <> Err/None.
// #todo have a combined assert for Try (Success/Fail) expressions.

// #insight We will stop returning Rust errors from most functions, so in the
// future this will not be as useful.
// #todo Consider `assert-err`, compatible with naming conventions.
// (assert-error (Char ""))
// (assert-error (Char "") <err>)
pub fn eval_assert_error(op: &Expr, args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo Implement optional error matching.

    let [expr, ..] = args else {
        return Err(Error::invalid_arguments(
            "requires `expr` argument",
            op.range(),
        ));
    };

    // let _error_name = args.get(1);

    let result = eval(expr, context);
    if result.is_err() {
        // #todo Update *test-count*.
        Ok(Expr::Bool(true))
    } else {
        if let Some(value) = context.get("*test-failures*", true) {
            if let Some(mut failures) = value.as_array_mut() {
                let file_path = get_current_file_path(context);
                let location = if let Some(range) = op.range() {
                    format!(":{}:{}", range.start.line + 1, range.start.col + 1)
                } else {
                    String::new()
                };
                // msg.push_str(&format!("\n>>> {left_value} != {right_value}"));
                let mut msg = "Expected error not returned".to_string();
                msg.push_str(&format!("\n  at {file_path}{location}"));
                msg.push_str(&format!("\n>>> {expr}"));
                failures.push(Expr::string(msg));
            }
        }
        Ok(Expr::Bool(false))
    }
    // #todo Also implement error-matching.
}

// #todo Implement assert-not-error/assert-not-err.
// #todo Refactor, extract common functionality.

// #todo Add unit-tests!
