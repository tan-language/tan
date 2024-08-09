use std::collections::HashMap;
// use std::io::{BufRead, BufReader};
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;

use crate::error::Error;
use crate::util::module_util::require_module;
use crate::{context::Context, expr::Expr};

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
pub fn process_exit(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo Investigate if using Tokio in FFI somehow messes the flushing of streams, especially compiler errors.
    // Flush the standard streams.
    std::io::stdout().flush().expect("stdout flushed");
    std::io::stderr().flush().expect("stderr flushed");

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

// #insight not used yet.
/// Return the process arguments as an array, includes the foreign ('host') arguments.
pub fn process_foreign_args(_args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let mut args = Vec::new();

    for arg in std::env::args() {
        args.push(Expr::string(arg))
    }

    Ok(Expr::array(args))
}

/// Return the process arguments as an array
pub fn process_args(_args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo warn in arguments passed!
    // #todo add unit-test

    let process_args = context.top_scope.get("**process-args**").unwrap();
    let process_args = process_args.as_array().unwrap();

    // #todo consider a ref expression!
    // #todo #hack #nasty crappy temp solution.
    let mut args = Vec::new();
    for arg in process_args.iter() {
        args.push(arg.clone());
    }
    Ok(Expr::array(args))
}

// #todo consider renaming to just `env`?
// #todo optionally support key/name argument to return the value of a specific env variable.
/// Return the process environment variables as a Map/Map.
pub fn process_env_vars(_args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let mut env_vars = HashMap::new();

    for (key, value) in std::env::vars() {
        env_vars.insert(key, Expr::string(value));
    }

    Ok(Expr::map(env_vars))
}

// #todo spawn
// #todo shell

// #todo
// pub fn process_spawn(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
//     let [cmd] = args else {
//         return Err(Error::invalid_arguments(
//             "`exec` requires `cmd` argument",
//             None,
//         ));
//     };

//     let Expr::String(cmd_string) = cmd.unpack() else {
//         return Err(Error::invalid_arguments(
//             "`cmd` argument should be a String", // #todo mention `Stringable` or `Stringer`
//             cmd.range(),
//         ));
//     };

//     let mut args = cmd_string.split(" ");

//     let Some(cmd) = args.next() else {
//         return Err(Error::invalid_arguments(
//             "`cmd` argument can't be empty",
//             cmd.range(),
//         ));
//     };

//     let Ok(output) = std::process::Command::new(cmd).args(args).output() else {
//         // #todo should be runtime error.
//         // #todo even more it should be a Tan error.
//         return Err(Error::general("failed to execute cmd `{cmd}`"));
//     };

//     // #todo also return status and stderr.
//     // #todo proper conversion of stdout output.
//     // #todo could return map {status, stdout, stderr}

//     Ok(Expr::string(
//         String::from_utf8(output.stdout).unwrap_or_default(),
//     ))
// }

// #todo rename to shell? or exec shell?
// #todo shortcut?
/// Similar to C's system function:
/// The command specified by string is passed to the host environment to be
/// executed by the command processor.
pub fn process_exec(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [cmd] = args else {
        return Err(Error::invalid_arguments(
            "`exec` requires `cmd` argument",
            None,
        ));
    };

    let Expr::String(cmd_string) = cmd.unpack() else {
        return Err(Error::invalid_arguments(
            "`cmd` argument should be a String", // #todo mention `Stringable` or `Stringer`
            cmd.range(),
        ));
    };

    let Ok(output) = std::process::Command::new("sh")
        .args(["-c", cmd_string])
        .output()
    else {
        // #todo should be runtime error.
        // #todo even more it should be a Tan error.
        return Err(Error::general("failed to execute cmd `{cmd}`"));
    };

    // #todo also return status and stderr.
    // #todo proper conversion of stdout output.
    // #todo could return map {status, stdout, stderr}

    Ok(Expr::string(
        String::from_utf8(output.stdout).unwrap_or_default(),
    ))
}

// #todo also implement a version that supports piping, with Tan streams/ports/channels.
// #todo the piping version should use Tokio/Async to implement multiplexing of STDOUT and STDERR.

// #todo find a better name, e.g. `exec-streaming`.
// #ref https://stackoverflow.com/questions/31992237/how-would-you-stream-output-from-a-process
/// The command specified by string is passed to the host environment to be
/// executed by the command processor.
/// The STDOUT and STDERR of the process are 'streamed' to the caller STDOUT, STDERR.
pub fn process_shell(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [cmd] = args else {
        return Err(Error::invalid_arguments(
            "`exec` requires `cmd` argument",
            None,
        ));
    };

    let Expr::String(cmd_string) = cmd.unpack() else {
        return Err(Error::invalid_arguments(
            "`cmd` argument should be a String", // #todo mention `Stringable` or `Stringer`
            cmd.range(),
        ));
    };

    let Ok(mut process) = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd_string)
        // .stdout(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    else {
        // #todo should be runtime error.
        // #todo even more it should be a Tan error.
        return Err(Error::general("failed to spawn cmd `{cmd}`"));
    };

    // let stdout = process.stdout.as_mut().unwrap();
    // let stdout_reader = BufReader::new(stdout);
    // let stdout_lines = stdout_reader.lines();

    // for line in stdout_lines {
    //     println!("{}", line.unwrap());
    // }

    let Ok(status) = process.wait() else {
        // #todo should be runtime error.
        // #todo even more it should be a Tan error.
        return Err(Error::general("failed to spawn cmd `{cmd}`"));
    };

    let Some(code) = status.code() else {
        // #todo how to handle this?
        return Err(Error::general("process terminated by signal, cmd `{cmd}`"));
    };

    Ok(Expr::Int(code as i64))
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

pub fn setup_lib_process(context: &mut Context) {
    let module = require_module("process", context);

    module.insert("exit", Expr::ForeignFunc(Arc::new(process_exit)));
    module.insert("exit$$", Expr::ForeignFunc(Arc::new(process_exit))); // #todo is this needed?

    // (let file (process/args 1))
    module.insert("args", Expr::ForeignFunc(Arc::new(process_args)));
    module.insert("args$$", Expr::ForeignFunc(Arc::new(process_args))); // #todo is this needed?

    // #todo
    // Better API:
    // - process/env -> Map with all variables
    // - process/env-var -> Query a variable by name, e.g. `(let var (process/env-var "VAR-NAME"))`

    // #todo (let tan-path (process/env :TANPATH))
    // (let tan-path ((process/env-vars) :TANPATH))
    module.insert("env-vars", Expr::ForeignFunc(Arc::new(process_env_vars)));
    module.insert("env-vars$$", Expr::ForeignFunc(Arc::new(process_env_vars))); // #todo is this needed?

    // (let output (process/exec "ls -al"))
    module.insert("exec", Expr::ForeignFunc(Arc::new(process_exec)));
    module.insert("exec$$String", Expr::ForeignFunc(Arc::new(process_exec)));

    module.insert("shell", Expr::ForeignFunc(Arc::new(process_shell)));
    module.insert("shell$$String", Expr::ForeignFunc(Arc::new(process_shell)));
}

// #todo add some tests, even without assertions, just to exercise these functions.
