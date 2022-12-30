use crate::expr::Expr;

// #TODO consider `Ann`, `Ax`, `An`, `Av`, `Anned`

// #Insight
// The Annotated struct will be used a lot, it makes sense to use
// Option to avoid unnecessary HashMap allocations.

// #Insight
// Annotations are 'culled' in the parser, so we can use them for 'shebang'.

// #TODO get range from annotation.

#[derive(Debug, Clone)]
pub struct Ann<T>(pub T, pub Option<Vec<Expr>>);

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
