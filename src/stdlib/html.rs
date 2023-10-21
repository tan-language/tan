// #todo think a bit more about a good name
// #todo probably should move out from std lib into platform lib
// #todo also perform attribute and body escaping
//

use std::{rc::Rc, sync::Arc};

use crate::{
    context::Context,
    error::Error,
    expr::{format_value, Expr},
    module::Module,
};

// #todo consider using 'interned'/self-evaluating symbols instead of strings for text nodes.

fn render_expr(expr: &Expr) -> Result<Expr, Error> {
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

                if let Some(term) = terms.get(i) {
                    let attributes: String = if let Some(dict) = term.as_dict() {
                        i += 1;
                        // #todo eval value!
                        // #todo escape value!
                        format!(
                            " {}",
                            dict.iter()
                                .map(|(k, v)| format!("{k}=\"{}\"", format_value(v)))
                                .collect::<Vec<String>>()
                                .join(" ")
                        )
                    } else {
                        "".to_string()
                    };

                    // #todo escape body/
                    let mut body = String::from("");

                    while i < terms.len() {
                        let child = render_expr(&terms[i])?;
                        body.push_str(&format_value(&child));
                        i += 1;
                    }

                    // #todo eval body.
                    Ok(Expr::string(format!("<{sym}{attributes}>{body}</{sym}>")))
                } else {
                    Ok(Expr::string(format!("<{sym} />")))
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

// #todo find a better name.
pub fn html_from_expr_expr(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    if let Some(expr) = args.first() {
        render_expr(expr)
    } else {
        Err(Error::invalid_arguments(
            "expected at least one argument",
            None,
        ))
    }
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
