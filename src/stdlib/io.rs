use crate::{
    api::eval_string,
    context::Context,
    error::Error,
    expr::{format_value, Expr},
};

// #todo should take a &mut Context
pub fn read_string(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    // #todo support all `Stringable`s

    // #todo argh, temp ultra-hack.
    let mut context = Context::new();

    if let Some(input) = args.first() {
        let Some(input) = input.as_string() else {
            return Err(Error::invalid_arguments("expected String argument", input.range()));
        };

        // #todo think carefully which eval function to use.
        let result = eval_string(input, &mut context);

        if let Ok(expr) = result {
            Ok(expr)
        } else {
            // #todo something more clever needed here!
            // #todo use an aggregate Error, something like Error::failed_use()
            dbg!(&result);
            Err(Error::general("cannot read string, eval failed"))
        }
    } else {
        Err(Error::invalid_arguments("expected one argument", None))
    }
}

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
