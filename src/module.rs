use std::rc::Rc;

use crate::scope::Scope;

// #idea ModuleLoader
// #idea consider hashing to detect the same modules!

// #todo keep and define `path`, `name`, `prefix`.

#[derive(Debug, Clone)]
pub struct Module {
    pub stem: String,
    pub scope: Rc<Scope>,
}

// #todo impl Default

/// A module defines an isolated scope and an associated namespace.
impl Module {
    pub fn new(stem: impl Into<String>) -> Self {
        Self {
            stem: stem.into(),
            scope: Rc::new(Scope::prelude()),
        }
    }
}
