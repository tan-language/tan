use std::sync::Arc;

use crate::{expr::Expr, scope::Scope};

// #idea ModuleLoader
// #idea consider hashing to detect the same modules!

// #todo keep and define `path`, `name`, `prefix`.
// #todo keep path/url, compute stem/name.

#[derive(Debug, Clone)]
pub struct Module {
    pub stem: String,
    pub scope: Arc<Scope>,
}

// #todo impl Default

impl Default for Module {
    fn default() -> Self {
        Self {
            stem: "default".to_string(),
            scope: Arc::new(Scope::default()),
        }
    }
}

/// A module defines an isolated scope and an associated namespace.
impl Module {
    pub fn new(stem: impl Into<String>, parent_scope: Arc<Scope>) -> Self {
        Self {
            stem: stem.into(),
            scope: Arc::new(Scope::new(parent_scope)),
        }
    }

    pub fn insert(
        &self,
        name: impl Into<String>,
        value: impl Into<Arc<Expr>>,
    ) -> Option<Arc<Expr>> {
        self.scope.insert(name, value)
    }
}

#[cfg(test)]
mod tests {
    use super::Module;

    #[test]
    fn new_modules_dont_duplicate_prelude() {
        let module = Module::default();
        assert_eq!(module.scope.bindings.borrow().len(), 0);
    }
}
