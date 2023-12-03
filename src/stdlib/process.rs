use std::collections::HashMap;
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

/// Return the process arguments as an array
pub fn process_args(_args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let mut args = Vec::new();

    for arg in std::env::args() {
        args.push(Expr::string(arg))
    }

    Ok(Expr::array(args))
}

// #todo consider renaming to just `env`?
// #todo optionally support key/name argument to return the value of a specific env variable.
/// Return the process environment variables as a Dict/Map.
pub fn process_env_vars(_args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let mut env_vars = HashMap::new();

    for (key, value) in std::env::vars() {
        env_vars.insert(key, Expr::string(value));
    }

    Ok(Expr::dict(env_vars))
}

// https://stackoverflow.com/questions/21011330/how-do-i-invoke-a-system-command-and-capture-its-output

// #todo use one spawn for both string and ProcessSpec?
// #todo (process/spawn-cmd "ls -al") ; spawn-str, spawn-command, cmd, sh, shell, exec, run
// #todo (process/spawn-child child-process) -> Process(-Handle) ; just `spawn`
// #todo (Process env args id stdin, stdout stderr status current-dir)

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

    // (let tan-path (process/env :TANPATH))
    scope.insert("env-vars", Expr::ForeignFunc(Arc::new(process_env_vars)));
    scope.insert("env-vars$$", Expr::ForeignFunc(Arc::new(process_env_vars))); // #todo is this needed?

    // #todo this is a hack.
    let module_path = format!("{}/std/process", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}
