// use std::fmt;

use crate::expr::Expr;

// #TODO consider `Ann`?
// #TODO keep the annotation as Expr, not String.

// #Insight
// The Annotated struct will be used a lot, it makes sense to use
// Option to avoid unnecessary HashMap allocations.

// #Insight
// Annotations are 'culled' in the parser, so we can use them for 'shebang'.

#[derive(Debug, Clone)]
pub struct Annotated<T>(pub T, pub Option<Vec<Expr>>);

impl<T> Annotated<T> {
    pub fn new(value: T) -> Self {
        Self(value, None)
    }
}

impl<T> AsRef<T> for Annotated<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Annotated<T> {
    fn from(value: T) -> Self {
        Annotated::new(value)
    }
}
