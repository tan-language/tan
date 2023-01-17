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
}

impl<T> Ann<T> {
    // #TODO introduce `Unknown` type? or just use `One`?
    // #TODO this should somehow return &Expr.
    pub fn get_type(&self) -> Expr {
        // #TODO optimize get_type for literals, and even skip adding as annotation?
        let Some(ref ann ) = self.1 else {
                // #TODO One == Any ?
                return Expr::symbol("One");
            };

        let Some(ann) = ann.get("type") else {
                return Expr::symbol("One");
            };

        // #TODO we should avoid this, try to return ref somehow (e.g. have One as predefined global? or static in Ann?)
        ann.clone()
    }

    pub fn set_type(&mut self, type_expr: Expr) {
        self.1
            .get_or_insert(HashMap::new())
            .insert("type".to_owned(), type_expr);
    }

    // #TODO find a better name.
    pub fn to_type_string(&self) -> String {
        let type_ann = self.get_type();

        if let Expr::Symbol(type_name) = type_ann {
            type_name
        } else {
            "One".to_string()
        }
    }

    // pub fn set_method()
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
