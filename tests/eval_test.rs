mod common;

use tan::{
    eval::{env::Env, error::EvalError, eval},
    expr::Expr,
};

use crate::common::{eval_file, eval_string, read_file};

#[test]
fn eval_processes_arithmetic_expressions() {
    let result = eval_file("sum.tan");

    dbg!(&result);

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

    assert!(matches!(err, EvalError::UndefinedSymbol(s) if s == "write33"));
}

#[test]
fn eval_processes_conditionals() {
    let result = eval_file("conditional.tan");
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("conditional.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_processes_keyword_symbols() {
    let result = eval_string(":key").unwrap();

    assert!(matches!(result, Expr::KeySymbol(x) if x == "key"));

    // let value = format!("{}", result.unwrap());
    // let expected_value = read_file("conditional.value.tan");

    // assert_eq!(value, expected_value);
}

#[test]
fn eval_processes_empty_list() {
    let expr = Expr::List(Vec::new());
    let mut env = Env::new();

    let value = eval(expr, &mut env).unwrap();

    assert!(matches!(value, Expr::One));
}

#[test]
fn eval_processes_let() {
    let result = eval_string("(do (let a (+ 1 2 3)) a)");
    dbg!(&result);

    // #TODO add asserts!
}

#[test]
fn eval_reports_let_errors() {
    let result = eval_string("(do (let if (+ 1 2 3)) a)");

    assert!(result.is_err());

    let err = result.unwrap_err();

    assert!(
        matches!(err, EvalError::InvalidArguments(x) if x == "let cannot shadow the reserved symbol `if`")
    );
}

// #TODO extract full testing from file.

#[test]
fn eval_processes_quoted_expressions() {
    let result = eval_file("quoted.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("quoted.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_processes_function_definition_and_application() {
    let result = eval_file("factorial.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("factorial.value.tan");

    assert_eq!(value, expected_value);

    let result = eval_file("fibonacci.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("fibonacci.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_processes_dict() {
    let result = eval_file("dict.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("dict.value.tan");

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
