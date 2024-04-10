// #todo text/css
// #todo rename to `cssx`? or `scss`? or `sexp-css`?
// #todo conside `css-expr` name: https://docs.racket-lang.org/css-expr/
// #todo consider naming this a 'dialect' or 'dsl' or 'language' instead of text?

use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::{format_value, Expr},
    util::{module_util::require_module, try_lock_read},
};

// #todo #fixme key-symbol conversion skips the `:` chars.
// #todo maybe move to expr.rs as an alternative `as_stringable`?
fn try_string_from_expr(expr: &Expr) -> Option<String> {
    let expr = expr.unpack();

    match expr {
        Expr::Symbol(s) => Some(s.clone()),
        Expr::KeySymbol(s) => Some(format!(":{s}")),
        Expr::String(s) => Some(s.clone()),
        _ => None,
    }
}

// #todo is this evaluating something?
// #todo consider always adding the trailing `;`?

fn render_css_expr(expr: &Expr) -> Result<Expr, Error> {
    let expr = expr.unpack();

    // #todo try to unquote!

    match expr {
        Expr::List(terms) => {
            if let Some(op) = terms.first() {
                let Some(sym) = try_string_from_expr(op) else {
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
                    let mut declaration = format_value(&prop);
                    i += 1;
                    if i < terms.len() {
                        declaration.push_str(": ");
                        let value = render_css_expr(&terms[i])?; // #todo no render needed
                        declaration.push_str(&format_value(&value));
                    }
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
        // #todo write a unit test for this.
        Expr::Array(rules) => {
            let mut body: Vec<String> = Vec::new();
            // #todo #hack ultra hackish way to emulate unquote-explode in CSS-Expr
            let rules = try_lock_read(rules, None)?;
            let is_explode = if let Some(flag) = rules[0].as_string() {
                flag == "..."
            } else {
                false
            };
            if is_explode {
                for expr in rules.iter().skip(1) {
                    body.push(format_value(expr));
                }
                Ok(Expr::string(body.join(";")))
            } else {
                for expr in rules.iter() {
                    let value = render_css_expr(expr)?;
                    body.push(format_value(value));
                }
                // #todo consider \n\n as separator.
                Ok(Expr::string(body.join("\n")))
            }
        }
        Expr::Map(..) => {
            // #todo remove duplication with List above.
            // #todo what is the coorect type for this?
            // let items: &HashMap<String, Expr> = items.borrow();
            // #todo #hack temp solution.
            let items = expr.as_map().unwrap();
            let mut body: Vec<String> = Vec::new();
            for (key, value) in items.iter() {
                body.push(format!("{key}: {}", format_value(value)));
            }
            Ok(Expr::string(body.join("; ")))
        }
        _ => Ok(Expr::string(format_value(expr))),
    }
}

// #todo name `css_from_css_expr` ?
pub fn css_expr_to_css(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
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
    // #todo another name than dialect? (language, lang, flavor, dsl)
    // (use dialect/css-expr) (use dialect/css) (use dialect/html)
    let module = require_module("dialect/css-expr", context);

    // (let css (css-expr/to-css expr))
    module.insert("to-css", Expr::ForeignFunc(Arc::new(css_expr_to_css)));
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

    #[test]
    fn to_expr_to_css_supports_nested_css() {
        let input = r#"
            (use "/dialect/css-expr")

            (let expr #CSS-Expr
                '(body
                    margin-top: "0"
                    margin-bottom: "10px"
                    background: "red"

                    (main
                        (div.profile
                            background-color: "yellow"
                        )
                    )
                )
            )

            (css-expr/to-css expr)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = expr.as_string().unwrap();
        let expected = r#"body { margin-top: 0; margin-bottom: 10px; background: red; main { div.profile { background-color: yellow } } }"#;
        assert_eq!(value, expected);
    }

    #[test]
    fn to_expr_to_css_supports_nested_pseudo_classes() {
        let input = r#"
            (use "/dialect/css-expr")

            (let expr #CSS-Expr
                '(a
                    :text-decoration "none"
                    (:hover
                        :text-decoration "underline"
                    )
                )
            )

            (css-expr/to-css expr)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = expr.as_string().unwrap();
        let expected = r#"a { text-decoration: none; :hover { text-decoration: underline } }"#;
        assert_eq!(value, expected);
    }

    // #todo give a good name to this test.
    #[test]
    fn to_expr_to_css_expands_maps() {
        let input = r#"
            (use "/dialect/css-expr")

            (let rules (Func []
                {
                    :font-size "12px"
                }
            ))

            (let expr #CSS-Expr
                '(a
                    $(rules)
                )
            )

            (css-expr/to-css expr)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = expr.as_string().unwrap();
        let expected = r#"a { font-size: 12px }"#;
        assert_eq!(value, expected);
    }
}
