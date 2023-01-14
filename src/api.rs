// #TODO find a better name, e.g. `lang`, `sys`, `runtime`.

use crate::{
    ann::Ann,
    error::Error,
    eval::{env::Env, eval},
    expr::Expr,
    lexer::Lexer,
    parser::Parser,
    range::Ranged,
    resolve::resolve,
    typecheck::resolve_type,
};

/// A Result specialization for Tan api functions.
pub type Result<T> = std::result::Result<T, Ranged<Error>>;

/// Parses a Tan expression encoded as a text string.
pub fn parse_string(input: impl AsRef<str>) -> Result<Ann<Expr>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    Ok(expr)
}

// #TODO is there any reason to return Ann<Expr>?
/// Evaluates a Tan expression encoded as a text string.
pub fn eval_string(input: impl AsRef<str>, env: &mut Env) -> Result<Expr> {
    let expr = parse_string(input)?;

    // #TODO should we push a new env?
    let expr = resolve_type(expr, env)?;

    let expr = resolve(&expr)?;

    let value = eval(expr, env)?;

    Ok(value)
}
