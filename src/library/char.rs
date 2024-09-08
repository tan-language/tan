// #todo maybe as optimization, use special handling in eval?

// #todo create char from Num/Int also
// #todo function to get char int code
// #todo function to join chars into a string

use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{
        args::{unpack_char_arg, unpack_stringable_arg},
        module_util::require_module,
    },
};

pub fn char_new(args: &[Expr]) -> Result<Expr, Error> {
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

pub fn char_is_uppercase(args: &[Expr]) -> Result<Expr, Error> {
    let c = unpack_char_arg(args, 0, "char")?;
    Ok(Expr::Bool(c.is_uppercase()))
}

pub fn char_is_lowercase(args: &[Expr]) -> Result<Expr, Error> {
    let c = unpack_char_arg(args, 0, "char")?;
    Ok(Expr::Bool(c.is_lowercase()))
}

// #todo rename all setup_xxx functions to import_xxx.
pub fn setup_lib_char(context: &mut Context) {
    // #todo put in 'char' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    module.insert("Char", Expr::foreign_func(&char_new));

    module.insert("is-uppercase?", Expr::foreign_func(&char_is_uppercase));
    module.insert("is-lowercase?", Expr::foreign_func(&char_is_lowercase));
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
