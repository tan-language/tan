use std::{rc::Rc, sync::Arc};

use crate::error::Error;
use crate::{context::Context, expr::Expr, module::Module};

// https://doc.rust-lang.org/std/env/index.html

// #todo process/env, (let version (process/env :TAN-VERSION))
// #todo process/args, (let file (process/args 1)) (let file (1 (process/args)))

// #todo move env-vars and args to an env package, like Rust?

// #todo process/vars or process/env or process/env-vars

// (let debug (process/vars "DEBUG" true))
// (let args (process/args))
// (for (arg (process/args))
//     (writeln arg)
// )
// (let args (process/args->map))
// (let debug (args "debug"))

/// Terminates the current process with the specified exit code.
pub fn process_exit(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    if let Some(code) = args.first() {
        let Some(code) = code.as_int() else {
            return Err(Error::invalid_arguments(
                "expected Int argument",
                code.range(),
            ));
        };

        std::process::exit(code as i32);
    } else {
        // Exit with code=0 by default.
        std::process::exit(0);
    }
}

// #todo probably these FFI functions should just return an Expr, no Result.

/// Return the process arguments.
pub fn process_args(_args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let mut args = Vec::new();

    for arg in std::env::args() {
        args.push(Expr::string(arg.clone()))
    }

    Ok(Expr::array(args))
}

// #todo args
// #todo env

// #todo consider removing the `std` prefix from module paths, like haskell.
// #todo find a better prefix than setup_
// #todo use Rc/Arc consistently
// #todo some helpers are needed here, to streamline the code.

pub fn setup_std_process(context: &mut Context) {
    let module = Module::new("process", context.top_scope.clone());

    let scope = &module.scope;

    scope.insert("exit", Expr::ForeignFunc(Arc::new(process_exit)));
    scope.insert("exit$$", Expr::ForeignFunc(Arc::new(process_exit))); // #todo is this needed?

    // (let file (process/args 1))
    scope.insert("args", Expr::ForeignFunc(Arc::new(process_args)));
    scope.insert("args$$", Expr::ForeignFunc(Arc::new(process_args))); // #todo is this needed?

    // #todo this is a hack.
    let module_path = format!("{}/std/process", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}
