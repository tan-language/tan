use std::fs;

use crate::{
    ann::Ann,
    error::Error,
    eval::env::Env,
    expr::{format_value, Expr},
    range::Ranged,
};

// #TODO do FFI functions really need an env?
// #TODO differentiate pure functions that do not change the env!

/// Writes one or more expressions to the STDOUT sink/stream.
pub fn write(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
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

pub fn writeln(args: &[Ann<Expr>], env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
    // #TODO nasty implementation!
    write(args, env)?;
    write(&[Expr::string("\n").into()], env)
}

// #TODO consider mapping `:` to `__` and use #[allow(snake_case)]

/// Reads the contents of a text file as a string.
pub fn file_read_as_string(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
    let [path] = args else {
        return Err(Error::invalid_arguments("`read_as_string` requires a `path` argument").into());
    };

    let Ann(Expr::String(path), ..) = path else {
        return Err(Error::invalid_arguments("`path` argument should be a String").into());
    };

    let contents = fs::read_to_string(path)?;

    Ok(Expr::String(contents).into())
}
