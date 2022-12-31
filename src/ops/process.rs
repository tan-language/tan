use crate::{
    eval::{env::Env, error::EvalError},
    expr::Expr,
};

/// Exits the process.
pub fn exit(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    if let Some(code) = args.first() {
        let Expr::Int(code) = code else {
            return Err(EvalError::ArgumentError("expected Int argument".to_owned()));
        };

        let code = *code as i32;

        std::process::exit(code);
    } else {
        std::process::exit(0);
    }
}
