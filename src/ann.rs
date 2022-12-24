// use std::fmt;

use crate::expr::Expr;

// #TODO consider `Ann`?
// #TODO keep the annotation as Expr, not String.

// #Insight
// The Annotated struct will be used a lot, it makes sense to use
// Option to avoid unnecessary HashMap allocations.

#[derive(Debug, Clone)]
pub struct Annotated<T>(pub T, pub Option<Vec<Expr>>);

impl<T> Annotated<T> {
    pub fn new(value: T) -> Self {
        Self(value, None)
    }
}

// #TODO is this good? it hides the wrapped data.
// impl<T> fmt::Display for Annotated<T>
// where
//     T: fmt::Display,
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         self.0.fmt(f)
//     }
// }

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
