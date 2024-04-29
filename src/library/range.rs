use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

fn make_range<T>(start: T, end: T, step: T) -> Expr {
    // let expr = Expr::array(vec![re.into(), im.into()]);
    // #todo use IntRange, FloatRange.
    // annotate_type(expr, "Range")
    todo!();
}

pub fn range_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // let set: HashSet<Expr> = HashSet::new();
    // Ok(Expr::set(set))
    // #todo implement me!
    Ok(Expr::Nil)
}

pub fn setup_lib_range(context: &mut Context) {
    // #todo put in 'range' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    module.insert("Range", Expr::ForeignFunc(Arc::new(range_new)));
}
