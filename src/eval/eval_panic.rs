use crate::{context::Context, error::Error, expr::Expr};

use super::util::get_current_file_path;

// #todo make this anchor-compatible.
// #todo could be made a ForeignFunc actually, not performance sensitive.
// #todo extract to special_forms or something.
// #todo note that we pass op, this is a macro?
pub fn eval_panic(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo make message optional!

    // #todo the op.range() annotation could be applied externally.
    let [msg] = args else {
        return Err(Error::invalid_arguments("requires `msg` argument", None));
    };

    let Some(msg) = msg.as_stringable() else {
        return Err(Error::invalid_arguments(
            "`msg` argument should be a Stringable",
            msg.range(),
        ));
    };

    // #todo encode location.

    // #todo add panic constructor.
    let mut error = Error {
        variant: crate::error::ErrorVariant::Panic(msg.to_string()),
        file_path: get_current_file_path(context),
        notes: vec![],
    };

    error.push_note(msg, None);

    Err(error)
}
