use crate::range::Ranged;

// #TODO consider `Ann`?
// #TODO keep the annotation as Expr, not String.

#[derive(Debug)]
pub struct Annotated<T>(pub T, pub Option<Vec<Ranged<String>>>);

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
