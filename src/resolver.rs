use crate::{
    context::Context,
    error::Error,
    eval::{eval, util::eval_module},
    expr::{annotate, annotate_type, Expr},
    util::is_reserved_symbol,
};

// #todo resolver should handle 'use'!!! and _strip_ use expressions.

// #todo rename file to `sema`?
// #todo split into multiple passes?
// #todo it currently includes the optimize pass, split!

// #insight resolve_type and resolve_invocable should be combined, cannot be separate passes.

// #todo signature should also encode the return type!!
// #todo how to handle VARARG functions ?!?!
pub fn compute_signature(args: &[Expr]) -> String {
    let mut signature = Vec::new();

    for arg in args {
        signature.push(arg.static_type().to_string())
    }

    signature.join("$$")
}

pub fn compute_dyn_signature(args: &[Expr], context: &Context) -> String {
    let mut signature = Vec::new();

    for arg in args {
        signature.push(arg.dyn_type(context).to_string())
    }

    signature.join("$$")
}

// -----------------------------------------------------------------------------
// #WARN the resolver is temporarily disabled.

// #todo explain what the Resolver is doing.
/// The resolver performs the following functions:
/// - statically infers types
/// - resolves `use`d modules
pub struct Resolver {
    errors: Vec<Error>,
}

impl Resolver {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    // #todo no need to resolve basic types!

    // #todo maybe return multiple errors?
    // #todo should pass &Expr/Rc<Expr>
    pub fn resolve_expr(&mut self, expr: Expr, context: &mut Context) -> Expr {
        // eprintln!("<<< {} >>>", expr);
        // #todo update the original annotations!
        // #todo need to handle _all_ Expr variants.
        match expr {
            Expr::Annotated(_, ref ann) => {
                // #insight
                // We have to resolve even if the expr has a type annotation, it
                // might be missing a 'method' or other annotation.

                // #todo refactor and/or extract this functionality.
                let mut resolved_expr = self.resolve_expr(expr.unpack().clone(), context);
                if let Expr::Annotated(_, ref mut resolved_ann) = resolved_expr {
                    for (key, value) in ann {
                        resolved_ann.insert(key.clone(), value.clone());
                    }
                }
                resolved_expr
            }
            Expr::Int(_) => annotate_type(expr, "Int"),
            Expr::Float(_) => annotate_type(expr, "Float"),
            #[cfg(feature = "dec")]
            Expr::Dec(_) => annotate_type(expr, "Dec"),
            Expr::String(_) => annotate_type(expr, "String"),
            // #todo hmm... ultra-hack.
            Expr::Array(_) => annotate_type(expr, "Array"), // #todo what type of array?
            Expr::KeySymbol(_) => annotate_type(expr, "KeySymbol"),
            Expr::Symbol(ref sym) => {
                if is_reserved_symbol(sym) {
                    return annotate_type(expr, "Symbol");
                }

                // #todo handle 'PathSymbol'
                // #todo handle a Dict invocable (and other invocables).
                // #todo please note that multiple-dispatch is supposed to be dynamic!

                let result = if let Some(Expr::Symbol(method)) = expr.annotation("method") {
                    context.scope.get(method)
                } else {
                    // #todo ultra-hack just fall-back to 'function' name if method does not exist.
                    context.scope.get(sym)
                };

                let Some(value) = result else {
                    return annotate_type(expr, "Symbol");
                };

                // #hint this could help: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.unwrap_or_clone
                let value = self.resolve_expr((*value).clone(), context);
                return annotate(expr, "type", value.static_type().clone());
            }
            Expr::List(ref list) => {
                if list.is_empty() {
                    // This is handled statically, in the parser, but an extra, dynamic
                    // check is needed in resolve to handle the case where the
                    // expression is constructed programmatically (e.g. self-modifying code,
                    // dynamically constructed expression, homoiconicity, etc).
                    return expr;
                }

                // The unwrap here is safe.
                let head = list.first().unwrap();
                let tail = &list[1..];

                // #todo there should be no mangling, just an annotation!
                // #todo also perform error checking here, e.g. if the head is invocable.
                // #todo Expr.is_invocable, Expr.get_invocable_name, Expr.get_type
                // #todo handle non-symbol cases!
                // #todo signature should be the type, e.g. +::(Func Int Int Int) instead of +$$Int$$Int
                // #todo should handle Func!!
                // #todo convert to match, extract the iteration.
                if let Some(sym) = head.as_symbol() {
                    // #todo special handling of def
                    if sym == "let" {
                        // #todo also report some of these errors statically, maybe in a sema phase?
                        let mut args = tail.iter();

                        let mut resolved_let_list = vec![head.clone()];

                        loop {
                            let Some(sym) = args.next() else {
                                break;
                            };

                            let Some(value) = args.next() else {
                                // #todo error?
                                break;
                            };

                            let Some(s) = sym.as_symbol() else {
                                self.errors.push(Error::invalid_arguments(
                                    &format!("`{sym}` is not a Symbol"),
                                    sym.range(),
                                ));
                                // Continue to detect more errors.
                                continue;
                            };

                            if is_reserved_symbol(s) {
                                self.errors.push(Error::invalid_arguments(
                                    &format!("let cannot shadow the reserved symbol `{s}`"),
                                    sym.range(),
                                ));
                                // Continue to detect more errors.
                                continue;
                            }

                            let value = self.resolve_expr(value.clone(), context);
                            // let mut map = expr.1.clone().unwrap_or_default();
                            // map.insert("type".to_owned(), value.static_type().clone());
                            // ann = Some(map);

                            resolved_let_list.push(sym.clone());
                            resolved_let_list.push(value.clone());

                            // #todo notify about overrides? use `set`?
                            // #todo for some reason, this causes infinite loop
                            // #todo why is this needed in the first place?

                            // Try to apply definitions.

                            let result = eval(&value, context);

                            match result {
                                Ok(value) => {
                                    // #todo notify about overrides? use `set`?
                                    context.scope.insert(s, value);
                                }
                                Err(error) => {
                                    self.errors.push(error);
                                }
                            }
                        }

                        return Expr::maybe_annotated(
                            Expr::List(resolved_let_list),
                            head.annotations(),
                        );
                    } else if sym == "use" {
                        // #insight I moved this code to eval for the moment.
                        // #todo temp hack!!!

                        // #todo properly handle this here, strip the use expression, remove from eval
                        // #todo move this to resolve? use should be stripped at dyn-time
                        // #todo also support path as symbol.

                        // Import a directory as a module.

                        let Some(term) = tail.first() else {
                            self.errors.push(Error::invalid_arguments(
                                "malformed use expression",
                                expr.range(),
                            ));
                            // #todo what to return here?
                            return Expr::One;
                        };

                        let Some(module_path) = term.as_string() else {
                            self.errors.push(Error::invalid_arguments(
                                "malformed use expression",
                                expr.range(),
                            ));
                            // #todo what to return here?
                            return Expr::One;
                        };

                        // #todo make sure paths are relative to the current file.
                        let result = eval_module(module_path, context, false);

                        if let Err(errors) = result {
                            // #todo precise formating is _required_ here!
                            // eprintln!("{}", format_errors(&errors));
                            // dbg!(errors);
                            self.errors.push(Error::failed_use(module_path, errors)); // #todo add note with information here!
                                                                                      // #todo what to return here?
                            return Expr::One;
                        };

                        let Ok(Expr::Module(module)) = result else {
                            // #todo could use a panic here, this should never happen.
                            self.errors.push(Error::failed_use(module_path, vec![])); // #todo add note with information!
                                                                                      // #todo what to return here?
                            return Expr::One;
                        };

                        // Import public names from module scope into the current scope.

                        // #todo support (use "/path/to/module" *) or (use "/path/to/module" :embed)

                        // #todo temp, needs cleanup!
                        let bindings = module.scope.bindings.borrow().clone();
                        for (name, value) in bindings {
                            // #todo temp fix to not override the special var
                            if name.starts_with('*') {
                                continue;
                            }

                            // #todo ONLY export public bindings

                            let name = format!("{}/{}", module.stem, name);

                            // #todo assign as top-level bindings!
                            context.scope.insert(name, value.clone());
                        }

                        // #todo what could we return here? the Expr::Module?
                        Expr::One
                    } else if sym == "Func" {
                        // let mut resolved_tail = Vec::new();
                        // for term in tail {
                        //     resolved_tail.push(self.resolve_expr(term.clone(), env));
                        // }

                        // let head = if let Expr::Symbol(ref sym) = head.unpack() {
                        //     if !is_reserved_symbol(sym) {
                        //         // #todo should recursively resolve first!
                        //         // #todo signature should also encode the return type!!
                        //         // #todo how to handle VARARG functions ?!?!

                        //         let mut signature = Vec::new();

                        //         for term in &resolved_tail {
                        //             signature.push(term.static_type().to_string())
                        //         }

                        //         let signature = signature.join("$$");

                        //         annotate(
                        //             head.clone(),
                        //             "method",
                        //             Expr::Symbol(format!("{sym}$${signature}")),
                        //         )
                        //     } else {
                        //         head.clone()
                        //     }
                        // } else {
                        //     head.clone()
                        // };

                        // // #insight head should get resolved after the tail.
                        // let head = self.resolve_expr(head, env);

                        // let mut list = vec![head.clone()];
                        // list.extend(resolved_tail);

                        // return Expr::maybe_annotated(Expr::List(list), head.annotations());

                        // #todo do something ;-)
                        // #todo this is a temp hack, we don't resolve inside a function, argh!

                        // dbg!(&head);
                        // dbg!(&tail);

                        // println!("^^^^^");

                        // // Evaluate the arguments before calling the function.
                        // let args = eval_args(tail, env)?;

                        // // #todo ultra-hack to kill shared ref to `env`.
                        // let params = params.clone();
                        // let body = body.clone();

                        // // Dynamic scoping, #todo convert to lexical.

                        // env.push_new_scope();

                        // for (param, arg) in params.iter().zip(args) {
                        //     let Ann(Expr::Symbol(param), ..) = param else {
                        //             return Err(Ranged(Error::invalid_arguments("parameter is not a symbol"), param.get_range()));
                        //         };

                        //     env.insert(param, arg);
                        // }

                        // let result = eval(&body, env);

                        // env.pop();

                        // result

                        expr
                    } else {
                        let mut resolved_tail = Vec::new();
                        for term in tail {
                            resolved_tail.push(self.resolve_expr(term.clone(), context));
                        }

                        let head = if let Some(sym) = head.as_symbol() {
                            if !is_reserved_symbol(sym) {
                                // #todo should recursively resolve first!
                                // #todo signature should also encode the return type!!
                                // #todo how to handle VARARG functions ?!?!

                                let signature = compute_signature(&resolved_tail);

                                annotate(
                                    head.clone(),
                                    "method",
                                    Expr::Symbol(format!("{sym}$${signature}")),
                                )
                            } else {
                                head.clone()
                            }
                        } else {
                            head.clone()
                        };

                        // #insight head should get resolved after the tail.
                        let head = self.resolve_expr(head, context);

                        let mut list = vec![head.clone()];
                        list.extend(resolved_tail);

                        return Expr::maybe_annotated(Expr::List(list), head.annotations());
                    }
                } else {
                    // #todo handle map lookup case.
                    expr
                }
            }
            _ => expr, // #todo add a debugging trace here!
        }
    }

    // #todo better explain what this function does.
    // #todo what exactly is this env? how is this mutated?
    // Resolve pass (typechecking, definitions, etc)
    pub fn resolve(&mut self, expr: Expr, context: &mut Context) -> Result<Expr, Vec<Error>> {
        let expr = self.resolve_expr(expr, context);

        if self.errors.is_empty() {
            // #todo
            Ok(expr)
        } else {
            let errors = std::mem::take(&mut self.errors);
            Err(errors)
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

// #todo move to resolver_test.rs

#[cfg(test)]
mod tests {
    use crate::{api::parse_string, context::Context, resolver::Resolver};

    #[test]
    fn resolve_specializes_functions() {
        // let expr = parse_string("(let a 1)").unwrap();
        // let expr = parse_string("(+ 1 2)").unwrap();
        // let expr = parse_string("(do (let a 1.3) (+ a 2.2))").unwrap();
        let expr = parse_string("(do (let a 1.3) (+ a (+ 1.0 2.2)))").unwrap();
        dbg!(&expr);
        let mut resolver = Resolver::new();
        let mut context = Context::new();
        let expr = resolver.resolve(expr, &mut context).unwrap();
        dbg!(&expr);
        // #todo make this a test!
    }
}
