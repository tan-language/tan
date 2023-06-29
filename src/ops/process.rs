use crate::{ann::ANNO, error::Error, eval::env::Env, expr::Expr};

/// Terminates the current process with the specified exit code.
pub fn exit(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    if let Some(code) = args.first() {
        let ANNO(Expr::Int(code), ..) = code else {
            return Err(Error::invalid_arguments("expected Int argument", code.get_range()));
        };

        let code = *code as i32;

        std::process::exit(code);
    } else {
        // Exit with code=0 by default.
        std::process::exit(0);
    }
}

// #TODO args
// #TODO env
