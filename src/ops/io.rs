use std::fs;

use crate::{
    ann::Ann,
    error::Error,
    eval::env::Env,
    expr::{format_value, Expr},
    range::Range,
};

// #TODO do FFI functions really need an env?
// #TODO differentiate pure functions that do not change the env!

/// Writes one or more expressions to the STDOUT sink/stream.
pub fn write(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
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

    Ok(Expr::One.into())
}

pub fn writeln(args: &[Ann<Expr>], env: &Env) -> Result<Ann<Expr>, Error> {
    // #TODO nasty implementation!
    write(args, env)?;
    write(&[Expr::string("\n").into()], env)
}

// File < Resource
// #TODO extract file-system-related functionality to `fs` or even the more general `rs` == resource space.
// #TODO consider mapping `:` to `__` and use #[allow(snake_case)]

/// Reads the contents of a text file as a string.
pub fn file_read_as_string(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments("`read_as_string` requires a `path` argument", Range::default())); // #TODO range upstream
    };

    let Ann(Expr::String(path), ..) = path else {
        return Err(Error::invalid_arguments("`path` argument should be a String", path.get_range()));
    };

    let contents = fs::read_to_string(path)?;

    Ok(Expr::String(contents).into())
}

// #TODO decide on the parameters order.
pub fn file_write_string(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
    let [path, content] = args else {
        return Err(Error::invalid_arguments("`read_as_string` requires `path` and `content` arguments", Range::default())); // #TODO range upstream.
    };

    let Ann(Expr::String(path), ..) = path else {
        return Err(Error::invalid_arguments("`path` argument should be a String", path.get_range()));
    };

    let Ann(Expr::String(content), ..) = content else {
        return Err(Error::invalid_arguments("`content` argument should be a String", content.get_range()));
    };

    fs::write(path, content)?;

    Ok(Expr::One.into())
}
