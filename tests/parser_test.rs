use tan::{
    ann::Annotated,
    expr::Expr,
    lexer::{token::Token, Lexer},
    parser::Parser,
    range::Ranged,
    util::format::format_pretty_error,
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
    let expr = parser.parse();
    assert!(matches!(expr, Ok(Annotated(Expr::One, ..))));
}

#[test]
fn parse_reports_unexpected_tokens() {
    let input = ")";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let result = parser.parse();
    assert!(result.is_err());

    let err = result.unwrap_err();

    eprintln!("{}", format_pretty_error(&err, input, None));
}

// () == Expr::One (Unit)
#[test]
fn parse_handles_one() {
    let input = "()";
    let tokens = lex_tokens(input);
    let mut parser = Parser::new(tokens);

    let expr = parser.parse().unwrap();

    assert!(matches!(expr, Annotated(Expr::One, ..)));
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

    eprintln!("{}", format_pretty_error(&err, input, Some(filename)));
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
