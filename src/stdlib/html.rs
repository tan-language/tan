// #todo think a bit more about a good name
// #todo probably should move out from std lib into platform lib
// #todo also perform attribute and body escaping
// #todo add unit tests
// #todo what about attributes without value? for the moment just use true, e.g. { :attr true }, can optimize to no-value attr on rendering

// #todo translate (ul.nasty ..) to (ul {class: "nasty"})
// #todo translate (ul#nasty ..) to (ul {id: "nasty"})

use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    eval::util::eval_module,
    expr::{format_value, Expr},
    util::module_util::require_module,
};

// #todo investigate the interaction between expr/string interpolation '$' and quoting, make string interpolation work in quoted expr.

// example:
// ```tan
// (body
//     "Hello " $name "! Num: " (b "cool " $(+ 1 2))
//     (br)(br)
//     $(component '(i "Zonk"))
// )
// ```

// #todo how to interpolate values?
// #todo consider using 'interned'/self-evaluating symbols instead of strings for text nodes.
// #todo special handling of child strings with interpolation.
// #todo support pretty-printing

// #todo should take a list of expressions to handle '(!DOCTYPE "html")
// #todo handle self-closing tags, e.g. <meta ... />, <link ... />
// #todo should act on string, and only wrap in Expr::string(...) at the end.
fn render_expr(expr: &Expr) -> Result<Expr, Error> {
    let expr = expr.unpack();

    match expr {
        Expr::Array(terms) => {
            // #todo this is a temp solution, investigate refactoring opportunities
            let mut html = String::new();
            let terms = terms.borrow();
            for term in terms.iter() {
                let expr = render_expr(term)?;
                html.push_str(&format_value(&expr));
            }
            Ok(Expr::string(html))
        }
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

                    // #todo escape body/children
                    let mut body = String::from("");

                    while i < terms.len() {
                        // #todo if term is an array, spread it!
                        let term = &terms[i];

                        match term.unpack() {
                            Expr::Array(array) => {
                                let array = array.borrow();
                                for t in array.iter() {
                                    let child = render_expr(t)?;
                                    body.push_str(&format_value(&child));
                                }
                            }
                            _ => {
                                let child = render_expr(&terms[i])?;
                                body.push_str(&format_value(&child));
                            }
                        }
                        // #todo handle () / empty / Never (for conditionals)
                        // #insight spread will work nicely with for->list
                        // let child = render_expr(&terms[i])?;
                        // body.push_str(&format_value(&child));
                        i += 1;
                    }

                    // #todo eval body.

                    // #insight always close <script> tags.

                    if body.is_empty() && sym != "script" {
                        Ok(Expr::string(format!("<{sym}{attributes} />")))
                    } else {
                        Ok(Expr::string(format!("<{sym}{attributes}>{body}</{sym}>")))
                    }
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
        // #todo is there a better way to do this?
        // #insight Nothing/Zero expressions should be skipped.
        Expr::Zero => Ok(Expr::string("")),
        _ => Ok(Expr::string(format_value(expr))),
    }
}

// #todo find a better name.
pub fn html_from_expr(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    if let Some(expr) = args.first() {
        render_expr(expr)
    } else {
        Err(Error::invalid_arguments(
            "expected at least one argument",
            None,
        ))
    }
}

pub fn setup_lib_html(context: &mut Context) {
    let module = require_module("html", context);

    module.insert(
        "html-from-expr",
        Expr::ForeignFunc(Arc::new(html_from_expr)),
    );

    // #insight
    // This is currently an experiment to add additional methods implemented
    // in tan.

    // #todo handle error
    let _ = eval_module("html", context, true);
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context};

    // #todo add a test with arrays and conditionals.

    #[test]
    fn html_from_expr_usage() {
        // #todo extract as fixture.
        // #insight we intentionally use html tags with a single attribute, in this test, as ordering is currently not preserved.
        let input = r#"
            (use "html")

            (let name "George")

            (let component (Func [name]
                '(div "Component: " $name " is cool!")
            ))

            (let show-header true)
            (let show-footer false)

            (let expr #HTML
                '(html {lang: "el"}
                    (head
                        (title "Hello")
                        (link {href: "https://www.example.com/icon.png"})
                    )
                    (body
                        $(if show-header '(header "A nice header"))
                        "Hello " $name "! Num: " (b "cool " $(+ 1 2))
                        (br)(br)
                        $(component '(i "Stella"))
                        $(if show-footer
                            '(footer "A nice footer")
                        )
                    )
                )
            )

            (html/html-from-expr expr)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = expr.as_string().unwrap();
        let expected = r#"<html lang="el"><head><title>Hello</title><link href="https://www.example.com/icon.png" /></head><body><header>A nice header</header>Hello George! Num: <b>cool 3</b><br /><br /><div>Component: <i>Stella</i> is cool!</div></body></html>"#;
        assert_eq!(value, expected);
    }

    #[test]
    fn should_close_script_tags() {
        let input =
            r#"(use "html")(html/html-from-expr '(script {:src "https://example.com/script.js"}))"#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = expr.as_string().unwrap();
        let expected = r#"<script src="https://example.com/script.js"></script>"#;
        assert_eq!(value, expected);
    }

    #[test]
    fn should_handle_array_of_exprs() {
        let input = r#"
            (use "html")
            (let cool (Func []
                '(b "Cool")
            ))
            (let helper (Func []
                [
                    (cool)
                    '(i "Stuff")
                ]
            ))
            (html/html-from-expr '(div $(helper) (b "Works")))
            ; (html/html-from-expr '($(helper) (b "Works")))
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = expr.as_string().unwrap();
        let expected = r#"<div><b>Cool</b><i>Stuff</i><b>Works</b></div>"#;
        assert_eq!(value, expected);
    }
}
