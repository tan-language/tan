use crate::{
    error::Error,
    eval::env::Env,
    expr::{format_value, Expr},
};

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
