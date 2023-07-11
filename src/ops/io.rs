use std::fs;

use crate::{
    context::Context,
    error::Error,
    expr::{format_value, Expr},
};

// #TODO do FFI functions really need an env?
// #TODO differentiate pure functions that do not change the env!

// #TODO (write ...) should take one string parameter and an optional stream/port parameter, like scheme
// #TODO it could also get an Array/Seq parameter and join it, like JavaScript console.log
/// Writes one or more expressions to the STDOUT sink/stream.
pub fn write(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let output = args.iter().fold(String::new(), |mut str, x| {
        str.push_str(&format_value(x));
        str
    });

    // #TODO shenanigans to handle `\n` in string, how can we do this better?
    for line in output.split_inclusive("\\n") {
        if line.ends_with("\\n") {
            let mut line: String = line.to_owned();
            line.pop();
            line.pop();
            println!("{line}");
        } else {
            print!("{line}");
        }
    }

    Ok(Expr::One)
}

pub fn writeln(args: &[Expr], context: &Context) -> Result<Expr, Error> {
    // #TODO nasty implementation!
    write(args, context)?;
    write(&[Expr::string("\n")], context)
}

// File < Resource
// #TODO extract file-system-related functionality to `fs` or even the more general `rs` == resource space.
// #TODO consider mapping `:` to `__` and use #[allow(snake_case)]

/// Reads the contents of a text file as a string.
pub fn file_read_as_string(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments("`read_as_string` requires a `path` argument", None));
    };

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments("`path` argument should be a String", path.range()));
    };

    let contents = fs::read_to_string(path)?;

    Ok(Expr::String(contents))
}

// #TODO decide on the parameters order.
pub fn file_write_string(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [path, content] = args else {
        return Err(Error::invalid_arguments("`read_as_string` requires `path` and `content` arguments", None));
    };

    let Expr::String(path) = path.unpack() else {
        return Err(Error::invalid_arguments("`path` argument should be a String", path.range()));
    };

    let Expr::String(content) = content.unpack() else {
        return Err(Error::invalid_arguments("`content` argument should be a String", content.range()));
    };

    fs::write(path, content)?;

    Ok(Expr::One)
}
