use std::mem;

use super::Expr;

impl Expr {
    pub fn iter(&self) -> ExprIter<'_> {
        ExprIter {
            children: std::slice::from_ref(self),
            parent: None,
        }
    }
}

// #Insight
// The iterator is implemented as a separate struct, for flexibility.

// #todo support in-order, pre-order, post-order
// #todo implement owned iterator
// #todo implement mutable iterator
// #todo https://aloso.github.io/2021/03/09/creating-an-iterator

// #todo is this really DFS?
/// A depth-first Expr iterator.
#[derive(Default)]
pub struct ExprIter<'a> {
    children: &'a [Expr],
    parent: Option<Box<ExprIter<'a>>>,
}

impl<'a> Iterator for ExprIter<'a> {
    type Item = &'a Expr;

    // #todo this does not traverse Array, Dict, etc.
    fn next(&mut self) -> Option<Self::Item> {
        let Some(expr) = self.children.get(0) else {
            return match self.parent.take() {
                Some(parent) => {
                    // continue with the parent expr
                    *self = *parent;
                    self.next()
                }
                None => None,
            };
        };

        match expr.unpack() {
            Expr::List(children) => {
                self.children = &self.children[1..];
                // iterate the sub-trees
                *self = ExprIter {
                    children: children.as_slice(),
                    parent: Some(Box::new(mem::take(self))),
                };
                Some(expr)
            }
            _ => {
                self.children = &self.children[1..];
                Some(expr)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    #[test]
    fn expr_iter_performs_depth_first_iteration() {
        let input = "(quot (1 2 3 (4 5) (6 (+ 7 8)) 9 10))";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();

        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();

        let expr = &expr[0];

        let terms: Vec<String> = expr.iter().map(|ax| ax.to_string()).collect();
        let expected_terms = vec![
            "(quot (1 2 3 (4 5) (6 (+ 7 8)) 9 10))",
            "quot",
            "(1 2 3 (4 5) (6 (+ 7 8)) 9 10)",
            "1",
            "2",
            "3",
            "(4 5)",
            "4",
            "5",
            "(6 (+ 7 8))",
            "6",
            "(+ 7 8)",
            "+",
            "7",
            "8",
            "9",
            "10",
        ];
        assert_eq!(terms, expected_terms);
    }
}
