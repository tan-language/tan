// use std::fmt;

use crate::range::Ranged;

// #TODO consider `Ann`?
// #TODO keep the annotation as Expr, not String.

#[derive(Debug, Clone)]
pub struct Annotated<T>(pub T, pub Option<Vec<Ranged<String>>>);

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
