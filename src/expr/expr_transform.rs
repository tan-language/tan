use std::collections::HashMap;

use crate::context::Context;

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
                let terms = terms.into_iter().map(|t| t.clone().transform(f)).collect();
                let list = Expr::maybe_annotated(Expr::List(terms), ann);
                f(list)
            }
            // #todo ARGHHHHHH does not handle Dict, Array, etc.
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
                let terms = terms
                    .into_iter()
                    .map(|t| t.clone().transform_mut(f))
                    .collect();
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
            // #todo write unit test for dict quote
            // #todo ULTRA HACK: super nasty code here! and super non-optimal.
            // #todo properly handle array
            (Expr::Dict(dict), ann) => {
                // #todo investigate this clone!!!!
                let dict: HashMap<String, Expr> = dict
                    .borrow()
                    .clone()
                    .into_iter()
                    .map(|(key, value)| (key, value.clone().transform_mut(f)))
                    .collect();
                let dict = Expr::maybe_annotated(Expr::dict(dict), ann);
                f(dict)
            }
            // #todo ARGHHHHHH does not handle Dict, Array, etc.
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
    pub fn quot(self, _context: &mut Context) -> Self {
        todo!();
        // match self.extract() {
        //     (Expr::List(terms), ann) => {
        //         // #todo investigate this clone!!!!
        //         let terms = terms
        //             .into_iter()
        //             .map(|t| {
        //                 let value = f(t.clone());
        //                 t.clone().transform_mut(f);
        //                 value
        //             })
        //             .collect();
        //         let list = Expr::maybe_annotated(Expr::List(terms), ann);
        //         list
        //     }
        //     // #todo write unit test for array quote
        //     // #todo ULTRA HACK: super nasty code here!
        //     // #todo properly handle array
        //     (Expr::Array(terms), ann) => {
        //         // #todo investigate this clone!!!!
        //         let terms: Vec<Expr> = terms
        //             .borrow()
        //             .clone()
        //             .into_iter()
        //             .map(|t| t.clone().transform_mut(f))
        //             .collect();
        //         let array = Expr::maybe_annotated(Expr::array(terms), ann);
        //         f(array)
        //     }
        //     // #todo write unit test for dict quote
        //     // #todo ULTRA HACK: super nasty code here! and super non-optimal.
        //     // #todo properly handle array
        //     (Expr::Dict(dict), ann) => {
        //         // #todo investigate this clone!!!!
        //         let dict: HashMap<String, Expr> = dict
        //             .borrow()
        //             .clone()
        //             .into_iter()
        //             .map(|(key, value)| (key, value.clone().transform_mut(f)))
        //             .collect();
        //         let dict = Expr::maybe_annotated(Expr::dict(dict), ann);
        //         f(dict)
        //     }
        //     // #todo ARGHHHHHH does not handle Dict, Array, etc.
        //     _ => f(self),
        // }
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
                    .into_iter()
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
