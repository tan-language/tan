use core::fmt;
use std::collections::HashMap;

use crate::expr::{format_value, Expr};

// #TODO somehow annotations should trigger macros.

// #TODO
// - Uppercase -> (:type Uppercase)
// - lowercase -> (:lowercase true)
// - (:key value)

// #TODO consider {+/-}lowercase -> true/false

// #TODO consider `Ann`, `Ax`, `An`, `Av`, `Anned`
// #TODO define 'special' annotations, e.g. `type`, `range`, `method`, etc.
// #TODO maybe use ALLCAP for special annotations? e.g. TYPE, RANGE, METHOD.

// #Insight
// The Annotated struct will be used a lot, it makes sense to use
// Option to avoid unnecessary HashMap allocations.

// #Insight
// Annotations are 'culled' in the parser, so we can use them for 'shebang'.

// #TODO get range from annotation.

#[derive(Clone)]
pub struct Ann<T>(pub T, pub Option<HashMap<String, Expr>>);

impl<T> Ann<T> {
    // #TODO consider `with_type`.
    pub fn typed(value: T, type_expr: Expr) -> Self {
        let mut map = HashMap::new();
        map.insert("type".to_owned(), type_expr);
        Self(value, Some(map))
    }
}

impl<T> Ann<T> {
    pub fn set_annotation(&mut self, name: impl Into<String>, expr: Expr) {
        self.1
            .get_or_insert(HashMap::new())
            .insert(name.into(), expr);
    }

    pub fn get_annotation(&self, name: impl Into<String>) -> Option<&Expr> {
        let Some(ref ann ) = self.1 else {
            return None;
        };

        ann.get(&name.into())
    }

    pub fn contains_annotation(&self, name: impl Into<String>) -> bool {
        let Some(ref ann ) = self.1 else {
            return false;
        };

        ann.contains_key(&name.into())
    }

    pub fn set_type(&mut self, type_expr: Expr) {
        self.set_annotation("type", type_expr);
    }

    // #TODO consider specialization for `Ann<Expr>` that also checks the target?
    // #TODO optimize get_type for literals, and even skip adding as annotation?
    // #TODO introduce `Unknown` type? or just use `One`?
    pub fn get_type(&self) -> &Expr {
        self.get_annotation("type").unwrap_or(&Expr::One)
    }

    pub fn to_type_string(&self) -> String {
        format_value(self.get_type())
    }

    // #TODO get_range/span
    // #TODO get_method (multiple-dispatch)
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
