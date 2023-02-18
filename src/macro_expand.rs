use crate::{
    ann::Ann,
    error::Error,
    eval::{env::Env, eval},
    expr::Expr,
    range::Ranged,
    util::is_reserved_symbol,
};

// #Insight it mutates the env which is used in eval also!

// #TODO `elision`, `elide` sounds better than `prune`?
// #TODO rename to `prune_expand`?
// #TODO split prune and expand into separate passes?
// #TODO consider renaming the expr parameter to ast?
// #TODO macro_expand (and all comptime/static passes should return Vec<Ranged<Error>>>)
// #TODO support multiple errors, like in resolve.

/// Expands macro invocations, at compile time.
pub fn macro_expand(expr: Ann<Expr>, env: &mut Env) -> Result<Option<Ann<Expr>>, Ranged<Error>> {
    match expr {
        Ann(Expr::Comment(..), ..) => {
            // Prune Comment expressions.
            Ok(None)
        }
        Ann(Expr::Annotation(..), ..) => {
            // Prune Annotation expressions.
            Ok(None)
        }
        Ann(Expr::List(ref list), ..) => {
            // if list.is_empty() {
            //     // This is handled statically, in the parser, but an extra, dynamic
            //     // check is needed in the evaluator to handle the case where the
            //     // expression is constructed programmatically (e.g. self-modifying code,
            //     // dynamically constructed expression, homoiconicity, etc).
            //     return Ok(None);
            // }

            let head = list.first().unwrap(); // The unwrap here is safe.
            let tail = &list[1..];

            // Evaluate the head
            let Ok(head) = eval(head, env) else {
                // Don't err if we cannot eval the head.
                return Ok(Some(expr));
            };

            // #TODO can we remove this as_ref?
            match head.as_ref() {
                Expr::Macro(params, body) => {
                    // This is the actual macro-expansion

                    // #Insight
                    // Macro arguments are lazily evaluated.

                    let args = tail;

                    // #TODO ultra-hack to kill shared ref to `env`.
                    let params = params.clone();
                    let body = body.clone();

                    // #TODO what kind of scoping is this?

                    env.push_new_scope();

                    for (param, arg) in params.iter().zip(args) {
                        let Ann(Expr::Symbol(param), ..) = param else {
                                return Err(Ranged(Error::invalid_arguments("parameter is not a symbol"), param.get_range()));
                            };

                        env.insert(param, arg.clone());
                    }

                    let result = eval(&body, env)?;

                    env.pop();

                    Ok(Some(result))
                }
                Expr::Symbol(sym) => {
                    // #TODO oof the checks here happen also in resolver and eval, fix!
                    // #TODO actually we should use `def` for this purpose, instead of `let`.
                    if sym == "let" {
                        let mut args = tail.iter();

                        // #TODO should be def, no loop.

                        let Some(binding_sym) = args.next() else {
                            return Err(Ranged(Error::invalid_arguments("missing binding symbol"), expr.get_range()));
                        };

                        let Some(binding_value) = args.next() else {
                            return Err(Ranged(Error::invalid_arguments("missing binding value"), expr.get_range()));
                        };

                        let Ann(Expr::Symbol(s), ..) = binding_sym else {
                            return Err(Ranged(Error::invalid_arguments(format!("`{sym}` is not a Symbol")), binding_sym.get_range()));
                        };

                        if is_reserved_symbol(s) {
                            return Err(Ranged(
                                Error::invalid_arguments(format!(
                                    "let cannot shadow the reserved symbol `{s}`"
                                )),
                                binding_sym.get_range(),
                            ));
                        }

                        let binding_value = macro_expand(binding_value.clone(), env)?;

                        // #TODO notify about overrides? use `set`?
                        // #TODO consider if we should allow redefinitions.

                        if let Some(Ann(Expr::Macro(..), ..)) = binding_value {
                            // #TODO put all the definitions in one pass.
                            // Only define macros in this pass.
                            env.insert(s, binding_value.unwrap());

                            // #TODO verify with unit-test.
                            // Macro definition is pruned.
                            return Ok(None);
                        }

                        Ok(Some(
                            Expr::List(vec![
                                Expr::Symbol("let".to_owned()).into(),
                                binding_sym.clone(),
                                binding_value.unwrap(), // #TODO argh, remove the unwrap!
                            ])
                            .into(),
                        ))
                    } else if sym == "quot" {
                        let [value] = tail else {
                                return Err(Ranged(Error::invalid_arguments("missing quote target"), expr.get_range()));
                            };

                        // #TODO super nasty, quotes should be resolved statically (at compile time)
                        // #TODO hm, that clone, maybe `Rc` can fix this?
                        Ok(Some(
                            Expr::List(vec![
                                Expr::Symbol("quot".to_owned()).into(),
                                value.0.clone().into(),
                            ])
                            .into(),
                        ))
                    } else if sym == "Macro" {
                        let [args, body] = tail else {
                            return Err(Ranged(Error::invalid_arguments("malformed macro definition"), expr.get_range()));
                        };

                        let Ann(Expr::List(params), ..) = args else {
                            return Err(Ranged(Error::invalid_arguments("malformed macro parameters definition"), expr.get_range()));
                        };

                        // #TODO optimize!
                        Ok(Some(
                            Expr::Macro(params.clone(), Box::new(body.clone())).into(),
                        ))
                    } else {
                        // Other kind of list with symbol head, macro-expand tail.

                        let mut terms = Vec::new();
                        terms.push(head.clone());
                        for term in tail {
                            let term = macro_expand(term.clone(), env)?;
                            if let Some(term) = term {
                                terms.push(term);
                            }
                        }

                        Ok(Some(Expr::List(terms).into()))
                    }
                }
                _ => {
                    // Other kind of list with non-symbol head, macro-expand tail.
                    let mut terms = Vec::new();
                    terms.push(head.clone());
                    for term in tail {
                        let term = macro_expand(term.clone(), env)?;
                        if let Some(term) = term {
                            terms.push(term);
                        }
                    }

                    Ok(Some(Expr::List(terms).into()))
                }
            }
        }
        _ => Ok(Some(expr)),
    }
}
