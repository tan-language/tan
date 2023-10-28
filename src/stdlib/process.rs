use std::{rc::Rc, sync::Arc};

use crate::error::Error;
use crate::{context::Context, expr::Expr, module::Module};

// #todo process/env, (let version (process/env :TAN-VERSION))
// #todo process/args, (let file (process/args 1)) (let file (1 (process/args)))

/// Terminates the current process with the specified exit code.
pub fn exit(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    if let Some(code) = args.first() {
        let Some(code) = code.as_int() else {
            return Err(Error::invalid_arguments("expected Int argument", code.range()));
        };

        std::process::exit(code as i32);
    } else {
        // Exit with code=0 by default.
        std::process::exit(0);
    }
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

    scope.insert("exit", Expr::ForeignFunc(Arc::new(exit)));
    scope.insert("exit$$", Expr::ForeignFunc(Arc::new(exit)));

    // #todo this is a hack.
    let module_path = format!("{}/std/process", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}
