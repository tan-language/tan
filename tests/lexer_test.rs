mod common;

use tan::{
    error::Error,
    lexer::{token::Token, Lexer},
};

use crate::common::read_file;

#[test]
fn lex_handles_an_empty_string() {
    let input = read_file("empty.tan");
    let tokens = Lexer::new(&input).lex();

    let tokens = tokens.unwrap();

    assert_eq!(tokens.len(), 0);
}

#[test]
fn lex_returns_tokens() {
    let input = "((+ 1   25 399)  )";
    let tokens = Lexer::new(input).lex();

    let tokens = tokens.unwrap();

    dbg!(&tokens);

    assert_eq!(tokens.len(), 8);
    assert!(matches!(tokens[0].as_ref(), Token::LeftParen));
    assert!(matches!(tokens[2].as_ref(), Token::Symbol(x) if x == "+"));
    assert_eq!(tokens[2].1.start, 2);
    assert!(matches!(tokens[3].as_ref(), Token::Number(..)));
    assert_eq!(tokens[3].1.start, 4);
    // #TODO add more assertions.
}

#[test]
fn lex_parses_comments() {
    let input = "; This is a comment\n;; Another comment\n(write \"hello\"); end comment";
    let tokens = Lexer::new(input).lex();

    let tokens = tokens.unwrap();

    assert!(matches!(tokens[0].as_ref(), Token::Comment(x) if x == "; This is a comment"));
    assert!(matches!(tokens[1].as_ref(), Token::Comment(x) if x == ";; Another comment"));

    let c1 = &tokens[1];
    assert_eq!(c1.1.start, 20);
    assert_eq!(c1.1.end, 38);

    let c2 = &tokens[6];
    assert_eq!(c2.1.start, 54);
    assert_eq!(c2.1.end, input.len());
}

#[test]
fn lex_parses_dash_comments() {
    let input = "-- This is a comment\n-----\n(write \"hello\"); end comment";
    let tokens = Lexer::new(input).lex();

    let tokens = tokens.unwrap();

    assert!(matches!(tokens[0].as_ref(), Token::Comment(x) if x == "-- This is a comment"));
    assert!(matches!(tokens[1].as_ref(), Token::Comment(x) if x == "-----"));

    let c1 = &tokens[1];
    assert_eq!(c1.1.start, 21);
    assert_eq!(c1.1.end, 26);
}

#[test]
fn lex_parses_annotations() {
    let input = "
        #deprecated
        #(inline 'always)
        (let #public (add x y) (+ x y))
    ";
    let tokens = Lexer::new(input).lex();

    let tokens = tokens.unwrap();

    assert!(matches!(tokens[0].as_ref(), Token::Annotation(x) if x == "deprecated"));
    assert!(matches!(tokens[1].as_ref(), Token::Annotation(x) if x == "(inline 'always)"));
}

#[test]
fn lex_scans_ints() {
    let input = "(let a 123)";
    let tokens = Lexer::new(input).lex().unwrap();
    assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == "123"));
}

#[test]
fn lex_scans_floats() {
    let input = "(let a 1_274.34)";
    let tokens = Lexer::new(input).lex().unwrap();
    assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == "1274.34"));
}

#[test]
fn lex_scans_number_with_delimiters() {
    let input = r##"(let a {"score" 93})"##;
    let tokens = Lexer::new(input).lex().unwrap();

    assert!(matches!(tokens[5].as_ref(), Token::Number(n) if n == "93"));
}

#[test]
fn lex_scans_multiline_whitespace() {
    let input = "(+ 1 2) \n\n(+ 3 4)";
    let tokens = Lexer::new(input).lex().unwrap();

    assert!(matches!(tokens[5].as_ref(), Token::MultiLineWhitespace));
}

#[test]
fn lex_handles_number_separators() {
    let input = "(+ 1 3_000)";
    let tokens = Lexer::new(input).lex().unwrap();

    // dbg!(&tokens);

    assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == "3000"));
}

#[test]
fn lex_handles_signed_numbers() {
    let input = read_file("signed_number.tan");
    let tokens = Lexer::new(&input).lex();

    let tokens = tokens.unwrap();

    assert!(matches!(tokens[3].as_ref(), Token::Number(n) if n == "-123"));
    assert!(matches!(tokens[7].as_ref(), Token::Symbol(s) if s == "-variable"));
}

#[test]
fn lex_reports_unexpected_eof() {
    let input = "(let a -";
    let result = Lexer::new(input).lex();

    assert!(result.is_err());

    let err = result.unwrap_err();

    // eprintln!("{}", format_pretty_error(&err, input, None));

    assert!(matches!(err[0].0, Error::UnexpectedEnd));
}

#[test]
fn lex_reports_unterminated_strings() {
    let input = r##"(write "Hello)"##;
    let tokens = Lexer::new(input).lex();

    let result = tokens;

    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    assert!(matches!(err.0, Error::UnterminatedString));

    // eprintln!("{}", format_pretty_error(&err, input, None));

    assert_eq!(err.1.start, 7);
    assert_eq!(err.1.end, 14);
}

#[test]
fn lex_reports_unterminated_annotations() {
    let input = r##"
    #deprecated
    #(inline true
    (write "Hello)
    "##;
    let tokens = Lexer::new(input).lex();

    let result = tokens;

    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    assert!(matches!(err.0, Error::UnterminatedAnnotation));

    // eprintln!("{}", format_pretty_error(&err, input, None));

    assert_eq!(err.1.start, 21);
}
