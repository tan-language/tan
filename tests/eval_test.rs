mod common;

use tan::{
    api::eval_string,
    error::{Error, ErrorKind},
    eval::{env::Env, eval},
    expr::Expr,
    util::fmt::format_float,
};

use crate::common::{eval_file, read_file};

// #TODO add more tests, especially for error-reporting.

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
    let result = eval_file("do-intermediate-error.tan");

    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    assert!(matches!(err, Error{ kind: ErrorKind::UndefinedFunction(s, _), .. } if s == "write33"));
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
    let mut env = Env::prelude();
    let result = eval_string(":key", &mut env).unwrap();

    assert!(matches!(result.unpack(), Expr::KeySymbol(x) if x == "key"));
}

#[test]
fn eval_processes_empty_list() {
    let expr = Expr::List(Vec::new()).into();
    let mut env = Env::prelude();
    let value = eval(&expr, &mut env).unwrap();

    assert!(matches!(value.unpack(), Expr::One));
}

#[test]
fn eval_processes_let() {
    let mut env = Env::prelude();
    let result = eval_string("(do (let a (+ 1 2 3)) a)", &mut env);
    dbg!(&result);

    // #TODO add asserts!
}

#[test]
fn eval_processes_booleans() {
    let mut env = Env::prelude();
    let value = eval_string("(do (let flag true) flag)", &mut env).unwrap();
    assert!(matches!(value.unpack(), Expr::Bool(x) if *x));

    let value = eval_string("(do (let flag false) flag)", &mut env).unwrap();
    assert!(matches!(value.unpack(), Expr::Bool(x) if !x));
}

#[test]
fn eval_processes_chars() {
    let mut env = Env::prelude();
    let value = eval_string(r#"(let ch (Char "r")) ch"#, &mut env).unwrap();
    assert!(matches!(value.unpack(), Expr::Char(c) if *c == 'r'));
}

#[test]
fn eval_reports_let_errors() {
    let mut env = Env::prelude();
    let result = eval_string("(do (let if (+ 1 2 3)) a)", &mut env);

    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // assert!(
    //     matches!(err, Error{ kind: ErrorKind::InvalidArguments(x), .. } if x == "let cannot shadow the reserved symbol `if`")
    // );

    assert!(matches!(
        err,
        Error {
            kind: ErrorKind::InvalidArguments,
            ..
        }
    ));

    let range = err.notes.first().unwrap().range.as_ref().unwrap();

    assert_eq!(range.start.index, 9);
    assert_eq!(range.end.index, 11);
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
fn do_creates_new_lexical_scope() {
    let mut env = Env::prelude();
    let result = eval_string(
        "
    (do
        (let a 1)
        (do
            (let a (+ 1 2))
        )
        a
    )",
        &mut env,
    );
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = "1";

    assert_eq!(value, expected_value);
}

#[test]
fn ensure_a_infinite_recursion_is_fixed() {
    let mut env = Env::prelude();
    let result = eval_string(
        "
    (do
        (let a 1)
        (let a (+ a 2))
        a
    )",
        &mut env,
    );
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = "3";

    assert_eq!(value, expected_value);
}

#[test]
fn quot_handles_lists() {
    let mut env = Env::prelude();
    let result = eval_string("'(let a 1)", &mut env);
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = "(let a 1)";

    assert_eq!(value, expected_value);

    // #TODO argh! cannot quote if expressions (and more)
    let mut env = Env::prelude();
    let result = eval_string("'(if \"a\" b 1)", &mut env);
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = "(if \"a\" b 1)";

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
fn eval_processes_macros() {
    let result = eval_file("macro.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("macro.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_resolves_function_methods() {
    // assert Int method.

    let result = eval_file("add-int.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("add-int.value.tan");

    assert_eq!(value, expected_value);

    // assert Float method.

    let result = eval_file("add-float.tan");

    assert!(result.is_ok());

    let value = format_float(result.unwrap().as_float().unwrap());
    let expected_value = read_file("add-float.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_resolves_multi_let() {
    let result = eval_file("multi-let.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("multi-let.value.tan");

    assert_eq!(value, expected_value);
}

// #TODO add unit-test for function with multiple expressions.
