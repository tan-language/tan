use crate::{
    ann::Ann,
    error::Error,
    eval::env::Env,
    expr::{format_value, Expr},
    range::Ranged,
};

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
