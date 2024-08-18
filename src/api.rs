// #todo find a better name, e.g. `lang`, `sys`, `runtime`, tbh api is a good name though.

use std::path::Path;

use crate::{
    check::check,
    context::Context,
    error::Error,
    eval::eval,
    expr::Expr,
    lexer::{token::Token, Lexer},
    library::path::get_full_extension,
    macro_expand::macro_expand,
    optimize::optimize,
    parser::Parser,
    prune::prune,
    range::Position,
};

pub const TAN_FILE_EXTENSION: &str = "tan";

pub const TAN_FILE_EMOJI_EXTENSION: &str = "👅";

// #todo Add unit test.
// #todo Implement strict version that checks exactly *.tan and skips *.*.tan, e.g. *.data.tan
pub fn has_tan_extension(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if let Some(extension) = path.extension() {
        extension == TAN_FILE_EXTENSION || extension == TAN_FILE_EMOJI_EXTENSION
    } else {
        false
    }
}

// #todo Add unit test
/// A strict version of has_tan_extension that checks exactly `*.tan` and
/// skips `*.*.tan`, e.g. `*.data.tan``.
pub fn has_tan_extension_strict(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if let Some(extension) = get_full_extension(&path.to_string_lossy()) {
        extension == TAN_FILE_EXTENSION || extension == TAN_FILE_EMOJI_EXTENSION
    } else {
        false
    }
}

// #todo Optimize this!
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

// #todo Merge with parse_string.
pub fn parse_string_with_position(
    input: impl AsRef<str>,
    start_position: Position,
) -> Result<Expr, Vec<Error>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input).with_position(start_position);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(&tokens).with_position(start_position);
    let mut expr = parser.parse()?;

    // #todo temp solution
    let expr = expr.swap_remove(0);

    Ok(expr)
}

// #insight Use the compile* functions if you don't need transient expressions in the AST.
/// Parses a Tan expression encoded as a text string, returns all expressions parsed.
pub fn parse_string_all(input: impl AsRef<str>) -> Result<Vec<Expr>, Vec<Error>> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(&tokens);
    let exprs = parser.parse()?;

    Ok(exprs)
}

// #todo find a better name.
pub fn compile(expr: Expr, context: &mut Context) -> Result<Expr, Vec<Error>> {
    // #insight this is the main read/analysis pipeline, it consists of passes or stages.
    // #todo better use the term `stage` (multi-stage programming)

    // #insight
    // Macro expansion should be performed before resolving.

    // Prune pass

    // #insight first prune pass needed before macro_expand.
    // #todo find a better name for the `prune` stage.

    let Some(expr) = prune(expr) else {
        // The expression is pruned (elided)
        // #todo what should be returned here?
        return Ok(Expr::None);
    };

    // Expand macros.

    // #todo pass a dummy scope here? no need to polute the dyn-time environment with macro stuff.
    let expr = macro_expand(expr, context);

    // #todo bug, macro_expand strips let annotation!

    // #todo temp hack until macro_expand returns multiple errors.
    let Ok(expr) = expr else {
        return Err(vec![expr.unwrap_err()]);
    };

    // #todo maybe a second `prune` pass is needed?

    let Some(expr) = expr else {
        // The expression is pruned (elided)
        // #insight elision can happen also in macro_expand!
        return Ok(Expr::None);
    };

    // Check pass
    // #todo confusion with upcoming `unchecked` keyword/concept.
    // #todo find a better name (validation?)
    // #todo move check after optimize? in resolve?
    let expr = check(expr);
    let Ok(expr) = expr else {
        return Err(vec![expr.unwrap_err()]);
    };

    // Optimization pass

    // #todo should run after resolve?
    let expr = optimize(expr);

    // Resolve pass (typechecking, definitions, etc)

    // #todo should we push a new env?
    // let mut resolver = Resolver::new();
    // let expr = resolver.resolve(expr, context)?;

    Ok(expr)
}

// #todo should it really update the context?
// #todo should refactor
/// Reads a Tan expression encoded as a text string, and 'compiles' it for evaluation.
/// Updates the context with definitions.
pub fn compile_string(
    input: impl AsRef<str>,
    context: &mut Context,
) -> Result<Vec<Expr>, Vec<Error>> {
    let exprs = parse_string_all(input)?;

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

    let mut compiled_exprs = Vec::new();

    for expr in exprs {
        // #todo should return option.
        let expr = compile(expr, context)?;
        if !expr.is_none() {
            compiled_exprs.push(expr);
        }
    }

    Ok(compiled_exprs)
}

// #todo a version where no context is passed, call it `exec_string` or `run_string`?
// #todo this implements in essence a do block. Maybe no value should be returned?
/// Evaluates a Tan expression encoded as a text string.
pub fn eval_string(input: impl AsRef<str>, context: &mut Context) -> Result<Expr, Vec<Error>> {
    let exprs = compile_string(input, context)?;

    let mut last_value = Expr::None;

    for expr in exprs {
        let value = eval(&expr, context);

        let Ok(value) = value else {
            return Err(vec![value.unwrap_err()]);
        };

        last_value = value;
    }

    Ok(last_value)
}

// #todo implement run() and run_string() (encapsulate Context)

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
