use crate::span::Spanned;

pub type Ann = Vec<Spanned<String>>;

// #TODO consider Anned?
#[derive(Debug)]
pub struct Annotated<T>(pub T, pub Ann);

impl<T> Annotated<T> {
    pub fn new(value: T) -> Self {
        Self(value, Vec::new())
    }
}

impl<T> AsRef<T> for Annotated<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
