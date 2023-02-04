// #TODO find a better name, e.g. `lang`, `sys`, `runtime`.

use crate::{
    ann::Ann,
    error::Error,
    eval::{env::Env, eval},
    expr::Expr,
    lexer::{token::Token, Lexer},
    macro_expand::macro_expand,
    parser::Parser,
    range::Ranged,
    resolver::Resolver,
};

/// Lexes a Tan expression encoded as a text string.
pub fn lex_string(
    input: impl AsRef<str>,
) -> std::result::Result<Vec<Ranged<Token>>, Vec<Ranged<Error>>> {
    let input = input.as_ref();
    let mut lexer = Lexer::new(input);
    lexer.lex()
}

/// Parses a Tan expression encoded as a text string.
pub fn parse_string(input: impl AsRef<str>) -> std::result::Result<Ann<Expr>, Vec<Ranged<Error>>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    Ok(expr)
}

/// Evaluates a Tan expression encoded as a text string.
pub fn eval_string(
    input: impl AsRef<str>,
    env: &mut Env,
) -> std::result::Result<Ann<Expr>, Vec<Ranged<Error>>> {
    let expr = parse_string(input)?;

    // #TODO should we push a new env?
    let mut resolver = Resolver::new();
    let expr = resolver.resolve(expr, env)?;

    let expr = macro_expand(expr, env);

    // #TODO temp hack until macro_expand returns multiple errors.
    let Ok(expr) = expr else {
        return Err(vec![expr.unwrap_err()]);
    };

    let Some(expr) = expr else {
        // #TODO more precise error needed here.
        return Err(vec!(Error::UnexpectedEnd {}.into()));
    };

    dbg!(&expr);

    let value = eval(&expr, env);

    let Ok(value) = value else {
        return Err(vec![value.unwrap_err()]);
    };

    Ok(value)
}
