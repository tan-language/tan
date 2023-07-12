use std::rc::Rc;

use crate::scope::Scope;

// #idea ModuleLoader
// #idea consider hashing to detect the same modules!

#[derive(Debug, Clone)]
pub struct Module {
    pub scope: Rc<Scope>,
}

/// A module defines an isolated scope and an associated namespace.
impl Module {
    pub fn new() -> Self {
        Self {
            scope: Rc::new(Scope::prelude()),
        }
    }
}
