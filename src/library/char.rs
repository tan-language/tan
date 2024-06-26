// #todo maybe as optimization, use special handling in eval?

// #todo create char from Num/Int also
// #todo function to get char int code
// #todo function to join chars into a string

use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{args::unpack_stringable_arg, module_util::require_module},
};

// #todo implement trait without context.
pub fn char_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo also support Int as argument.
    let c = unpack_stringable_arg(args, 0, "c")?;

    if c.len() != 1 {
        return Err(Error::invalid_arguments(
            "the string argument should be one char long",
            args[0].range(),
        ));
    }

    let c = c.chars().next().unwrap();

    Ok(Expr::Char(c))
}

// #todo rename all setup_xxx functions to import_xxx.
pub fn setup_lib_char(context: &mut Context) {
    // #todo put in 'char' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    module.insert("Char", Expr::ForeignFunc(Arc::new(char_new)));
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::{api::eval_string, context::Context, expr::Expr};

    #[test]
    fn char_new_usage() {
        let mut context = Context::new();

        let input = r#"(Char "c")"#;
        let expr = eval_string(input, &mut context).unwrap();
        assert_matches!(expr, Expr::Char(c) if c == 'c');

        let input = r#"(Char "")"#;
        let result = eval_string(input, &mut context);
        assert!(result.is_err());

        let input = r#"(Char "abc")"#;
        let result = eval_string(input, &mut context);
        assert!(result.is_err());
    }
}
