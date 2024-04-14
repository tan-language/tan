use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, util::standard_names::CURRENT_FILE_PATH};

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

    let file_path = context
        .get_special(CURRENT_FILE_PATH)
        // #todo this duplicated code from eval, refactor+extract
        // #todo think about how to best handle this.
        // #insight use unwrap_or_else to be more fault tolerant, when no file is available (eval_string, repl, etc...)
        .unwrap_or_else(|| Arc::new(Expr::string("UNKNOWN")))
        .as_string()
        .unwrap()
        .to_string();

    // #todo add panic constructor.
    let mut error = Error {
        variant: crate::error::ErrorVariant::Panic(msg.to_string()),
        file_path: file_path.clone(),
        notes: vec![],
    };

    error.push_note(msg, None);

    Err(error)
}
