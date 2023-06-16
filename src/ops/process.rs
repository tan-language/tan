use crate::{ann::Ann, error::Error, eval::env::Env, expr::Expr};

/// Terminates the current process with the specified exit code.
pub fn exit(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
    if let Some(code) = args.first() {
        let Ann(Expr::Int(code), ..) = code else {
            return Err(Error::InvalidArguments("expected Int argument".to_owned()).into());
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
