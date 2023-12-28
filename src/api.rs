// #todo find a better name, e.g. `lang`, `sys`, `runtime`, tbh api is a good name though.

use std::path::Path;

use crate::{
    context::Context,
    error::Error,
    eval::eval,
    expr::Expr,
    lexer::{token::Token, Lexer},
    macro_expand::macro_expand,
    optimize::optimize,
    parser::Parser,
    prune::prune,
    // resolver::Resolver,
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

// #todo optimize this!
pub fn strip_tan_extension(path: impl Into<String>) -> String {
    let path = path.into();

    if let Some(path) = path.strip_suffix(&format!(".{TAN_FILE_EXTENSION}")) {
        path.to_owned()
    } else if let Some(path) = path.strip_suffix(&format!(".{TAN_FILE_EMOJI_EXTENSION}")) {
        path.to_owned()
    } else {
        path
    }
}

/// Lexes a Tan expression encoded as a text string.
pub fn lex_string(input: impl AsRef<str>) -> Result<Vec<Token>, Vec<Error>> {
    let input = input.as_ref();
    let mut lexer = Lexer::new(input);
    lexer.lex()
}

// #todo temp solution for compatibility.
// #todo remove this!
/// Parses a Tan expression encoded as a text string, returns first expression.
pub fn parse_string(input: impl AsRef<str>) -> Result<Expr, Vec<Error>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(&tokens);
    let mut expr = parser.parse()?;

    // #todo temp solution
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

// #todo should refactor
// #todo what is a good name?
/// Reads and resolves a Tan expression encoded as a text string.
/// Updates the environment with definitions.
pub fn resolve_string(
    input: impl AsRef<str>,
    context: &mut Context,
) -> Result<Vec<Expr>, Vec<Error>> {
    let exprs = parse_string_all(input)?;

    // dbg!(&exprs);

    // #todo also resolve static-use (normal use) here!

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
        // #insight this is the main read/analysis pipeline, it consists of passes or stages.
        // #todo better use the term `stage` (multi-stage programming)

        // #insight
        // Macro expansion should be performed before resolving.

        // Prune pass

        // #insight first prune pass needed before macro_expand.
        // #todo find a better name for the `prune` stage.

        let Some(expr) = prune(expr) else {
            // The expression is pruned (elided)
            continue;
        };

        // Expand macros.

        // #todo pass a dummy scope here? no need to polute the dyn-time environment with macro stuff.
        let expr = macro_expand(expr, context);

        // #todo temp hack until macro_expand returns multiple errors.
        let Ok(expr) = expr else {
            return Err(vec![expr.unwrap_err()]);
        };

        // #todo maybe a second `prune` pass is needed?

        let Some(expr) = expr else {
            // The expression is pruned (elided)
            // #insight elision can happen also in macro_expand!
            continue;
        };

        // Optimization pass

        // #todo should run after resolve?
        let expr = optimize(expr);

        // Resolve pass (typechecking, definitions, etc)

        // #todo should we push a new env?
        // let mut resolver = Resolver::new();
        // let expr = resolver.resolve(expr, context)?;

        resolved_exprs.push(expr);
    }

    Ok(resolved_exprs)
}

// #todo this implements in essence a do block. Maybe no value should be returned?
/// Evaluates a Tan expression encoded as a text string.
pub fn eval_string(input: impl AsRef<str>, context: &mut Context) -> Result<Expr, Vec<Error>> {
    let exprs = resolve_string(input, context)?;

    // println!("--- {}", &exprs[0]);
    // dbg!(&exprs);

    let mut last_value = Expr::One;

    for expr in exprs {
        let value = eval(&expr, context);

        let Ok(value) = value else {
            return Err(vec![value.unwrap_err()]);
        };

        last_value = value;
    }

    Ok(last_value)
}

#[cfg(test)]
mod tests {
    use crate::api::strip_tan_extension;

    #[test]
    fn strip_tan_extension_usage() {
        assert_eq!(strip_tan_extension("hello.tan"), "hello");
        assert_eq!(
            strip_tan_extension("deep/nesting/hello.tan"),
            "deep/nesting/hello"
        );
        assert_eq!(strip_tan_extension("emoji/works.👅"), "emoji/works");
    }
}
