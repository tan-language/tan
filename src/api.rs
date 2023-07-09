// #TODO find a better name, e.g. `lang`, `sys`, `runtime`, tbh api is a good name though.

use std::path::Path;

use crate::{
    error::Error,
    eval::{env::Env, eval},
    expr::Expr,
    lexer::{token::Token, Lexer},
    macro_expand::macro_expand,
    optimize::optimize,
    parser::Parser,
    prune::prune,
    resolver::Resolver,
};

pub const TAN_FILE_EXTENSION: &str = "tan";

pub const TAN_FILE_EMOJI_EXTENSION: &str = "👅";

pub fn has_tan_extension(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if let Some(extension) = path.extension() {
        extension == TAN_FILE_EXTENSION || extension == TAN_FILE_EMOJI_EXTENSION
    } else {
        false
    }
}

// #todo argh! optimize this!!
pub fn strip_tan_extension(path: impl Into<String>) -> String {
    let path = path.into();

    if let Some(path) = path.strip_suffix(&format!(".{TAN_FILE_EXTENSION}")) {
        return path.to_owned();
    } else if let Some(path) = path.strip_suffix(&format!(".{TAN_FILE_EMOJI_EXTENSION}")) {
        return path.to_owned();
    } else {
        return path;
    }
}

/// Lexes a Tan expression encoded as a text string.
pub fn lex_string(input: impl AsRef<str>) -> Result<Vec<Token>, Vec<Error>> {
    let input = input.as_ref();
    let mut lexer = Lexer::new(input);
    lexer.lex()
}

// #TODO temp solution for compatibility.
// #TODO remove this!
/// Parses a Tan expression encoded as a text string, returns first expression.
pub fn parse_string(input: impl AsRef<str>) -> Result<Expr, Vec<Error>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(&tokens);
    let mut expr = parser.parse()?;

    // #TODO temp solution
    let expr = expr.swap_remove(0);

    Ok(expr)
}

/// Parses a Tan expression encoded as a text string, returns all expressions parsed.
pub fn parse_string_all(input: impl AsRef<str>) -> Result<Vec<Expr>, Vec<Error>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(&tokens);
    let exprs = parser.parse()?;

    Ok(exprs)
}

// #TODO what is a good name?
/// Reads and resolves a Tan expression encoded as a text string.
/// Updates the environment with definitions.
pub fn resolve_string(input: impl AsRef<str>, env: &mut Env) -> Result<Vec<Expr>, Vec<Error>> {
    let exprs = parse_string_all(input)?;

    // #insight
    //
    // AST -> Executable pipeline:
    //
    // - macro-expand
    // - resolve
    // - optimize

    // // Nice debugging tool!
    // for ex in &exprs {
    //     for e in ex.iter() {
    //         println!("-- {e:?}");
    //     }
    // }

    let mut resolved_exprs = Vec::new();

    for expr in exprs {
        // #Insight
        // Macro expansion should be performed before resolving.

        // Prune pass

        // #insight first prune pass needed before macro_expand.

        let Some(expr) = prune(expr) else {
            // The expression is pruned (elided)
            continue;
        };

        // Expand macros.

        let expr = macro_expand(expr, env);

        // #TODO temp hack until macro_expand returns multiple errors.
        let Ok(expr) = expr else {
            return Err(vec![expr.unwrap_err()]);
        };

        // #TODO maybe a second `prune` pass is needed?

        let Some(expr) = expr else {
            // The expression is pruned (elided)
            // #insight elision can happen also in macro_expand!
            continue;
        };

        // Optimization pass

        // #TODO should run after resolve?
        let expr = optimize(expr);

        // Resolve pass (typechecking, definitions, etc)

        // #TODO should we push a new env?
        let mut resolver = Resolver::new();
        let expr = resolver.resolve(expr, env)?;

        resolved_exprs.push(expr);
    }

    Ok(resolved_exprs)
}

// #TODO this implements in essence a do block. Maybe no value should be returned?
/// Evaluates a Tan expression encoded as a text string.
pub fn eval_string(input: impl AsRef<str>, env: &mut Env) -> Result<Expr, Vec<Error>> {
    let exprs = resolve_string(input, env)?;

    let mut last_value = Expr::One.into();

    for expr in exprs {
        let value = eval(&expr, env);

        let Ok(value) = value else {
            return Err(vec![value.unwrap_err()]);
        };

        last_value = value;
    }

    Ok(last_value)
}
