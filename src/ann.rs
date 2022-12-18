use crate::span::Spanned;

// #TODO consider `Ann`?

#[derive(Debug)]
pub struct Annotated<T>(pub T, pub Option<Vec<Spanned<String>>>);

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
