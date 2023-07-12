use std::rc::Rc;

use crate::scope::Scope;

// #idea ModuleLoader

#[derive(Debug, Clone)]
pub struct Module {
    pub scope: Rc<Scope>,
}

// #insight
// A module defines an isolated scope.

/// A module is an isolated scope and defines a namespace.
impl Module {
    pub fn new() -> Self {
        Self {
            scope: Rc::new(Scope::prelude()),
        }
    }
}
