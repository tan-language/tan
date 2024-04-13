use crate::{context::Context, error::Error, expr::Expr};

use super::{eval, insert_binding};

pub fn eval_let(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo there is currently no resolver, duh.
    // #todo actually some resolving is happening in macro_expand, e.g. checking for binding values.
    // #todo this is already parsed statically by resolver, no need to duplicate the tests here?
    // #todo also report some of these errors statically, maybe in a sema phase?
    // #todo use 'location' or 'lvalue' instead of name?

    let mut args = args.iter();

    loop {
        let Some(name) = args.next() else {
            break;
        };

        let Some(value) = args.next() else {
            // #todo error?
            break;
        };

        let value = eval(value, context)?;

        insert_binding(name, value, context)?
    }

    // #todo return last value, it would require some cloning currently.
    Ok(Expr::Nil)
}
