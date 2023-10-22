mod common;

use assert_matches::assert_matches;
use tan::{
    error::ErrorKind,
    lexer::{token::TokenKind, Lexer},
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
    assert!(matches!(tokens[0].kind(), TokenKind::LeftParen));
    assert!(matches!(tokens[2].kind(), TokenKind::Symbol(lexeme) if lexeme == "+"));
    assert_eq!(tokens[2].range().start.index, 2);
    assert!(matches!(tokens[3].kind(), TokenKind::Number(..)));
    assert_eq!(tokens[3].range().start.index, 4);
    assert_eq!(tokens[3].range().start.line, 0);
    assert_eq!(tokens[3].range().start.col, tokens[3].range().start.index);
    // #todo add more assertions.
}

#[test]
fn lex_parses_comments() {
    let input = "; This is a comment\n;; Another comment\n(write \"hello\"); end comment";
    let tokens = Lexer::new(input).lex();

    let tokens = tokens.unwrap();

    assert!(
        matches!(tokens[0].kind(), TokenKind::Comment(lexeme, ..) if lexeme == "; This is a comment")
    );
    assert!(
        matches!(tokens[1].kind(), TokenKind::Comment(lexeme, ..) if lexeme == ";; Another comment")
    );

    let r1 = &tokens[1].range();
    assert_eq!(r1.start.index, 20);
    assert_eq!(r1.start.line, 1);
    assert_eq!(r1.start.col, 0);
    assert_eq!(r1.end.index, 38);
    assert_eq!(r1.end.line, 1);
    assert_eq!(r1.end.col, 18);

    let r2 = &tokens[6].range();
    assert_eq!(r2.start.index, 54);
    assert_eq!(r2.end.index, input.len());
}

// `--` line comments no longer supported.
// #[test]
// fn lex_parses_dash_comments() {
//     let input = "-- This is a comment\n-----\n(write \"hello\"); end comment";
//     let tokens = Lexer::new(input).lex();

//     let tokens = tokens.unwrap();

//     assert!(matches!(tokens[0].as_ref(), Token::Comment(x) if x == "-- This is a comment"));
//     assert!(matches!(tokens[1].as_ref(), Token::Comment(x) if x == "-----"));

//     let c1 = &tokens[1];
//     assert_eq!(c1.1.start, 21);
//     assert_eq!(c1.1.end, 26);
// }

#[test]
fn lex_parses_annotations() {
    let input = "
        #deprecated
        #(inline 'always)
        (let #public (add x y) (+ x y))
    ";
    let tokens = Lexer::new(input).lex();

    let tokens = tokens.unwrap();

    assert!(matches!(tokens[0].kind(), TokenKind::Annotation(lexeme) if lexeme == "deprecated"));
    assert!(
        matches!(tokens[1].kind(), TokenKind::Annotation(lexeme) if lexeme == "(inline 'always)")
    );
}

#[test]
fn lex_scans_ints() {
    let input = "(let a 123)";
    let tokens = Lexer::new(input).lex().unwrap();
    assert!(matches!(tokens[3].kind(), TokenKind::Number(lexeme) if lexeme == "123"));
}

#[test]
fn lex_scans_floats() {
    let input = "(let a 1_274.34)";
    let tokens = Lexer::new(input).lex().unwrap();
    assert!(matches!(tokens[3].kind(), TokenKind::Number(lexeme) if lexeme == "1274.34"));
}

#[test]
fn lex_scans_number_with_delimiters() {
    let input = r##"(let a {"score" 93})"##;
    let tokens = Lexer::new(input).lex().unwrap();

    assert!(matches!(tokens[5].kind(), TokenKind::Number(lexeme) if lexeme == "93"));
}

#[test]
fn lex_scans_multiline_whitespace() {
    let input = "(+ 1 2) \n\n(+ 3 4)";
    let tokens = Lexer::new(input).lex().unwrap();

    assert!(matches!(tokens[5].kind(), TokenKind::MultiLineWhitespace));
}

#[test]
fn lex_handles_shebang_line() {
    let input = "#!/usr/bin/sh tan\n(writeln (+ 2 3)))\n";
    let tokens = Lexer::new(input).lex().unwrap();

    assert!(matches!(tokens[0].kind(), TokenKind::LeftParen));
}

#[test]
fn lex_handles_number_separators() {
    let input = "(+ 1 3_000)";
    let tokens = Lexer::new(input).lex().unwrap();

    assert!(matches!(tokens[3].kind(), TokenKind::Number(lexeme) if lexeme == "3000"));
}

#[test]
fn lex_handles_signed_numbers() {
    let input = read_file("signed-number.tan");
    let tokens = Lexer::new(&input).lex();

    let tokens = tokens.unwrap();

    assert!(matches!(tokens[3].kind(), TokenKind::Number(lexeme) if lexeme == "-123"));
    assert!(matches!(tokens[7].kind(), TokenKind::Symbol(lexeme) if lexeme == "-variable"));
}

#[test]
fn lex_reports_unexpected_eof() {
    let input = "(let a -";
    let result = Lexer::new(input).lex();

    assert!(result.is_err());

    let err = result.unwrap_err();

    // eprintln!("{}", format_pretty_error(&err, input, None));

    assert!(matches!(err[0].kind(), ErrorKind::UnexpectedEnd));
}

#[test]
fn lex_reports_unterminated_strings() {
    let input = r##"(write "Hello)"##;
    let tokens = Lexer::new(input).lex();

    let result = tokens;

    assert!(result.is_err());

    let err = result.unwrap_err();
    let err = &err[0];

    assert!(matches!(err.kind(), ErrorKind::UnterminatedString));

    // eprintln!("{}", format_pretty_error(&err, input, None));

    let range = err.range().unwrap();

    // #todo add tests for line, col.

    assert_eq!(range.start.index, 7);
    assert_eq!(range.end.index, 14);
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

    assert!(matches!(err.kind(), ErrorKind::UnterminatedAnnotation));

    // eprintln!("{}", format_pretty_error(&err, input, None));

    let range = err.range().unwrap();

    assert_eq!(range.start.index, 21);
}

#[test]
fn lex_handles_quasiquoting() {
    let input = "'(hello world $(cos 1))";
    let tokens = Lexer::new(input).lex().unwrap();

    assert_matches!(tokens[0].kind(), TokenKind::Quote);
    assert_matches!(tokens[4].kind(), TokenKind::Unquote);
}
