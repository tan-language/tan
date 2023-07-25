use std::rc::Rc;

use crate::{
    context::Context,
    error::Error,
    eval::eval,
    expr::{annotate_range, Expr},
    scope::Scope,
    util::is_reserved_symbol,
};

// #Insight it mutates the env which is used in eval also!

// #todo `elision`, `elide` sounds better than `prune`?
// #todo rename to `prune_expand`?
// #todo split prune and expand into separate passes?
// #todo consider renaming the expr parameter to ast?
// #todo macro_expand (and all comptime/static passes should return Vec<Ranged<Error>>>)
// #todo support multiple errors, like in resolve.

// #todo return Vec<Error> like all other methods?

// #todo move pruning to optimize to run AFTER macro-expansion, macros could produce prunable exprs?
// #todo add macro-expansion tests!!!

// #insight
// If the input expr is just a macro definition, it can be elided!

// #todo maybe separate macro_def from macro_expand?

/// Expands macro invocations, at compile time.
pub fn macro_expand(expr: Expr, context: &mut Context) -> Result<Option<Expr>, Error> {
    match expr.unpack() {
        Expr::List(ref list) => {
            let head = list.first().unwrap(); // The unwrap here is safe.
            let tail = &list[1..];

            // Evaluate the head
            let Ok(op) = eval(head, context) else {
                // Don't err if we cannot eval the head.
                return Ok(Some(expr));
            };

            match op.unpack() {
                Expr::Macro(params, body) => {
                    // This is the actual macro-expansion

                    // #Insight
                    // Macro arguments are lazily evaluated.

                    let args = tail;

                    // #todo wtf is this ultra-hack?
                    // #todo ultra-hack to kill shared ref to `env`.
                    // let params = params.clone();
                    // let body = body.clone();

                    let prev_scope = context.scope.clone();
                    context.scope = Rc::new(Scope::new(prev_scope.clone()));

                    for (param, arg) in params.iter().zip(args) {
                        let Expr::Symbol(param) = param.unpack() else {
                            return Err(Error::invalid_arguments("parameter is not a symbol", param.range()));
                        };

                        context.scope.insert(param, arg.clone());
                    }
                    // #todo this code is the same as in the (do ..) block, extract.

                    // #todo do should be 'monadic', propagate Eff (effect) wrapper.
                    let mut value = Expr::One;

                    for expr in body {
                        value = eval(expr, context)?;
                    }

                    context.scope = prev_scope;

                    Ok(Some(value))
                }
                Expr::Symbol(sym) => {
                    // #todo oof the checks here happen also in resolver and eval, fix!
                    // #todo actually we should use `def` for this purpose, instead of `let`.
                    if sym == "let" {
                        let mut args = tail.iter();

                        // #todo should be def, no loop. <-- IMPORTANT!!
                        // #todo make more similar to the corresponding eval code.

                        let mut result_exprs = vec![Expr::Symbol("let".to_owned())];

                        loop {
                            let Some(binding_sym) = args.next() else {
                                break;
                                // return Err(Error::invalid_arguments("missing binding symbol", expr.range()));
                            };

                            let Some(binding_value) = args.next() else {
                                return Err(Error::invalid_arguments("missing binding value", expr.range()));
                            };

                            let Expr::Symbol(s) = binding_sym.unpack() else {
                                return Err(Error::invalid_arguments(&format!("`{sym}` is not a Symbol <<<<"), binding_sym.range()));
                            };

                            if is_reserved_symbol(s) {
                                return Err(Error::invalid_arguments(
                                    &format!("let cannot shadow the reserved symbol `{s}`"),
                                    binding_sym.range(),
                                ));
                            }

                            let binding_value = macro_expand(binding_value.clone(), context)?;

                            // #todo notify about overrides? use `set`?
                            // #todo consider if we should allow redefinitions.

                            let Some(binding_value) = binding_value else {
                                return Err(Error::invalid_arguments("Invalid arguments", None));
                            };

                            if let Expr::Macro(..) = binding_value.unpack() {
                                // #todo put all the definitions in one pass.
                                // Only define macros in this pass.
                                context.scope.insert(s, binding_value);

                                // #todo verify with unit-test.
                                // Macro definition is pruned.
                                // return Ok(None);
                            } else {
                                result_exprs.push(binding_sym.clone());
                                result_exprs.push(binding_value);
                            }
                        }

                        Ok(Some(Expr::List(result_exprs)))
                    } else if sym == "quot" {
                        let [value] = tail else {
                                return Err(Error::invalid_arguments("missing quote target", expr.range()));
                            };

                        // #todo super nasty, quotes should be resolved statically (at compile time)
                        // #todo hm, that clone, maybe `Rc` can fix this?
                        Ok(Some(
                            Expr::List(vec![
                                Expr::Symbol("quot".to_owned()).into(),
                                value.unpack().clone(),
                            ])
                            .into(),
                        ))
                    } else if sym == "Macro" {
                        // #todo this is duplicated in eval, think about this!!! probably should remove from eval.

                        let Some(params) = tail.first() else {
                            // #todo seems the range is not reported correctly here!!!
                            return Err(Error::invalid_arguments(
                                "malformed macro definition, missing function parameters",
                                expr.range(),
                            ));
                        };

                        let body = &tail[1..];

                        let Expr::List(params) = params.unpack() else {
                            return Err(Error::invalid_arguments("malformed macro parameters definition", params.range()));
                        };

                        // #todo optimize!
                        Ok(Some(Expr::Macro(params.clone(), body.into())))
                    } else {
                        // Other kind of list with symbol head, macro-expand tail.

                        let mut terms = Vec::new();
                        terms.push(op.clone());
                        for term in tail {
                            let term = macro_expand(term.clone(), context)?;
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
                        let term = macro_expand(term.clone(), context)?;
                        if let Some(term) = term {
                            terms.push(term);
                        }
                    }

                    Ok(Some(annotate_range(
                        Expr::List(terms),
                        // #todo hmmmm this unwrap!!!
                        expr.range().unwrap(),
                    )))
                }
            }
        }
        _ => Ok(Some(expr)),
    }
}
