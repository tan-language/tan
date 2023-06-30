use crate::{
    error::Error,
    eval::{env::Env, eval},
    expr::{annotate_range, Expr},
    util::is_reserved_symbol,
};

// #Insight it mutates the env which is used in eval also!

// #TODO `elision`, `elide` sounds better than `prune`?
// #TODO rename to `prune_expand`?
// #TODO split prune and expand into separate passes?
// #TODO consider renaming the expr parameter to ast?
// #TODO macro_expand (and all comptime/static passes should return Vec<Ranged<Error>>>)
// #TODO support multiple errors, like in resolve.

// #TODO return Vec<Error> like all other methods?

// #TODO move pruning to optimize to run AFTER macro-expansion, macros could produce prunable exprs?
// #TODO add macro-expansion tests!!!

// #TODO
/// Expands macro invocations, at compile time.
pub fn macro_expand(expr: Expr, env: &mut Env) -> Result<Option<Expr>, Error> {
    match expr.unpack() {
        Expr::Comment(..) => {
            // #TODO move prune elsewhere.
            // Prune Comment expressions.
            Ok(None)
        }
        Expr::TextSeparator => {
            // #TODO remove TextSeparator anws.
            // #TODO move prune elsewhere.
            // Prune TextSeparator expressions.
            Ok(None)
        }
        Expr::List(ref list) => {
            let head = list.first().unwrap(); // The unwrap here is safe.
            let tail = &list[1..];

            // Evaluate the head
            let Ok(head) = eval(head, env) else {
                // Don't err if we cannot eval the head.
                return Ok(Some(expr));
            };

            match head.unpack() {
                Expr::Macro(params, body) => {
                    // This is the actual macro-expansion

                    // #Insight
                    // Macro arguments are lazily evaluated.

                    let args = tail;

                    // #TODO wtf is this ultra-hack?
                    // #TODO ultra-hack to kill shared ref to `env`.
                    // let params = params.clone();
                    // let body = body.clone();

                    // #TODO what kind of scoping is this?

                    env.push_new_scope();

                    for (param, arg) in params.iter().zip(args) {
                        let Expr::Symbol(param) = param.unpack() else {
                            return Err(Error::invalid_arguments("parameter is not a symbol", param.range()));
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
                            return Err(Error::invalid_arguments("missing binding symbol", expr.range()));
                        };

                        let Some(binding_value) = args.next() else {
                            return Err(Error::invalid_arguments("missing binding value", expr.range()));
                        };

                        let Expr::Symbol(s) = binding_sym.unpack() else {
                            return Err(Error::invalid_arguments(&format!("`{sym}` is not a Symbol"), binding_sym.range()));
                        };

                        if is_reserved_symbol(s) {
                            return Err(Error::invalid_arguments(
                                &format!("let cannot shadow the reserved symbol `{s}`"),
                                binding_sym.range(),
                            ));
                        }

                        let binding_value = macro_expand(binding_value.clone(), env)?;

                        // #TODO notify about overrides? use `set`?
                        // #TODO consider if we should allow redefinitions.

                        let Some(binding_value) = binding_value else {
                            return Err(Error::invalid_arguments("Invalid arguments", None));
                        };

                        if let Expr::Macro(..) = binding_value.unpack() {
                            // #TODO put all the definitions in one pass.
                            // Only define macros in this pass.
                            env.insert(s, binding_value);

                            // #TODO verify with unit-test.
                            // Macro definition is pruned.
                            return Ok(None);
                        }

                        Ok(Some(
                            Expr::List(vec![
                                Expr::Symbol("let".to_owned()).into(),
                                binding_sym.clone(),
                                binding_value,
                            ])
                            .into(),
                        ))
                    } else if sym == "quot" {
                        let [value] = tail else {
                                return Err(Error::invalid_arguments("missing quote target", expr.range()));
                            };

                        // #TODO super nasty, quotes should be resolved statically (at compile time)
                        // #TODO hm, that clone, maybe `Rc` can fix this?
                        Ok(Some(
                            Expr::List(vec![
                                Expr::Symbol("quot".to_owned()).into(),
                                value.unpack().clone(),
                            ])
                            .into(),
                        ))
                    } else if sym == "Macro" {
                        let [args, body] = tail else {
                            return Err(Error::invalid_arguments("malformed macro definition", expr.range()));
                        };

                        let Expr::List(params) = args.unpack() else {
                            return Err(Error::invalid_arguments("malformed macro parameters definition", expr.range()));
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

                    Ok(Some(annotate_range(
                        Expr::List(terms),
                        // #TODO hmmmm this unwrap!!!
                        expr.range().unwrap(),
                    )))
                }
            }
        }
        _ => Ok(Some(expr)),
    }
}
