mod common;

use assert_matches::assert_matches;

use tan::{
    api::eval_string,
    context::Context,
    error::{Error, ErrorKind},
    eval::{eval, util::eval_module},
    expr::{format_value, Expr},
    util::fmt::format_float,
};

use crate::common::{eval_file, read_file};

// #todo add more tests, especially for error-reporting.

#[test]
fn eval_processes_arithmetic_expressions() {
    let result = eval_file("sum.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("sum.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_processes_decimal_arithmetic_expressions() {
    let result = eval_file("sum-dec.tan");

    dbg!(&result);

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("sum-dec.value.tan");

    assert_eq!(value, expected_value);
}

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
    assert_matches!(err, Error{ kind: ErrorKind::UndefinedSymbol(s), .. } if s == "write33");
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
    let mut context = Context::new();
    let result = eval_string(":key", &mut context).unwrap();

    assert_matches!(result.unpack(), Expr::KeySymbol(x) if x == "key");
}

#[test]
fn eval_processes_empty_list() {
    let expr = Expr::List(Vec::new());

    let mut context = Context::new();
    let value = eval(&expr, &mut context).unwrap();

    assert_matches!(value.unpack(), Expr::One);
}

#[test]
fn eval_processes_let() {
    let mut context = Context::new();
    let result = eval_string("(do (let a (+ 1 2 3)) a)", &mut context);
    dbg!(&result);

    // #todo add asserts!
}

#[test]
fn eval_processes_booleans() {
    let mut context = Context::new();
    let value = eval_string("(do (let flag true) flag)", &mut context).unwrap();
    assert_matches!(value.unpack(), Expr::Bool(x) if *x);

    let value = eval_string("(do (let flag false) flag)", &mut context).unwrap();
    assert_matches!(value.unpack(), Expr::Bool(x) if !x);
}

#[test]
fn eval_processes_chars() {
    let mut context = Context::new();
    let value = eval_string(r#"(let ch (Char "r")) ch"#, &mut context).unwrap();
    assert_matches!(value.unpack(), Expr::Char(c) if *c == 'r');
}

#[test]
fn eval_reports_let_errors() {
    let mut context = Context::new();
    let result = eval_string("(do (let if (+ 1 2 3)) a)", &mut context);

    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // assert!(
    //     matches!(err, Error{ kind: ErrorKind::InvalidArguments(x), .. } if x == "let cannot shadow the reserved symbol `if`")
    // );

    assert_matches!(
        err,
        Error {
            kind: ErrorKind::InvalidArguments,
            ..
        }
    );

    let range = err.notes.first().unwrap().range.as_ref().unwrap();

    assert_eq!(range.start.index, 9);
    assert_eq!(range.end.index, 11);
}

// #todo extract full testing from file.

#[test]
fn eval_processes_quoted_expressions() {
    let result = eval_file("quoted.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("quoted.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_handles_unquoting() {
    let result = eval_file("unquoting.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("unquoting.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_handles_recursive_unquoting() {
    let result = eval_file("unquoting-recursive.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("unquoting-recursive.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn do_creates_new_lexical_scope() {
    let mut context = Context::new();
    let result = eval_string(
        "
    (do
        (let a 1)
        (do
            (let a (+ 1 2))
        )
        a
    )",
        &mut context,
    );
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = "1";

    assert_eq!(value, expected_value);
}

#[test]
fn ensure_a_infinite_recursion_is_fixed() {
    let mut context = Context::new();
    let result = eval_string(
        "
    (do
        (let a 1)
        (let a (+ a 2))
        a
    )",
        &mut context,
    );
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = "3";

    assert_eq!(value, expected_value);
}

#[test]
fn quot_handles_lists() {
    let mut context = Context::new();
    let result = eval_string("'(let a 1)", &mut context);
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = "(let a 1)";

    assert_eq!(value, expected_value);

    // #todo argh! cannot quote if expressions (and more)
    let mut context = Context::new();
    let result = eval_string("'(if \"a\" b 1)", &mut context);
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = "(if \"a\" b 1)";

    assert_eq!(value, expected_value);
}

#[test]
fn quot_handles_interpolation() {
    let mut context = Context::new();
    let input = "'(hello world (+ 1 0) $(+ 1 2))";
    let result = eval_string(input, &mut context);
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = "(hello world (+ 1 0) 3)";
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
fn eval_handles_function_with_no_params() {
    let result = eval_file("func-no-params.tan");
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("func-no-params.value.tan");
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
fn eval_processes_dict_with_keys() {
    let result = eval_file("key-dict.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("key-dict.value.tan");

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

#[test]
fn eval_handles_functions_with_multiple_expressions() {
    let result = eval_file("block-function.tan");

    dbg!(&result);
    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("block-function.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_handles_arrays() {
    let result = eval_file("array.tan");

    assert!(result.is_ok());

    let value = format!("{}", result.unwrap());
    let expected_value = read_file("array.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_escaped_strings() {
    let result = eval_file("string-escape.tan");

    assert!(result.is_ok());

    let value = format_value(result.unwrap());
    let expected_value = read_file("string-escape.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn eval_handles_nested_resolve() {
    let result = eval_file("nested-resolve.tan");

    assert!(result.is_ok());

    let value = format_value(result.unwrap());
    let expected_value = read_file("nested-resolve.value.tan");

    assert_eq!(value, expected_value);
}

// #todo make closures work, we need true lexical-scoping.
#[test]
fn eval_handles_closures() {
    // #todo rename the filename
    // #todo no need to use a *.value.tan file for simple results.
    let result = eval_file("pseudo-curry.tan");

    dbg!(&result);
    assert!(result.is_ok());

    let value = format_value(result.unwrap());
    let expected_value = read_file("pseudo-curry.value.tan");

    assert_eq!(value, expected_value);
}

#[test]
fn module_cannot_access_private_members_of_other_modules() {
    let mut context = Context::new();
    context
        .scope
        .insert("*current-module-path*", Expr::string("tests/fixtures"));
    let result = eval_module("./main.tan", &mut context, false);

    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_matches!(&err[0].kind, ErrorKind::UndefinedSymbol(sym) if sym == "submodule/afunc");
}

#[test]
fn eval_function_returns_dict() {
    let expr = eval_file("func-dict.tan").unwrap();

    let Some(dict) = expr.as_dict() else {
        panic!();
    };

    let body = dict.get("body");

    assert_matches!(body, Some(Expr::String(s)) if s == "quote: a quote");
}

#[test]
fn format_string_retains_escaped_chars() {
    let result = eval_file("format-str.tan");

    assert_matches!(result, Ok(Expr::String(s)) if s == r#"has \"escaped\" chars"#);
}

#[test]
fn format_float_has_fractional_part() {
    let result = eval_file("format-float.tan");

    assert_matches!(result, Ok(Expr::String(s)) if s == "1.0");
}

#[test]
fn for_let_regression() {
    let result = eval_file("for-let.tan");
    assert!(result.is_ok())
}

#[test]
fn eval_and() {
    let mut context = Context::new();
    let result = eval_string("(and true false)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if !b);

    let result = eval_string("(and false false false true)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if !b);

    let result = eval_string("(and true true (= 1 1))", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if b);

    let result = eval_string("(and true)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if b);
}

#[test]
fn eval_or() {
    let mut context = Context::new();
    let result = eval_string("(or true false)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if b);

    let result = eval_string("(or false false false true)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if b);

    let result = eval_string("(or false false false)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if !b);
}

#[test]
fn eval_eq() {
    let mut context = Context::new();
    let result = eval_string("(= 1 1)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if b);

    let mut context = Context::new();
    let result = eval_string("(= 1 2)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if !b);

    let mut context = Context::new();
    let result = eval_string(r#"(= "hello" "hello")"#, &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if b);

    let mut context = Context::new();
    let result = eval_string("(= :hello :hello)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if b);
}

#[test]
fn eval_not() {
    let mut context = Context::new();
    let result = eval_string("(not true)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if !b);

    let mut context = Context::new();
    let result = eval_string("(not false)", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if b);

    let mut context = Context::new();
    let result = eval_string("(not (= 1 1))", &mut context);
    assert_matches!(result, Ok(Expr::Bool(b)) if !b);
}

#[test]
fn eval_cond() {
    let mut context = Context::new();

    let expr = eval_string(
        r#"
        (do
            (let rank 3)
            (cond
                (> rank 10) :high
                (> rank 5)  :medium
                true        :low       ; use `else`
            )
        )
    "#,
        &mut context,
    )
    .unwrap();

    assert_matches!(expr.unpack(), Expr::KeySymbol(sym) if sym == "low");

    let expr = eval_string(
        r#"
        (do
            (let rank 3)
            (cond
                (> rank 10) :high
                (> rank 5)  :medium
                else        :low
            )
        )
    "#,
        &mut context,
    )
    .unwrap();

    assert_matches!(expr.unpack(), Expr::KeySymbol(sym) if sym == "low");

    let expr = eval_string(
        r#"
        (do
            (let rank 15)
            (cond
                (> rank 10) :high
                (> rank 5)  :medium
                else        :low
            )
        )
    "#,
        &mut context,
    )
    .unwrap();

    assert_matches!(expr.unpack(), Expr::KeySymbol(sym) if sym == "high");

    // #todo add extra tests to check for malformed conds.
}
