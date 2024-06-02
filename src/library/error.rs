use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{args::unpack_stringable_arg, module_util::require_module},
};

// #todo error wrap.
// #todo error pretty-print / format-pretty
// #todo error variant (don't use the word `kind` reserved for type-system)

pub fn error_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let reason = unpack_stringable_arg(args, 0, "reason")?;
    Ok(Expr::error(reason))
}

pub fn setup_lib_error(context: &mut Context) {
    // #todo put in 'error' / 'err' or path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    // #todo consider `Err`.
    module.insert("Error", Expr::ForeignFunc(Arc::new(error_new)));
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn error_new_usage() {
        let mut context = Context::new();

        let input = r#"
            (Error "undefined symbol")
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = r#"(Error "undefined symbol")"#;
        assert_eq!(value, expected);
    }
}
