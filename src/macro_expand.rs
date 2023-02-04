use crate::{
    ann::Ann, error::Error, eval::env::Env, expr::Expr, range::Ranged, util::is_reserved_symbol,
};

// #Insight it mutates the env which is used in eval also!

// #TODO remove the macro definitions from the AST
// #TODO consider renaming the expr parameter to ast?

/// Expands macro invocations, at comptime.
pub fn macro_expand(expr: Ann<Expr>, env: &mut Env) -> Result<Ann<Expr>, Ranged<Error>> {
    match expr {
        Ann(Expr::List(ref list), ..) => {
            if list.is_empty() {
                // This is handled statically, in the parser, but an extra, dynamic
                // check is needed in the evaluator to handle the case where the
                // expression is constructed programmatically (e.g. self-modifying code,
                // dynamically constructed expression, homoiconicity, etc).
                return Ok(expr);
            }

            let head = list.first().unwrap(); // The unwrap here is safe.
            let tail = &list[1..];

            match head.as_ref() {
                Expr::Symbol(sym) => {
                    // #TODO actually we should use `def` for this purpose, instead of `let`.
                    if sym == "let" {
                        let mut args = tail.iter();

                        loop {
                            let Some(binding_sym) = args.next() else {
                                break;
                            };

                            let Some(binding_value) = args.next() else {
                                // #TODO error?
                                break;
                            };

                            let Ann(Expr::Symbol(s), ..) = binding_sym else {
                                return Err(Error::invalid_arguments(format!("`{sym}` is not a Symbol")).into());
                            };

                            if is_reserved_symbol(s) {
                                return Err(Error::invalid_arguments(format!(
                                    "let cannot shadow the reserved symbol `{s}`"
                                ))
                                .into());
                            }

                            let binding_value = macro_expand(binding_value.clone(), env)?;

                            // #TODO notify about overrides? use `set`?
                            // #TODO consider if we should allow redefinitions.

                            if let Ann(Expr::Macro(..), ..) = binding_value {
                                // #TODO put all the definitions in one pass.
                                // Only define macros in this pass.
                                env.insert(s, binding_value);
                            }
                        }

                        // #TODO return last value!
                        Ok(Expr::One.into())
                    } else if sym == "Macro" {
                        let [args, body] = tail else {
                            return Err(Error::invalid_arguments("malformed macro definition").into());
                        };

                        let Ann(Expr::List(params), ..) = args else {
                            return Err(Error::invalid_arguments("malformed macro parameters definition").into());
                        };

                        // #TODO optimize!
                        Ok(Expr::Macro(params.clone(), Box::new(body.clone())).into())
                    } else {
                        Ok(expr)
                    }
                }
                _ => Ok(expr),
            }
        }
        _ => Ok(expr),
    }
}
