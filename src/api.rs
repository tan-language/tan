// #TODO find a better name, e.g. `lang`, `sys`, `runtime`.

use crate::{
    ann::Ann,
    error::Error,
    error2::ParseError,
    eval::{env::Env, eval},
    expr::Expr,
    lexer::Lexer,
    parser::Parser,
    range::Ranged,
    typecheck::resolve_type,
};

/// A Result specialization for Tan api functions.
pub type Result<T> = std::result::Result<T, Ranged<Error>>;

/// Parses a Tan expression encoded as a text string.
pub fn parse_string(
    input: impl AsRef<str>,
) -> std::result::Result<Ann<Expr>, Vec<Ranged<ParseError>>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    // #TODO WARNING temporary!
    // let tokens = lexer.lex()?;
    let tokens = lexer.lex().unwrap();

    let mut parser = Parser::new(tokens);
    // #TODO WARNING temporary!
    let expr = parser.parse()?;

    Ok(expr)
}

/// Evaluates a Tan expression encoded as a text string.
pub fn eval_string(input: impl AsRef<str>, env: &mut Env) -> Result<Ann<Expr>> {
    // #TODO WARNING temporary!
    // let expr = parse_string(input)?;
    let expr = parse_string(input).unwrap();

    // #TODO should we push a new env?
    let expr = resolve_type(expr, env)?;

    let value = eval(&expr, env)?;

    Ok(value)
}
