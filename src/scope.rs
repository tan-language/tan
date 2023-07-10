use std::{collections::HashMap, rc::Rc};

use crate::expr::Expr;

pub struct Scope {
    pub parent: Option<Rc<Scope>>,
    pub bindings: HashMap<String, Rc<Expr>>,
    // #idea have separate values/annotations!!!
    // #idea annotate only named expressions/bindings, don't annotate literals! to make the above work.
}

impl Scope {
    // #todo consider different name.
    pub fn prelude() -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(), // #todo initialize with prelude!
        }
    }

    pub fn new(parent: Rc<Scope>) -> Self {
        Self {
            parent: Some(parent),
            bindings: HashMap::new(),
        }
    }

    // #todo do the impl Intos slow down?
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<Expr>) -> Option<Rc<Expr>> {
        self.bindings.insert(name.into(), Rc::new(value.into()))
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<Rc<Expr>> {
        let value = self.bindings.get(name.as_ref());

        if let Some(value) = value {
            Some(value.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.get(name)
            } else {
                None
            }
        }
    }
}
