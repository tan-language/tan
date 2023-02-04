use crate::{
    ann::Ann,
    error::Error,
    eval::{env::Env, eval},
    expr::Expr,
    range::Ranged,
    util::is_reserved_symbol,
};

// #Insight it mutates the env which is used in eval also!

// #TODO remove the macro definitions from the AST
// #TODO consider renaming the expr parameter to ast?
// #TODO macro_expand (and all comptime/static passes should return Vec<Ranged<Error>>>)

/// Expands macro invocations, at comptime.
pub fn macro_expand(expr: Ann<Expr>, env: &mut Env) -> Result<Option<Ann<Expr>>, Ranged<Error>> {
    match expr {
        Ann(Expr::List(ref list), ..) => {
            if list.is_empty() {
                // This is handled statically, in the parser, but an extra, dynamic
                // check is needed in the evaluator to handle the case where the
                // expression is constructed programmatically (e.g. self-modifying code,
                // dynamically constructed expression, homoiconicity, etc).
                return Ok(None);
            }

            let head = list.first().unwrap(); // The unwrap here is safe.
            let tail = &list[1..];

            // Evaluate the head
            let Ok(head) = eval(head, env) else {
                // Don't err if we cannot eval the head.
                println!("~~~~~~~~~~~~~ THIS {head}");
                return Ok(Some(expr));
            };

            // let Ok(head) = env.get(head) else {
            //     println!("~~~~~~~~~~~~~ THIS {head}");
            //     return Ok(Some(expr));
            // }

            // println!("--- {head:?}");

            // #TODO can we remove this as_ref?
            match head.as_ref() {
                Expr::Macro(params, body) => {
                    // This is the actual macro-expansion

                    // #Insight
                    // Macro arguments are lazily evaluated.

                    println!("*********");

                    // dbg!(&params);
                    // dbg!(&body);

                    let args = tail;

                    // #TODO ultra-hack to kill shared ref to `env`.
                    let params = params.clone();
                    let body = body.clone();

                    // #TODO what kind of scoping is this?

                    env.push_new_scope();

                    for (param, arg) in params.iter().zip(args) {
                        let Ann(Expr::Symbol(param), ..) = param else {
                                return Err(Error::invalid_arguments("parameter is not a symbol").into());
                            };

                        env.insert(param, arg.clone());
                    }

                    // dbg!(&body);

                    let result = eval(&body, env)?;

                    dbg!(&result);

                    env.pop();

                    Ok(Some(result))
                }
                Expr::Symbol(sym) => {
                    println!("___ {sym}");
                    // #TODO actually we should use `def` for this purpose, instead of `let`.
                    if sym == "let" {
                        let mut args = tail.iter();

                        // #TODO should be def, no loop.

                        let Some(binding_sym) = args.next() else {
                            return Err(Error::invalid_arguments("missing binding symbol").into());
                        };

                        let Some(binding_value) = args.next() else {
                            return Err(Error::invalid_arguments("missing binding value").into());
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

                        if let Some(Ann(Expr::Macro(..), ..)) = binding_value {
                            // #TODO put all the definitions in one pass.
                            // Only define macros in this pass.
                            env.insert(s, binding_value.unwrap());
                            return Ok(None);
                        }

                        // env.insert(s, binding_value.unwrap());
                        println!("^^^^^^^^^^^^^^ {expr}");
                        // let expr = macro_expand(expr, env);
                        Ok(Some(
                            Expr::List(vec![
                                Expr::Symbol("let".to_owned()).into(),
                                binding_sym.clone(),
                                binding_value.unwrap(), // #TODO argh, remove the unwrap!
                            ])
                            .into(),
                        ))
                    } else if sym == "Macro" {
                        dbg!(&tail);
                        let [args, body] = tail else {
                            return Err(Error::invalid_arguments("malformed macro definition").into());
                        };

                        let Ann(Expr::List(params), ..) = args else {
                            return Err(Error::invalid_arguments("malformed macro parameters definition").into());
                        };

                        // #TODO optimize!
                        Ok(Some(
                            Expr::Macro(params.clone(), Box::new(body.clone())).into(),
                        ))
                    } else {
                        // Other kind of list with symbol head, macro-expand tail.
                        println!("+++++++++++++++++ {head}");

                        let mut terms = Vec::new();
                        terms.push(head.clone());
                        for term in tail {
                            println!("..... {head} {term}");
                            let term = macro_expand(term.clone(), env)?;
                            if let Some(term) = term {
                                println!("%%%%%% {head} {term}");
                                terms.push(term);
                            }
                        }

                        println!("!!!!===>>>>>> {head} {}", Expr::List(terms.clone()));

                        // if let Some(expr) = m

                        Ok(Some(Expr::List(terms).into()))
                    }
                }
                _ => {
                    // Other kind of list with non-symbol head, macro-expand tail.
                    println!("!!!!! MEGA COOL !!!!!!");

                    let mut terms = Vec::new();
                    terms.push(head.clone());
                    for term in tail {
                        // println!("..... {term:?}");
                        let term = macro_expand(term.clone(), env)?;
                        if let Some(term) = term {
                            terms.push(term);
                        }
                    }

                    // println!("=== {terms:?}");

                    Ok(Some(Expr::List(terms).into()))
                }
            }
        }
        _ => Ok(Some(expr)),
    }
}
