// #todo find good module path: network/http?
// network/http
// network/http/ws
// network/smtp
// better than protocol/http, more specific.

// #ref https://tokio.rs/tokio/topics/bridging

pub fn http_get(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    todo!();
    // if let Some(expr) = args.first() {
    //     render_css_expr(expr)
    // } else {
    //     Err(Error::invalid_arguments(
    //         "expected at least one argument",
    //         None,
    //     ))
    // }
}

pub fn setup_lib_http(context: &mut Context) {
    // #todo find a good path, probably: net/http or network/http

    // let module = Module::new("http", context.top_scope.clone());

    // let scope = &module.scope;

    // // (let css (css-expr/to-css expr))
    // scope.insert("to-css", Expr::ForeignFunc(Arc::new(css_expr_to_css)));

    // // #todo another name than dialect? (language, lang, flavor, dsl)
    // // (use dialect/css-expr) (use dialect/css) (use dialect/html)
    // // #todo this is a hack.
    // let module_path = format!("{}/@std/dialect/css-expr", context.root_path);
    // // #todo introduce a helper for this.
    // context.module_registry.insert(module_path, Rc::new(module)); // #todo use Arc everywhere!
}
