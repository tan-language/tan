use std::{rc::Rc, sync::Arc};

use comrak::{markdown_to_html, Options};

use crate::{context::Context, error::Error, expr::Expr, module::Module};

// #todo find a better name for this module.
// #todo this should be extracted to a separate crate, and/or a dynamic library.

pub fn html_from_common_mark(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    if let Some(expr) = args.first() {
        let Some(markup) = expr.as_string() else {
            return Err(Error::invalid_arguments(
                "`markup` should be a String",
                None,
            ));
        };

        let html = markdown_to_html(markup, &Options::default());

        Ok(Expr::String(html))
    } else {
        Err(Error::invalid_arguments(
            "expected at least one argument",
            None,
        ))
    }
}

pub fn setup_text_cmark(context: &mut Context) {
    let module = Module::new("cmark", context.top_scope.clone());

    let scope = &module.scope;

    // (let html htmlx/to-html expr)
    // (let html htmlx/expr-to-html expr)
    // (let html htmlx/expr->html expr)
    // (let html cmark/to-html markup)
    scope.insert(
        "to-html",
        Expr::ForeignFunc(Arc::new(html_from_common_mark)),
    );

    // #todo find a good name/path for this library.
    // #todo this is a hack.
    let module_path = format!("{}/@std/text/cmark", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context};

    #[test]
    fn html_from_common_mark_usage() {
        // #todo extract as fixture.
        let input = r###"
            (use "/text/cmark")

            ; #todo what could be a good 'category'/'tag' annotation?
            (let markup #Common-Mark "# Title\nThis is a _nice_ text.")

            (cmark/to-html markup)
        "###;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = expr.as_string().unwrap();
        let expected = "<h1>Title</h1>\n<p>This is a <em>nice</em> text.</p>\n";
        assert_eq!(value, expected);
    }
}
