mod common;

use assert_matches::assert_matches;

use tan::{
    api::{parse_string, parse_string_all},
    error::ErrorVariant,
    expr::{format_value, Expr},
    lexer::{token::Token, Lexer},
    parser::Parser,
};

fn read_input(filename: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
}

fn lex_tokens(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    lexer.lex().unwrap()
}

#[test]
fn parse_handles_an_empty_token_list() {
    let input = &read_input("empty.tan");
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);
    let expr = parser.parse().unwrap();
    assert_eq!(expr.len(), 0);
}

#[test]
fn parse_handles_multiple_expressions() {
    let input = &read_input("multiple-expressions.tan");
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);
    let expr = parser.parse().unwrap();

    // The comment, TextSeparator, and 3 expressions.
    assert_eq!(expr.len(), 5);
}

#[test]
fn parse_reports_unexpected_tokens() {
    let input = ")";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);

    let result = parser.parse();
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // eprintln!("{}", format_pretty_error(&err, input, None));

    let range = err.range().unwrap();

    assert_eq!(range.start.index, 0);
    assert_eq!(range.end.index, 1);

    let input = "]";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);

    let result = parser.parse();
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // eprintln!("{}", format_pretty_error(&err, input, None));

    // #todo introduce assert_range helper!

    let range = err.range().unwrap();

    assert_eq!(range.start.index, 0);
    assert_eq!(range.end.index, 1);

    let input = "}";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);

    let result = parser.parse();
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // eprintln!("{}", format_pretty_error(&err, input, None));

    let range = err.range().unwrap();

    assert_eq!(range.start.index, 0);
    assert_eq!(range.end.index, 1);
}

#[test]
fn parse_reports_multiple_unexpected_tokens() {
    let input = "(do (let a ]) (le b ]] 1))";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);

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

    let range = err.range().unwrap();

    assert_eq!(range.start.index, 0);
    assert_eq!(range.end.index, 1);

    // #insight we should allow consecutive quotes, emit a linter warning instead!

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
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse().unwrap();

    let expr = &expr[0];

    dbg!(&expr);

    assert_matches!(expr.unpack(), Expr::Nil);
}

#[test]
fn parse_handles_a_simple_expression() {
    let input = &read_input("hello-world.tan");
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);

    let result = parser.parse();
    dbg!(&result);
}

#[test]
fn parse_reports_unterminated_lists() {
    let filename = "unterminated-list-expr.tan";
    let input = &read_input(filename);
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);

    let result = parser.parse();
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    // eprintln!("{}", format_pretty_error(&err, input, Some(filename)));

    let range = err.range().unwrap();

    assert_eq!(range.start.index, 20);
    assert_eq!(range.end.index, 34);
}

#[test]
fn parse_handles_annotations() {
    let input = r#"
    (let a #zonk #U8 1 b #{:inline true :other 1} 3)
    "#;
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse().unwrap();

    let xs = expr[0].as_list().unwrap();
    let x = &xs[2];
    let ann = x.annotations().unwrap();
    assert_matches!(ann.get("type").unwrap(), Expr::Type(t) if t == "U8");
    assert_matches!(ann.get("zonk").unwrap(), Expr::Bool(b) if *b);

    let x = &xs[4];
    let ann = x.annotations().unwrap();
    assert_matches!(ann.get("inline").unwrap().unpack(), Expr::Bool(b) if *b);
    assert_matches!(ann.get("other").unwrap().unpack(), Expr::Int(n) if *n == 1);
}

#[test]
fn parse_handles_more_complex_annotations() {
    let input = r#"
    (let a #(Func [Int Int] Int) (Func [x y] (+ x y)))
    "#;
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse().unwrap();
    // dbg!(expr);
    let xs = expr[0].as_list().unwrap();
    let x = &xs[2];
    let ann = x.annotations().unwrap();
    dbg!(&ann);
    // #todo fix! not correct values!
}

#[test]
fn parse_keeps_correct_range_annotations() {
    // Test that the parser keeps the range information passed by the lexer.

    let input = "(+ a b)";
    let expr = parse_string(input).unwrap();

    let Expr::List(exprs) = expr.unpack() else {
        panic!("assertion failed: invalid form")
    };

    let expr = &exprs[1];

    let range = expr.range().unwrap();

    assert_eq!(range.start.index, 3);
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.col, 3);
}

#[test]
fn parse_handles_multiline_whitespace() {
    let input = "(+ 1 2) \n\n(+ 3 4)";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse().unwrap();
    dbg!(&expr);
}

// #todo find a better name for 'template_strings'
#[test]
fn parse_handles_template_strings() {
    let input =
        r#"(let m "An amount: $110.00. Here is a number: ${num}, and another: ${another-num}")"#;
    let result = parse_string(input).unwrap();

    let Expr::List(exprs) = result.unpack() else {
        panic!("assertion failed: invalid form")
    };

    // dbg!(exprs);

    // let Expr::List(ref exprs) = exprs[2].unpack() else {
    //     panic!("assertion failed: invalid form")
    // };

    // assert_matches!(&exprs[0].unpack(), Expr::Symbol(s) if s == "format");
    // assert_eq!(exprs.len(), 5);

    // #insight the template string is not transformed at the parsing stage.
    assert_eq!(exprs.len(), 3);

    // #todo test error handling.
}

#[test]
fn parse_parses_arrays() {
    let input = r#"(let m ["george" "chris" "costas"])"#;
    let result = parse_string(input).unwrap();

    let Expr::List(exprs) = result.unpack() else {
        panic!("assertion failed: invalid form")
    };

    let Expr::List(ref exprs) = exprs[2].unpack() else {
        panic!("assertion failed: invalid form")
    };

    assert_matches!(&exprs[0].unpack(), Expr::Symbol(s) if s == "Array");
    assert_matches!(exprs.len(), 4);
}

#[test]
fn parse_parses_maps() {
    let input = r#"(let m {"name" "george" "value" 1})"#;
    let expr = parse_string(input).unwrap();

    for e in expr.iter() {
        println!("-- {e:?}");
    }

    let Expr::List(exprs) = expr.unpack() else {
        panic!("assertion failed: invalid form")
    };

    let Expr::List(ref exprs) = exprs[2].unpack() else {
        panic!("assertion failed: invalid form")
    };

    assert_matches!(&exprs[0].unpack(), Expr::Symbol(s) if s == "Map");
    assert_matches!(exprs.len(), 5);
}

// #todo move to eval_test?
// #[test]
// fn parse_parses_maps() {
//     let input = r##"(let m {"name" "george" "value" 1})"##;
//     let result = parse_string(input).unwrap();

//     let Ann(Expr::List(vec), ..) = result else {
//         panic!("assertion failed: invalid form")
//     };

//     assert_matches!(&vec[2], Ann(Expr::Map(map), ..) if map.len() == 2);
// }

#[test]
fn parse_detects_ints() {
    let input = "(let a 123)";
    let result = parse_string(input).unwrap();

    let Expr::List(vec) = result.unpack() else {
        panic!("invalid form")
    };

    assert_matches!(&vec[2].unpack(), Expr::Int(n) if *n == 123);
}

#[test]
fn parse_detects_floats() {
    let input = "(let a 1_274.34)";
    let result = parse_string(input).unwrap();

    let Expr::List(vec) = result.unpack() else {
        panic!("invalid form")
    };

    assert_matches!(&vec[2].unpack(), Expr::Float(n) if *n == 1274.34);
}

#[test]
fn parse_handles_numbers_with_radix() {
    let input = "(let a 0xfe)";
    let result = parse_string(input).unwrap();

    let Expr::List(vec) = result.unpack() else {
        panic!("invalid form")
    };

    assert_matches!(&vec[2].unpack(), Expr::Int(n) if *n == 254);

    let input = "(let a 0b1010)";
    let result = parse_string(input).unwrap();

    let Expr::List(vec) = result.unpack() else {
        panic!("invalid form")
    };

    assert_matches!(&vec[2].unpack(), Expr::Int(n) if *n == 10);

    let input = "(let a 0b00000)";
    let result = parse_string(input).unwrap();

    let Expr::List(vec) = result.unpack() else {
        panic!("invalid form")
    };

    assert_matches!(&vec[2].unpack(), Expr::Int(n) if *n == 0);

    let input = "(let a 0o755)";
    let result = parse_string(input).unwrap();

    let Expr::List(vec) = result.unpack() else {
        panic!("invalid form")
    };

    assert_matches!(&vec[2].unpack(), Expr::Int(n) if *n == 493);
}

#[test]
fn parse_reports_number_errors() {
    let input = "(+ 1 3$%99)";
    let result = parse_string(input);

    assert!(result.is_err());

    let err = result.unwrap_err();

    assert_eq!(err.len(), 1);

    let err = &err[0];

    assert_matches!(err.variant(), ErrorVariant::MalformedInt);

    // eprintln!("{}", format_pretty_error(&err, input, None));

    // #todo bring this back!!
    // if let Ranged(Error::MalformedInt(pie), range) = err {
    //     assert_eq!(pie.kind(), &IntErrorKind::InvalidDigit);
    //     assert_eq!(range.start, 5);
    //     assert_eq!(range.end, 10);
    // }
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
    assert_matches!(expr.unpack(), Expr::Comment(x, ..) if x == "; This is a comment");
}

// #todo use assert_matches!
#[test]
fn parse_handles_int_range() {
    let input = "(let a 2..30|3)";
    let result = parse_string(input).unwrap();

    let Expr::List(exprs) = result.unpack() else {
        panic!("invalid form")
    };

    // #insight we no longer build Range expressions in parser (to support dynamic expressions)
    // assert_matches!(&exprs[2].unpack(), Expr::IntRange(start, end, step) if *start == 2 && *end == 30 && *step == 3);

    let value = format_value(&exprs[2]);
    let expected = "(Range 2 30 3)";
    assert_eq!(value, expected);
}

#[test]
fn parse_handles_float_range() {
    let input = "(let a 2.1..30.2|0.1)";
    let result = parse_string(input).unwrap();

    let Expr::List(exprs) = result.unpack() else {
        panic!("invalid form")
    };

    // #insight we no longer build Range expressions in parser (to support dynamic expressions)
    // assert_matches!(&exprs[2].unpack(), Expr::FloatRange(start, end, step) if *start == 2.0 && *end == 30.0 && *step == 3.0);

    let value = format_value(&exprs[2]);
    let expected = "(Range 2.1 30.2 0.1)";
    assert_eq!(value, expected);
}

#[test]
fn parse_handles_ellipsis() {
    // #insight invalid tan, but good enough for this test.
    let input = "(let ... 1)";
    let result = parse_string(input).unwrap();

    let Expr::List(exprs) = result.unpack() else {
        panic!("invalid form")
    };

    assert_matches!(&exprs[1].unpack(), Expr::Symbol(s) if s == "...");
}
