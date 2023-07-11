use crate::{context::Context, error::Error, expr::Expr};

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

// #TODO args
// #TODO env
