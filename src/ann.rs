use core::fmt;
use std::collections::HashMap;

use crate::expr::Expr;

// #TODO somehow annotations should trigger macros.

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
pub struct Ann<T>(pub T, pub Option<HashMap<String, Expr>>);

impl<T> Ann<T> {
    pub fn typed(value: T, type_expr: Expr) -> Self {
        let mut map = HashMap::new();
        map.insert("type".to_owned(), type_expr);
        Self(value, Some(map))
    }

    pub fn set_type_annotation(&mut self, type_expr: Expr) {
        let mut map = self.1.clone().unwrap_or_default();
        map.insert("type".to_owned(), type_expr);
        self.1 = Some(map);
    }
}

// #TODO get_type
// #TODO get_range/span

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
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            write!(f, "{:?}@[{anns}]", self.0)
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
