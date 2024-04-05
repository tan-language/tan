use std::collections::HashMap;

use crate::{context::Context, eval::eval};

use super::Expr;

impl Expr {
    // #todo this is some kind of map-reduce, try to use some kind of interator.
    // #todo alternatively, this implements some kind of visitor pattern.

    // #todo I think this is depth-first!
    // #todo we also need breadth-first!
    /// Transforms the expression by recursively applying the `f` mapping
    /// function.
    pub fn transform<F>(self, f: &F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        match self.extract() {
            (Expr::List(terms), ann) => {
                // #todo investigate this clone!!!!
                let terms = terms.iter().map(|t| t.clone().transform(f)).collect();
                let list = Expr::maybe_annotated(Expr::List(terms), ann);
                f(list)
            }
            // #todo ARGHHHHHH does not handle Map, Array, etc.
            _ => f(self),
        }
    }

    pub fn transform_mut<F>(self, f: &mut F) -> Self
    where
        F: FnMut(Self) -> Self,
    {
        match self.extract() {
            (Expr::List(terms), ann) => {
                // #todo investigate this clone!!!!
                let terms = terms.iter().map(|t| t.clone().transform_mut(f)).collect();
                let list = Expr::maybe_annotated(Expr::List(terms), ann);
                f(list)
            }
            // #todo write unit test for array quote
            // #todo ULTRA HACK: super nasty code here!
            // #todo properly handle array
            (Expr::Array(terms), ann) => {
                // #todo investigate this clone!!!!
                let terms: Vec<Expr> = terms
                    .borrow()
                    .clone()
                    .into_iter()
                    .map(|t| t.clone().transform_mut(f))
                    .collect();
                let array = Expr::maybe_annotated(Expr::array(terms), ann);
                f(array)
            }
            // #todo write unit test for map quote
            // #todo ULTRA HACK: super nasty code here! and super non-optimal.
            // #todo properly handle array
            (Expr::Map(map), ann) => {
                // #todo investigate this clone!!!!
                let map: HashMap<String, Expr> = map
                    .borrow()
                    .clone()
                    .into_iter()
                    .map(|(key, value)| (key, value.clone().transform_mut(f)))
                    .collect();
                let map = Expr::maybe_annotated(Expr::map(map), ann);
                f(map)
            }
            // #todo ARGHHHHHH does not handle Map, Array, etc.
            _ => f(self),
        }
    }

    /// Transforms the expression by recursively applying the `f` mapping
    /// function, breadth-first-search
    pub fn transform_bfs<F>(self, _f: &F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        todo!()
    }

    // #todo hack, move elsewhere
    // #todo ultra nasty code, remove all clones.
    pub fn quot(self, context: &mut Context) -> Self {
        match self.extract() {
            (Expr::List(terms), ann) => {
                if terms.is_empty() {
                    self
                } else if let Some(sym) = terms[0].unpack().as_symbol() {
                    if sym == "unquot" {
                        debug_assert!(terms.len() == 2);
                        // #todo remove the unwrap!
                        // #todo quote should return Result.
                        match eval(&terms[1], context) {
                            Ok(expr) => expr,
                            Err(error) => {
                                // #todo this is a temp (bad) solution.
                                // #todo somehow properly report this error.
                                eprintln!("{error:?}");
                                panic!("error in quoted expression: `{}`", &terms[1]);
                            }
                        }
                    } else {
                        let terms = terms.iter().map(|t| t.clone().quot(context)).collect();
                        Expr::maybe_annotated(Expr::List(terms), ann)
                    }
                } else {
                    let terms = terms.iter().map(|t| t.clone().quot(context)).collect();
                    Expr::maybe_annotated(Expr::List(terms), ann)
                }
            }
            (Expr::Array(terms), ann) => {
                // #todo investigate this clone!!!!
                let terms: Vec<Expr> = terms
                    .borrow()
                    .clone()
                    .into_iter()
                    .map(|t| t.quot(context))
                    .collect();

                Expr::maybe_annotated(Expr::array(terms), ann)
            }
            (Expr::Map(map), ann) => {
                // #todo investigate this clone!!!!
                let map: HashMap<String, Expr> = map
                    .borrow()
                    .clone()
                    .into_iter()
                    .map(|(key, value)| (key, value.quot(context)))
                    .collect();

                Expr::maybe_annotated(Expr::map(map), ann)
            }
            _ => self,
        }
    }

    /// Transforms the expression by recursively applying the `f` mapping
    /// function.
    pub fn filter_transform<F>(self, f: &F) -> Option<Self>
    where
        F: Fn(Self) -> Option<Self>,
    {
        match self.extract() {
            (Expr::List(terms), ann) => {
                // #todo investigate this clone!!!!
                let terms = terms
                    .iter()
                    .filter_map(|t| t.clone().filter_transform(f))
                    .collect();
                let list = Expr::maybe_annotated(Expr::List(terms), ann);
                f(list)
            }
            _ => f(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{api::parse_string, expr::Expr};

    pub fn identity_fn(expr: Expr) -> Expr {
        expr
    }

    #[test]
    fn transform_with_identity_function() {
        let input = "(quot (1 2 3 (4 5) (6 (+ 7 8)) 9 10))";

        let expr = parse_string(input).unwrap();

        let expr_string = expr.to_string();

        let expr_transformed = expr.transform(&identity_fn);

        assert_eq!(expr_string, expr_transformed.to_string());
    }
}
