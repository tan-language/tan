mod common;

use std::num::IntErrorKind;

use tan::{
    ann::Ann,
    api::{parse_string, parse_string_all},
    error::Error,
    expr::Expr,
    lexer::{token::Token, Lexer},
    parser::Parser,
    range::Ranged,
};

fn read_input(filename: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
}

fn lex_tokens(input: &str) -> Vec<Ranged<Token>> {
    let mut lexer = Lexer::new(input);
    lexer.lex().unwrap()
}

#[test]
fn parse_handles_an_empty_token_list() {
    let input = &read_input("empty.tan");
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);
    let expr = parser.parse().unwrap();
    assert_eq!(expr.len(), 0);
}

#[test]
fn parse_handles_multiple_expressions() {
    let input = &read_input("multiple_expressions.tan");
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);
    let expr = parser.parse().unwrap();

    // The comment, TextSeparator, and 3 expressions.
    assert_eq!(expr.len(), 5);
}

#[test]
fn parse_reports_unexpected_tokens() {
    let input = ")";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let result = parser.parse();
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // eprintln!("{}", format_pretty_error(&err, input, None));

    assert_eq!(err.1.start, 0);
    assert_eq!(err.1.end, 1);

    let input = "]";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let result = parser.parse();
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // eprintln!("{}", format_pretty_error(&err, input, None));

    assert_eq!(err.1.start, 0);
    assert_eq!(err.1.end, 1);

    let input = "}";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let result = parser.parse();
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // eprintln!("{}", format_pretty_error(&err, input, None));

    assert_eq!(err.1.start, 0);
    assert_eq!(err.1.end, 1);
}

#[test]
fn parse_reports_multiple_unexpected_tokens() {
    let input = "(do (let a ]) (le b ]] 1))";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let result = parser.parse();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.len(), 3);
}

#[test]
fn parse_reports_quote_errors() {
    // Quote at EOF

    let input = "'";
    let result = parse_string(input);

    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    assert_eq!(err.1.start, 0);
    assert_eq!(err.1.end, 1);

    // #Insight we should allow consecutive quotes, emit a linter warning instead!

    // // Consecutive quotes

    // let input = "(let a '' 1)";
    // let result = parse_string(input);

    // assert!(result.is_err());

    // let err = result.unwrap_err();
    // let err = &err[0];

    // assert_eq!(err.1.start, 7);
    // assert_eq!(err.1.end, 8);
}

// () == Expr::One (Unit)
#[test]
fn parse_handles_one() {
    let input = "()";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let expr = parser.parse().unwrap();

    let expr = &expr[0];

    dbg!(&expr);

    assert!(matches!(expr, Ann(Expr::One, ..)));
}

#[test]
fn parse_handles_a_simple_expression() {
    let input = &read_input("hello_world.tan");
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let result = parser.parse();
    dbg!(&result);
}

#[test]
fn parse_reports_unterminated_lists() {
    let filename = "unterminated_list_expr.tan";
    let input = &read_input(filename);
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let result = parser.parse();
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // eprintln!("{}", format_pretty_error(&err, input, Some(filename)));

    assert_eq!(err.1.start, 20);
    assert_eq!(err.1.end, 34);
}

#[test]
fn parse_handles_annotations() {
    let input = r#"
    (let a #zonk #Int8 25 b #(inline true) 1)
    "#;
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let expr = parser.parse().unwrap();
    dbg!(&expr);
}

#[test]
fn parse_handles_multiline_whitespace() {
    let input = "(+ 1 2) \n\n(+ 3 4)";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let expr = parser.parse().unwrap();
    dbg!(&expr);
}

#[test]
fn parse_parses_arrays() {
    let input = r#"(let m ["george" "chris" "costas"])"#;
    let result = parse_string(input).unwrap();

    let Ann(Expr::List(exprs), ..) = result else {
        panic!("assertion failed: invalid form")
    };

    let Ann(Expr::List(ref exprs), ..) = exprs[2] else {
        panic!("assertion failed: invalid form")
    };

    assert!(matches!(&exprs[0], Ann(Expr::Symbol(s), ..) if s == "Array"));
    assert!(matches!(exprs.len(), 4));
}

#[test]
fn parse_parses_dicts() {
    let input = r#"(let m {"name" "george" "value" 1})"#;
    let expr = parse_string(input).unwrap();

    for e in expr.iter() {
        println!("-- {e:?}");
    }

    let Ann(Expr::List(exprs), ..) = expr else {
        panic!("assertion failed: invalid form")
    };

    let Ann(Expr::List(ref exprs), ..) = exprs[2] else {
        panic!("assertion failed: invalid form")
    };

    assert!(matches!(&exprs[0], Ann(Expr::Symbol(s), ..) if s == "Dict"));
    assert!(matches!(exprs.len(), 5));
}

// #TODO move to eval_test?
// #[test]
// fn parse_parses_dicts() {
//     let input = r##"(let m {"name" "george" "value" 1})"##;
//     let result = parse_string(input).unwrap();

//     let Ann(Expr::List(vec), ..) = result else {
//         panic!("assertion failed: invalid form")
//     };

//     assert!(matches!(&vec[2], Ann(Expr::Dict(dict), ..) if dict.len() == 2));
// }

#[test]
fn parse_detects_ints() {
    let input = "(let a 123)";
    let result = parse_string(input).unwrap();

    let Ann(Expr::List(vec), ..) = result else {
        panic!("invalid form")
    };

    assert!(matches!(&vec[2], Ann(Expr::Int(n), ..) if *n == 123));
}

#[test]
fn parse_detects_floats() {
    let input = "(let a 1_274.34)";
    let result = parse_string(input).unwrap();

    let Ann(Expr::List(vec), ..) = result else {
        panic!("invalid form")
    };

    assert!(matches!(&vec[2], Ann(Expr::Float(n), ..) if *n == 1274.34));
}

#[test]
fn parse_handles_numbers_with_radix() {
    let input = "(let a 0xfe)";
    let result = parse_string(input).unwrap();

    let Ann(Expr::List(vec), ..) = result else {
        panic!("invalid form")
    };

    assert!(matches!(&vec[2], Ann(Expr::Int(n), ..) if *n == 254));

    let input = "(let a 0b1010)";
    let result = parse_string(input).unwrap();

    let Ann(Expr::List(vec), ..) = result else {
        panic!("invalid form")
    };

    assert!(matches!(&vec[2], Ann(Expr::Int(n), ..) if *n == 10));

    let input = "(let a 0b00000)";
    let result = parse_string(input).unwrap();

    let Ann(Expr::List(vec), ..) = result else {
        panic!("invalid form")
    };

    assert!(matches!(&vec[2], Ann(Expr::Int(n), ..) if *n == 0));

    let input = "(let a 0o755)";
    let result = parse_string(input).unwrap();

    let Ann(Expr::List(vec), ..) = result else {
        panic!("invalid form")
    };

    assert!(matches!(&vec[2], Ann(Expr::Int(n), ..) if *n == 493));
}

#[test]
fn parse_reports_number_errors() {
    let input = "(+ 1 3$%99)";
    let result = parse_string(input);

    assert!(result.is_err());

    let err = result.unwrap_err();

    assert_eq!(err.len(), 1);

    let err = &err[0];

    assert!(matches!(err.0, Error::MalformedInt(..)));

    // eprintln!("{}", format_pretty_error(&err, input, None));

    if let Ranged(Error::MalformedInt(pie), range) = err {
        assert_eq!(pie.kind(), &IntErrorKind::InvalidDigit);
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 10);
    }
}

#[test]
fn parse_reports_multiple_number_errors() {
    let input = "(+ 1 3$%99 34%#$ 55$$4)";
    let result = parse_string(input);

    assert!(result.is_err());

    let err = result.unwrap_err();

    assert_eq!(err.len(), 3);
}

#[test]
fn parse_keeps_comments() {
    let input = "; This is a comment\n(+ 1 2)";
    let exprs = parse_string_all(input).unwrap();

    let expr = &exprs[0];
    assert!(matches!(expr, Ann(Expr::Comment(x), ..) if x == "; This is a comment"));
}
