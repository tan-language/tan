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
pub fn lex_string(input: impl AsRef<str>) -> Result<Vec<Ranged<Token>>, Vec<Ranged<Error>>> {
    let input = input.as_ref();
    let mut lexer = Lexer::new(input);
    lexer.lex()
}

// #TODO temp solution for compatibility.
/// Parses a Tan expression encoded as a text string, returns first expression.
pub fn parse_string(input: impl AsRef<str>) -> Result<Ann<Expr>, Vec<Ranged<Error>>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(tokens);
    let mut expr = parser.parse()?;

    // #TODO temp solution
    let expr = expr.swap_remove(0);

    Ok(expr)
}

/// Parses a Tan expression encoded as a text string, returns all expressions parsed.
pub fn parse_string_all(input: impl AsRef<str>) -> Result<Vec<Ann<Expr>>, Vec<Ranged<Error>>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    Ok(expr)
}

// #TODO this implements in essence a do block. Maybe no value should be returned?
/// Evaluates a Tan expression encoded as a text string.
pub fn eval_string(input: impl AsRef<str>, env: &mut Env) -> Result<Ann<Expr>, Vec<Ranged<Error>>> {
    let exprs = parse_string_all(input)?;

    let mut last_value = Expr::One.into();

    for expr in exprs {
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

        // println!("--->>> {expr}");

        let value = eval(&expr, env);

        let Ok(value) = value else {
            return Err(vec![value.unwrap_err()]);
        };

        last_value = value;
    }

    Ok(last_value)
}
