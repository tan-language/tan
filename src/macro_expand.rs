use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    eval::eval,
    expr::{annotate_range, expr_clone, Expr},
    scope::Scope,
    util::{args::unpack_arg, is_reserved_symbol},
};

// #insight it mutates the env which is used in eval also!

// #todo `elision`, `elide` sounds better than `prune`?
// #todo rename to `prune_expand`?
// #todo split prune and expand into separate passes?
// #todo consider renaming the expr parameter to ast?
// #todo macro_expand (and all comptime/static passes should return Vec<Ranged<Error>>>)
// #todo support multiple errors, like in resolve.

// #todo should we care about dynamic-scoping here?

// #todo return Vec<Error> like all other methods?

// #todo move pruning to optimize to run AFTER macro-expansion, macros could produce prunable exprs?
// #todo add macro-expansion tests!!!

// #insight
// If the input expr is just a macro definition, it can be elided!

// #todo maybe separate macro_def from macro_expand?

// #todo #think this prematurely strips list annotations.

/// Expands macro invocations, at compile time.
pub fn macro_expand(expr: Expr, context: &mut Context) -> Result<Option<Expr>, Error> {
    match expr.unpack() {
        Expr::List(ref list) => {
            let head = list.first().unwrap(); // The unwrap here is safe.
            let tail = &list[1..];

            // Evaluate the head
            let Ok(op) = eval(head, context) else {
                // #todo what exactly is happening here?
                // Don't err if we cannot eval the head.
                return Ok(Some(expr));
            };

            match op.unpack() {
                Expr::Macro(params, body) => {
                    // This is the actual macro-expansion

                    // #insight
                    // Macro arguments are lazily evaluated.

                    let args = tail;

                    // #todo wtf is this ultra-hack?
                    // #todo ultra-hack to kill shared ref to `env`.
                    // let params = params.clone();
                    // let body = body.clone();

                    let prev_scope = context.scope.clone();
                    context.scope = Arc::new(Scope::new(prev_scope.clone()));

                    for (param, arg) in params.iter().zip(args) {
                        let Expr::Symbol(param) = param.unpack() else {
                            return Err(Error::invalid_arguments(
                                "parameter is not a symbol",
                                param.range(),
                            ));
                        };

                        context.scope.insert(param, arg.clone());
                    }
                    // #todo this code is the same as in the (do ..) block, extract.

                    // #todo do should be 'monadic', propagate Eff (effect) wrapper.
                    let mut value = Expr::Nil;

                    for expr in body {
                        value = eval(expr, context)?;
                    }

                    context.scope = prev_scope;

                    Ok(Some(value))
                }
                Expr::Type(sym) => {
                    // #insight macro handling is removed from eval, there are no runtime/dynamic macro definitions
                    // #insight to create/eval macros at runtime (dyn-time) make sure you call macro-expand.
                    // #todo macros should be handled at a separate, comptime, macroexpand pass.
                    // #todo actually two passes, macro_def, macro_expand
                    // #todo probably macro handling should be removed from eval, there are no runtime/dynamic macro definitions!!
                    if sym == "Macro" {
                        let Some(params) = tail.first() else {
                            // #todo seems the range is not reported correctly here!!!
                            return Err(Error::invalid_arguments(
                                "malformed macro definition, missing function parameters",
                                expr.range(),
                            ));
                        };

                        let body = &tail[1..];

                        let Expr::List(params) = params.unpack() else {
                            return Err(Error::invalid_arguments(
                                "malformed macro parameters definition",
                                params.range(),
                            ));
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

                        Ok(Some(Expr::List(terms)))
                    }
                }
                Expr::Symbol(sym) => {
                    // #insight let-ds is not relevant for static-time macro expansion.
                    // #todo oof the checks here happen also in resolver and eval, fix!
                    // #todo actually we should use `def` for this purpose, instead of `let`.
                    if sym == "let" {
                        let mut args = tail.iter();

                        // #todo should be def, no loop. <-- IMPORTANT!!
                        // #todo make more similar to the corresponding eval code.

                        // #insight this was stripping the annotations from let!
                        // let mut result_exprs = vec![Expr::Symbol("let".to_owned())];
                        let mut result_exprs = vec![op.clone()];

                        loop {
                            let Some(binding_sym) = args.next() else {
                                break;
                                // return Err(Error::invalid_arguments("missing binding symbol", expr.range()));
                            };

                            let Some(binding_value) = args.next() else {
                                return Err(Error::invalid_arguments(
                                    "missing binding value",
                                    expr.range(),
                                ));
                            };

                            let Expr::Symbol(s) = binding_sym.unpack() else {
                                // #todo #ultra-hack
                                // #todo proper destructuring needed here!
                                result_exprs.push(binding_sym.clone());
                                result_exprs.push(binding_value.clone());
                                continue;
                                // return Err(Error::invalid_arguments(
                                //     &format!("`{sym}` is not a Symbol"),
                                //     binding_sym.range(),
                                // ));
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

                        // #todo #WARNING annotations are stripped here!
                        Ok(Some(Expr::maybe_annotated(
                            Expr::List(result_exprs),
                            expr.annotations(),
                        )))
                    } else if sym == "quot" {
                        let [value] = tail else {
                            return Err(Error::invalid_arguments(
                                "missing quote target",
                                expr.range(),
                            ));
                        };

                        // #todo super nasty, quotes should be resolved statically (at compile time)
                        // #todo hm, that clone, maybe `Arc` can fix this?
                        Ok(Some(Expr::List(vec![
                            Expr::Symbol("quot".to_owned()),
                            value.unpack().clone(),
                        ])))
                    } else if sym == "+<-" {
                        // Expand some assignment operators
                        // #todo consider `assign-add` or `+assign` instead.
                        // #todo assign+, assign-, assign*, assign/
                        // #todo extract as helper for all assignment operators.
                        // #todo think about this.
                        // #todo maybe use "+=" as the operator?
                        // #todo what is a better name than `accum`? `target`?
                        let accum = unpack_arg(tail, 0, "accum")?;
                        let value = unpack_arg(tail, 1, "value")?;

                        // (+<- accum value) -> (<- accum (+ accum value))

                        // #todo make sure we clone the correct ranges of the symbols.

                        // #todo how can we remove clones?

                        let expanded_expr = Expr::List(vec![
                            Expr::symbol("<-"),
                            accum.clone(),
                            Expr::List(vec![
                                Expr::symbol("+"),
                                expr_clone(accum),
                                expr_clone(value),
                            ]),
                        ]);

                        Ok(Some(expanded_expr))

                        // #todo
                        // Add more assignment operators: `-<-`, `*<-`, `\<-`, `map<-`, etc
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

                        Ok(Some(Expr::List(terms)))
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
                        // #todo remove this unwrap!!!
                        expr.range().unwrap(),
                    )))
                }
            }
        }
        _ => Ok(Some(expr)),
    }
}
