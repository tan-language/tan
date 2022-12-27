mod common;

use tan::eval::error::EvalError;

use crate::common::{eval_file, read_file};

#[test]
fn eval_processes_arithmetic_expressions() {
    let result = eval_file("sum.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("sum.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn do_reports_intermediate_errors() {
    let result = eval_file("do_intermediate_error.tan");

    assert!(result.is_err());

    let err = result.unwrap_err();

    assert!(matches!(err, EvalError::UndefinedSymbolError(s) if s == "write33"));
}

#[test]
fn eval_processes_conditionals() {
    let result = eval_file("conditional.tan");
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("conditional.value.tan");

    assert_eq!(value, expected_value);
}
