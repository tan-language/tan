use super::Expr;

impl Expr {
    pub fn iter(&self) -> ExprIter<'_> {
        ExprIter(self)
    }
}

// #Insight
// The iterator is implemented as a separate struct, for flexibility.

/// A depth-first Expr iterator.
pub struct ExprIter<'a>(&'a Expr);

// #TODO implement the depth-first iterator
// #TODO implement owned iterator
// #TODO implement mutable iterator
// #TODO https://aloso.github.io/2021/03/09/creating-an-iterator
