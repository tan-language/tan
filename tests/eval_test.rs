mod common;

use tan::eval::error::EvalError;

use crate::common::{eval_file, read_file};

#[test]
fn eval_processes_arithmetic_expressions() {
    let result = eval_file("sum.tan");

    assert!(result.is_ok());

    let result = format!("{}", result.unwrap());
    let expected_result = read_file("sum.result.tan");

    assert_eq!(result, expected_result);
}

#[test]
fn do_reports_intermediate_errors() {
    let result = eval_file("do_intermediate_error.tan");

    assert!(result.is_err());

    let err = result.unwrap_err();

    assert!(matches!(err, EvalError::UndefinedSymbol(s) if s == "write33"));
}
