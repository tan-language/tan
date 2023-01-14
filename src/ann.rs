use core::fmt;

use crate::expr::Expr;

// #TODO
// - Uppercase -> (:type Uppercase)
// - lowercase -> (:lowercase true)
// - (:key value)

// #TODO consider {+/-}lowercase -> true/false

// #TODO consider `Ann`, `Ax`, `An`, `Av`, `Anned`

// #Insight
// The Annotated struct will be used a lot, it makes sense to use
// Option to avoid unnecessary HashMap allocations.

// #Insight
// Annotations are 'culled' in the parser, so we can use them for 'shebang'.

// #TODO get range from annotation.

#[derive(Clone)]
pub struct Ann<T>(pub T, pub Option<Vec<Expr>>);

impl<T> Ann<T> {
    pub fn typed(value: T, ann: Expr) -> Self {
        Self(value, Some(vec![ann]))
    }
}

impl<T> fmt::Debug for Ann<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.1.is_some() {
            let anns = self
                .1
                .clone()
                .unwrap()
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(",");
            write!(f, "{:?}::{anns}", self.0)
        } else {
            write!(f, "{:?}", self.0)
        }
    }
}

impl<T> fmt::Display for Ann<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> Ann<T> {
    pub fn new(value: T) -> Self {
        Self(value, None)
    }
}

impl<T> AsRef<T> for Ann<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsRef<T> for Box<Ann<T>> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Ann<T> {
    fn from(value: T) -> Self {
        Ann::new(value)
    }
}
