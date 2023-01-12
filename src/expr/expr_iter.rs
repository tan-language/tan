use std::mem;

use crate::ann::Ann;

use super::Expr;

impl Ann<Expr> {
    pub fn iter(&self) -> ExprIter<'_> {
        ExprIter {
            children: std::slice::from_ref(self),
            parent: None,
        }
    }
}

// #Insight
// The iterator is implemented as a separate struct, for flexibility.

/// A depth-first Expr iterator.
#[derive(Default)]
pub struct ExprIter<'a> {
    children: &'a [Ann<Expr>],
    parent: Option<Box<ExprIter<'a>>>,
}

impl<'a> Iterator for ExprIter<'a> {
    type Item = &'a Ann<Expr>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.children.get(0) {
            None => match self.parent.take() {
                Some(parent) => {
                    // continue with the parent expr
                    *self = *parent;
                    self.next()
                }
                None => None,
            },
            Some(Ann(Expr::List(children), ..)) => {
                self.children = &self.children[1..];

                // start iterating the sub-trees
                *self = ExprIter {
                    children: children.as_slice(),
                    parent: Some(Box::new(mem::take(self))),
                };
                self.next()
            }
            _ => self.children.get(0),
        }
    }
}

// #TODO implement the depth-first iterator
// #TODO implement owned iterator
// #TODO implement mutable iterator
// #TODO https://aloso.github.io/2021/03/09/creating-an-iterator

#[cfg(test)]
mod tests {
    use crate::{
        api::eval_string,
        eval::{env::Env, prelude::setup_prelude},
    };

    #[test]
    fn expr_iter_performs_depth_first_iteration() {
        let env = &mut setup_prelude(Env::new());
        let input = "(quot (1 2 3 (4 5) (6 (7 8)) 9 10))";
        let expr = eval_string(input, env).unwrap();
        let value = expr.to_string();
        let expected = "(1 2 3 (4 5) (6 (7 8)) 9 10)";
        assert_eq!(value, expected);
    }
}
