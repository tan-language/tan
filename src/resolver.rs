use std::collections::HashMap;

use crate::{
    ann::Ann,
    error::Error,
    eval::{env::Env, eval},
    expr::Expr,
    range::Ranged,
    util::is_reserved_symbol,
};

// #TODO rename file to `sema`?
// #TODO support multiple errors.
// #TODO split into multiple passes?
// #TODO it currently includes the optimize pass, split!

// #Insight resolve_type and resolve_invocable should be combined, cannot be separate passes.

pub struct Resolver {
    errors: Vec<Ranged<Error>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    fn push_error(&mut self, error: Ranged<Error>) {
        self.errors.push(error);
    }

    pub fn resolve_expr(&mut self, mut expr: Ann<Expr>, env: &mut Env) -> Ann<Expr> {
        // #TODO update the original annotations!
        // #TODO need to handle _all_ Expr variants.
        match expr {
            Ann(Expr::Int(_), _) => {
                expr.set_type(Expr::symbol("Int"));
                expr
            }
            Ann(Expr::Float(_), _) => {
                expr.set_type(Expr::symbol("Float"));
                expr
            }
            Ann(Expr::String(_), _) => {
                expr.set_type(Expr::symbol("String"));
                expr
            }
            Ann(Expr::KeySymbol(_), _) => {
                expr.set_type(Expr::symbol("KeySymbol"));
                expr
            }
            // #TODO hmm... ultra-hack.
            Ann(Expr::Array(..), _) => {
                expr.set_type(Expr::symbol("Array"));
                expr
            }
            Ann(Expr::Symbol(ref sym), _) => {
                if is_reserved_symbol(sym) {
                    expr.set_type(Expr::symbol("Symbol"));
                    return expr;
                }

                // #TODO handle 'PathSymbol'

                // #TODO handle a Dict invocable (and other invocables).
                // #TODO please note that multiple-dispatch is supposed to be dynamic!

                let result = if let Some(Expr::Symbol(method)) = expr.get_annotation("method") {
                    env.get(method)
                } else {
                    // #TODO ultra-hack just fall-back to 'function' name if method does not exist.
                    env.get(sym)
                };

                let Some(value) = result else {
                    expr.set_type(Expr::symbol("Symbol"));
                    return expr;
                };

                let value = self.resolve_expr(value.clone(), env);
                expr.set_type(value.get_type().clone());
                expr
            }
            Ann(Expr::List(ref list), _) => {
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

                // #TODO there should be no mangling, just an annotation!
                // #TODO also perform error checking here, e.g. if the head is invocable.
                // #TODO Expr.is_invocable, Expr.get_invocable_name, Expr.get_type
                // #TODO handle non-symbol cases!
                // #TODO signature should be the type, e.g. +::(Func Int Int Int) instead of +$$Int$$Int
                if let Ann(Expr::Symbol(ref sym), _) = head {
                    // #TODO special handling of def
                    if sym == "let" {
                        // #TODO also report some of these errors statically, maybe in a sema phase?
                        let mut args = tail.iter();

                        let mut resolved_let_list = vec![Ann::new(Expr::symbol("let"))];
                        let mut ann = None;

                        loop {
                            let Some(sym) = args.next() else {
                                break;
                            };

                            let Some(value) = args.next() else {
                                // #TODO error?
                                break;
                            };

                            let Ann(Expr::Symbol(s), ..) = sym else {
                                self.push_error(Ranged(Error::invalid_arguments(format!("`{sym}` is not a Symbol")), sym.get_range()));
                                // Continue to detect more errors.
                                continue;
                            };

                            if is_reserved_symbol(s) {
                                self.push_error(Ranged(
                                    Error::invalid_arguments(format!(
                                        "let cannot shadow the reserved symbol `{s}`"
                                    )),
                                    sym.get_range(),
                                ));
                                // Continue to detect more errors.
                                continue;
                            }

                            let value = self.resolve_expr(value.clone(), env);
                            let mut map = expr.1.clone().unwrap_or_default();
                            map.insert("type".to_owned(), value.get_type().clone());
                            ann = Some(map);

                            resolved_let_list.push(sym.clone());
                            resolved_let_list.push(value.clone());

                            // #TODO notify about overrides? use `set`?
                            // #TODO for some reason, this causes infinite loop
                            // #TODO why is this needed in the first place?

                            // #TODO nasty code, revisit
                            // Try to apply definitions.

                            let result = eval(&value, env);

                            let Ok(value) = eval(&value, env) else {
                                // #TODO properly handle the error!
                                let err = result.unwrap_err();
                                dbg!(err);
                                // #TODO totally random.
                                return value;
                            };

                            // #TODO notify about overrides? use `set`?
                            env.insert(s, value);
                        }

                        Ann(Expr::List(resolved_let_list), ann)
                    } else {
                        let mut resolved_tail = Vec::new();
                        for term in tail {
                            resolved_tail.push(self.resolve_expr(term.clone(), env));
                        }

                        let head = if let Ann(Expr::Symbol(ref sym), ann_sym) = head {
                            let mut ann_sym = ann_sym.clone();

                            if !is_reserved_symbol(sym) {
                                // #TODO should recursively resolve first!

                                let mut signature = Vec::new();

                                for term in &resolved_tail {
                                    signature.push(term.to_type_string())
                                }

                                let signature = signature.join("$$");

                                ann_sym.get_or_insert(HashMap::new()).insert(
                                    "method".to_owned(),
                                    Expr::Symbol(format!("{sym}$${signature}")),
                                );
                            };

                            Ann(Expr::Symbol(sym.clone()), ann_sym)
                        } else {
                            head.clone()
                        };

                        // #Insight head should get resolved after the tail.
                        let head = self.resolve_expr(head, env);

                        let mut list = vec![head.clone()];
                        list.extend(resolved_tail);

                        Ann(Expr::List(list), head.1)
                    }
                } else {
                    // #TODO handle map lookup case.
                    expr
                }
            }
            _ => expr,
        }
    }

    pub fn resolve(
        &mut self,
        expr: Ann<Expr>,
        env: &mut Env,
    ) -> Result<Ann<Expr>, Vec<Ranged<Error>>> {
        let expr = self.resolve_expr(expr, env);

        if self.errors.is_empty() {
            // #TODO
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

#[cfg(test)]
mod tests {
    use crate::{api::parse_string, eval::env::Env, resolver::Resolver};

    #[test]
    fn resolve_specializes_functions() {
        // let expr = parse_string("(let a 1)").unwrap();
        // let expr = parse_string("(+ 1 2)").unwrap();
        // let expr = parse_string("(do (let a 1.3) (+ a 2.2))").unwrap();
        let expr = parse_string("(do (let a 1.3) (+ a (+ 1.0 2.2)))").unwrap();
        dbg!(&expr);
        let mut resolver = Resolver::new();
        let mut env = Env::prelude();
        let expr = resolver.resolve(expr, &mut env).unwrap();
        dbg!(&expr);
    }
}
