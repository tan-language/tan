mod common;

use crate::common::{eval_file, read_file};

#[test]
fn eval_processes_arithmetic_expressions() {
    let result = eval_file("sum.tan");

    assert!(result.is_ok());

    let result = format!("{}", result.unwrap());
    let expected_result = read_file("sum.result.tan");

    assert_eq!(result, expected_result);
}
