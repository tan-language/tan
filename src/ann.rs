use std::collections::HashMap;

pub type Ann = HashMap<String, String>;

// #TODO consider Anned?
#[derive(Debug)]
pub struct Annotated<T>(T, Ann);

impl<T> Annotated<T> {
    pub fn new(value: T) -> Self {
        Self(value, HashMap::new())
    }
}

impl<T> AsRef<T> for Annotated<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
