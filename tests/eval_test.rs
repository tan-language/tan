mod common;

use assert_matches::assert_matches;

use tan::{
    api::eval_string,
    context::Context,
    error::{Error, ErrorVariant},
    eval::eval,
    expr::Expr,
};

use crate::common::{eval_file, eval_input, read_file};

// #todo Add more tests, especially for error-reporting.
// #todo Prefer adding tan tests.
// #todo Convert the remaining tests to Tan.

#[test]
fn do_reports_intermediate_errors() {
    let result = eval_file("do-intermediate-error.tan");

    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    dbg!(&err);

    // #insight
    // emitting undefined-symbol happened after removing resolver but it may
    // actually be better.
    // assert_matches!(err, Error{ kind: ErrorKind::UndefinedFunction(s, _), .. } if s == "write33");
    assert_matches!(err, Error{ variant: ErrorVariant::UndefinedSymbol(s), .. } if s == "write33");
}

#[test]
fn eval_processes_keyword_symbols() {
    let result = eval_input(":key").unwrap();

    assert_matches!(result.unpack(), Expr::KeySymbol(x) if x == "key");
}

// #keep
#[test]
fn eval_processes_empty_list() {
    let expr = Expr::List(Vec::new());

    let mut context = Context::new();
    let value = eval(&expr, &mut context).unwrap();

    assert_matches!(value.unpack(), Expr::None);
}

// #keep
#[test]
fn eval_reports_let_errors() {
    let result = eval_input("(do (let if (+ 1 2 3)) a)");

    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // assert!(
    //     matches!(err, Error{ kind: ErrorKind::InvalidArguments(x), .. } if x == "let cannot shadow the reserved symbol `if`")
    // );

    assert_matches!(
        err,
        Error {
            variant: ErrorVariant::InvalidArguments,
            ..
        }
    );

    let range = err.notes.first().unwrap().range.as_ref().unwrap();

    assert_eq!(range.start.index, 9);
    assert_eq!(range.end.index, 11);
}

#[test]
fn eval_handles_function_with_no_params() {
    let result = eval_file("func-no-params.tan");
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("func-no-params.value.tan");
    assert_eq!(value, expected_value);
}

#[test]
fn eval_processes_map() {
    let result = eval_file("map.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("map.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_processes_map_with_keys() {
    let result = eval_file("key-map.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("key-map.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_processes_multiline_strings() {
    let result = eval_file("multi-line-string.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = "\"This is\n        some nice text\"";

    assert_eq!(value, expected_value);
}

#[test]
fn eval_processes_multiline_text() {
    let result = eval_file("multi-line-text.tan");

    assert!(result.is_ok());

    let value = result.unwrap().to_string();
    let expected_value = read_file("multi-line-text.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_processes_deep_data() {
    let result = eval_file("data.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("data.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn should_eval_panic() {
    let result = eval_file("panic.tan");
    let error = result.unwrap_err();
    let error = error.first().unwrap();
    assert_matches!(error.variant, ErrorVariant::Panic(..));
}

#[test]
fn eval_should_report_errors_in_function_incovations() {
    let result = eval_file("func-error.tan");
    assert!(result.is_err());
}

#[test]
fn eval_should_report_undefined_symbol_errors() {
    let result = eval_file("undefined.tan");
    assert!(result.is_err());
}

#[test]
fn eval_should_report_errors_in_args() {
    let result = eval_file("array-errors.tan");
    assert!(result.is_err());
}

// #[test]
// fn eval_should_support_list_destructuring() {
//     let expr = eval_file("list-destruct.tan").unwrap();
//     let value = format_value(expr);
//     assert_eq!(value, "1 | (2 3 4 5)");
// }

#[test]
fn eval_should_support_map_destructuring() {
    let result = eval_file("map-destruct.tan");
    let value = result.unwrap();
    let value = value.as_string().unwrap();
    assert_eq!(value, "George");
}

#[test]
fn eval_should_handle_func_def_with_annotation() {
    let input = &read_file("func-with-ann.tan");
    let mut context = Context::new();
    let _ = eval_string(input, &mut context);
    assert!(context.scope.contains_name("relu$$Float"));
    // #insight Verifies hack-fix for method lookup.
    assert!(context.scope.contains_name("relu"));
}
