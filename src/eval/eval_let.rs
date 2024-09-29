use crate::{context::Context, error::Error, expr::Expr};

use super::{eval, insert_binding};

// #todo correctly implement the let-rules
// #todo correctly implement the let shadowing rules

// #todo pass-through the annotations.

pub fn eval_let(op: &Expr, args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo there is currently no resolver, duh.
    // #todo actually some resolving is happening in macro_expand, e.g. checking for binding values.
    // #todo this is already parsed statically by resolver, no need to duplicate the tests here?
    // #todo also report some of these errors statically, maybe in a sema phase?
    // #todo use 'location' or 'lvalue' instead of name?

    // #insight 'pass-through' let annotations, only for ...def.

    let mut args = args.iter();

    loop {
        // #todo report `(let)` as error.
        let Some(name) = args.next() else {
            break;
        };

        let Some(value) = args.next() else {
            // #todo error?
            break;
        };

        // #todo Should maybe implement methods here?
        // #todo If it's an invocable convert to a method? Expr::Method.

        let value = Expr::maybe_annotated(eval(value, context)?, op.annotations());

        insert_binding(name, value, context)?
    }

    // #todo return last value, it would require some cloning currently.
    Ok(Expr::None)
}
