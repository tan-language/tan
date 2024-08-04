use std::sync::Arc;

use crate::{
    api::compile_string,
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

        // #todo should create a throwaway context instead?

        // #todo think carefully which eval function to use.
        // let result = eval_string(input, &mut context);
        let result = compile_string(input_str, context);

        // #todo have a version of read_string that returns ALL expressions?
        // #todo should return Expr::List(exprs) or a do block?
        if let Ok(exprs) = result {
            // #todo temp fix
            // #todo think carefully here!
            let expr = exprs.first().unwrap().clone(); // #todo double-argh!
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

// #todo temp solution, think about read_string vs read_string_all
// #todo maybe read_string should get an opts parameter?
// #todo implement (eval-string ..)
// #todo put example usage here!
pub fn read_string_all(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo support all `Stringable`s

    if let Some(input) = args.first() {
        let Some(input_str) = input.as_string() else {
            return Err(Error::invalid_arguments(
                "expected String argument",
                input.range(),
            ));
        };

        // #todo should create a throwaway context instead?

        // #todo think carefully which eval function to use.
        // let result = eval_string(input, &mut context);
        let result = compile_string(input_str, context);

        // #todo have a version of read_string tht returns ALL expressions?
        // #todo should return Expr::List(exprs) or a do block?
        if let Ok(exprs) = result {
            // #todo temp fix
            // #todo think carefully here!
            Ok(Expr::array(exprs))
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

// #todo Not that write can also write bytes and does not interspence spaces.
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

    Ok(Expr::None)
}

pub fn writeln(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo nasty implementation!
    write(args, context)?;
    write(&[Expr::string("\n")], context)
}

// #insight Note that `echo` is different than `writeln`.
// #todo offer a version that returns a string and does not print, similar to `join`. Maybe `join-all`?
// #todo #think Actually writeln can only work on strings.
pub fn echo(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let output: Vec<String> = args.iter().map(format_value).collect();
    // #insight Don't make the separator customizable, just use string interpolation or (intersperse ...) instead.
    // #insight Intersperse spaces to emulate JavaScript's `console.log`` behavior.
    let output = output.join(" ") + "\n";

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

    Ok(Expr::None)
}

pub fn setup_lib_io(context: &mut Context) {
    let module = require_module("prelude", context);

    // #todo separate read/read-string.

    module.insert("read", Expr::ForeignFunc(Arc::new(read_string)));
    module.insert("read$$String", Expr::ForeignFunc(Arc::new(read_string)));

    module.insert(
        "read-string-all",
        Expr::ForeignFunc(Arc::new(read_string_all)),
    );
    module.insert(
        "read-string-all$$String",
        Expr::ForeignFunc(Arc::new(read_string_all)),
    );

    module.insert("write", Expr::ForeignFunc(Arc::new(write)));
    module.insert("write$$String", Expr::ForeignFunc(Arc::new(write)));

    module.insert("writeln", Expr::ForeignFunc(Arc::new(writeln)));
    module.insert("writeln$$String", Expr::ForeignFunc(Arc::new(writeln)));

    // #todo temp implementation echo is different than writeln.
    module.insert("echo", Expr::ForeignFunc(Arc::new(echo)));
    module.insert("echo$$String", Expr::ForeignFunc(Arc::new(echo)));
}
