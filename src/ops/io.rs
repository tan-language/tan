use std::fs;

use crate::{
    error::Error,
    eval::env::Env,
    expr::{format_value, Expr},
};

// #TODO do FFI functions really need an env?
// #TODO differentiate pure functions that do not change the env!

/// Writes one or more expressions to the STDOUT sink/stream.
pub fn write(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
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

pub fn writeln(args: &[Expr], env: &Env) -> Result<Expr, Error> {
    // #TODO nasty implementation!
    write(args, env)?;
    write(&[Expr::string("\n")], env)
}

// File < Resource
// #TODO extract file-system-related functionality to `fs` or even the more general `rs` == resource space.
// #TODO consider mapping `:` to `__` and use #[allow(snake_case)]

/// Reads the contents of a text file as a string.
pub fn file_read_as_string(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
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
pub fn file_write_string(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
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
