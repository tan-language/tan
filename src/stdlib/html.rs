// #todo think a bit more about a good name
// #todo probably should move out from std lib into platform lib

use std::{rc::Rc, sync::Arc};

use crate::{context::Context, error::Error, expr::Expr, module::Module};

pub fn html_from_expr_expr(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    // if let Some(end) = args.first() {
    //     let Some(end) = end.as_int() else {
    //         return Err(Error::invalid_arguments(
    //             "expected Int argument",
    //             end.range(),
    //         ));
    //     };

    //     let mut rng = rand::thread_rng();

    //     Ok(Expr::Int(rng.gen_range(0..end)))
    // } else {
    //     Err(Error::invalid_arguments(
    //         "expected at least one argument",
    //         None,
    //     ))
    // }
    dbg!(&args);
    Ok(Expr::string("TODO"))
}

pub fn setup_std_html(context: &mut Context) {
    let module = Module::new("html", context.top_scope.clone());

    let scope = &module.scope;

    // (let html-string (html-from-expr expr))
    scope.insert(
        "html-from-expr",
        Expr::ForeignFunc(Arc::new(html_from_expr_expr)),
    );

    // #todo this is a hack.
    let module_path = format!("{}/std/html", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}
