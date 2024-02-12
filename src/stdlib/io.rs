use std::sync::Arc;

use crate::{
    api::resolve_string,
    context::Context,
    error::Error,
    expr::{format_value, Expr},
    util::module_util::require_module,
};

// #todo also register as a non-prelude module.

// #todo put example usage here!
pub fn read_string(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo support all `Stringable`s

    if let Some(input) = args.first() {
        let Some(input_str) = input.as_string() else {
            return Err(Error::invalid_arguments(
                "expected String argument",
                input.range(),
            ));
        };

        // #todo think carefully which eval function to use.
        // let result = eval_string(input, &mut context);
        let result = resolve_string(input_str, context);

        // #todo have a version of read_string tht returns ALL expressions?
        // #todo should return Expr::List(exprs) or a do block?
        if let Ok(expr) = result {
            // #todo temp fix
            // #todo think carefully here!
            let expr = expr.first().unwrap().clone(); // #todo double-argh!
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

// #todo do FFI functions really need an env?
// #todo differentiate pure functions that do not change the env!

// #todo (write ...) should take one string parameter and an optional stream/port parameter, like scheme
// #todo it could also get an Array/Seq parameter and join it, like JavaScript console.log
/// Writes one or more expressions to the STDOUT sink/stream.
pub fn write(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let output = args.iter().fold(String::new(), |mut str, x| {
        str.push_str(&format_value(x));
        str
    });

    // #todo shenanigans to handle `\n` in string, how can we do this better?
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

pub fn writeln(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo nasty implementation!
    write(args, context)?;
    write(&[Expr::string("\n")], context)
}

pub fn setup_lib_io(context: &mut Context) {
    let module = require_module("prelude", context);

    module.insert("read", Expr::ForeignFunc(Arc::new(read_string)));
    module.insert("read$$String", Expr::ForeignFunc(Arc::new(read_string)));

    module.insert("write", Expr::ForeignFunc(Arc::new(write)));
    module.insert("write$$String", Expr::ForeignFunc(Arc::new(write)));

    module.insert("writeln", Expr::ForeignFunc(Arc::new(writeln)));
    module.insert("writeln$$String", Expr::ForeignFunc(Arc::new(writeln)));
}
