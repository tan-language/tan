// #todo Maybe as optimization, use special handling in eval?

use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{args::unpack_int_arg, module_util::require_module},
};

// #todo implement trait without context.
pub fn u8_new(args: &[Expr]) -> Result<Expr, Error> {
    // #todo support more 'source' types.
    let value = unpack_int_arg(args, 0, "value")?;

    if !(0..256).contains(&value) {
        return Err(Error::invalid_arguments(
            "U8 values should be in 0..256",
            args[0].range(),
        ));
    }

    Ok(Expr::U8(value as u8))
}

// #todo rename all setup_xxx functions to import_xxx.
pub fn setup_lib_u8(context: &mut Context) {
    // #todo put in 'u8' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    module.insert("U8", Expr::foreign_func(&u8_new));
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::{api::eval_string, context::Context, expr::Expr};

    #[test]
    fn char_new_usage() {
        let mut context = Context::new();

        let input = r#"(U8 12)"#;
        let expr = eval_string(input, &mut context).unwrap();
        assert_matches!(expr, Expr::U8(n) if n == 12);

        let input = r#"(U8 300)"#;
        let result = eval_string(input, &mut context);
        assert!(result.is_err());

        let input = r#"(U8 -8)"#;
        let result = eval_string(input, &mut context);
        assert!(result.is_err());
    }
}
