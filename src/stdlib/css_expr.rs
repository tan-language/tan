// #todo text/css
// #todo rename to `cssx`? or `scss`? or `sexp-css`?
// #todo conside `css-expr` name: https://docs.racket-lang.org/css-expr/
// #todo consider naming this a 'dialect' or 'dsl' or 'language' instead of text?

use std::{rc::Rc, sync::Arc};

use crate::{
    context::Context,
    error::Error,
    expr::{format_value, Expr},
    module::Module,
};

// #todo is this evaluating something?
// #todo consider always adding the trailing `;`?

fn render_css_expr(expr: &Expr) -> Result<Expr, Error> {
    let expr = expr.unpack();

    match expr {
        Expr::List(terms) => {
            if let Some(op) = terms.first() {
                let Some(sym) = op.as_symbol() else {
                    // #todo we could return the argument position here and enrich the error upstream.
                    // #todo hmm, the error is too precise here, do we really need the annotations?
                    return Err(Error::invalid_arguments(
                        &format!("{op} is not a Symbol"),
                        op.range(),
                    ));
                };

                let mut i = 1;

                // #todo escape body/children

                let mut body: Vec<String> = Vec::new();

                while i < terms.len() {
                    let prop = render_css_expr(&terms[i])?; // #todo no render needed
                    let mut declaration = format!("{}: ", format_value(&prop));
                    i += 1;
                    let value = render_css_expr(&terms[i])?; // #todo no render needed
                    declaration.push_str(&format_value(&value));
                    i += 1;
                    body.push(declaration);
                }

                if body.is_empty() {
                    // #todo add exception for <script> tag.
                    Ok(Expr::string(format!("{sym} {{}}")))
                } else {
                    Ok(Expr::string(format!("{sym} {{ {} }}", body.join("; "))))
                }
            } else {
                // #todo offer context, e.g. in which function we are.
                Err(Error::invalid_arguments(
                    "empty expression, remove",
                    expr.range(),
                )) // #todo
            }
        }
        _ => Ok(Expr::string(format_value(expr))),
    }
}

// #todo name `css_from_css_expr` ?
pub fn css_expr_to_css(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    if let Some(expr) = args.first() {
        render_css_expr(expr)
    } else {
        Err(Error::invalid_arguments(
            "expected at least one argument",
            None,
        ))
    }
}

// #todo consider naming the library just `css`?
// #todo consider setup_mod_css or setup_module_css
pub fn setup_lib_css_expr(context: &mut Context) {
    let module = Module::new("css-expr", context.top_scope.clone());

    let scope = &module.scope;

    // (let css (css-expr/to-css expr))
    scope.insert("to-css", Expr::ForeignFunc(Arc::new(css_expr_to_css)));

    // #todo another name than dialect? (language, lang, flavor, dsl)
    // (use dialect/css-expr) (use dialect/css) (use dialect/html)
    // #todo this is a hack.
    let module_path = format!("{}/dialect/css-expr", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module)); // #todo use Arc everywhere!
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context};

    #[test]
    fn to_expr_to_css_usage() {
        // #todo extract as fixture.
        // #todo use #{dialect: :CSS} annotation
        // #insight no need for [] wrapper, in the actual file reader, return a list of expression, and run to-css on all expressions.
        // #insight we intentionally use html tags with a single attribute, in this test, as ordering is currently not preserved.
        let input = r#"
            (use "/dialect/css-expr")

            (let expr #CSS-Expr
                '(body
                    margin-top: "0"
                    margin-bottom: "10px"
                    background: "red"
                )
            )

            (css-expr/to-css expr)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = expr.as_string().unwrap();
        let expected = r#"body { margin-top: 0; margin-bottom: 10px; background: red }"#;
        assert_eq!(value, expected);
    }
}
