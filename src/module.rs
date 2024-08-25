use std::sync::Arc;

use crate::{
    expr::{Expr, FnNoContext, ForeignFnRef},
    scope::Scope,
};

// #idea ModuleLoader
// #idea Consider hashing to detect the same modules!

// #todo Keep and define `path`, `name`, `prefix`.
// #todo Keep path/url, compute stem/name.

#[derive(Debug, Clone)]
pub struct Module {
    pub stem: String,
    pub scope: Arc<Scope>,
}

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

    // #todo Think about the visibility.
    pub fn insert_foreign_func_no_context(
        &self,
        name: impl Into<String>,
        func: &'static FnNoContext,
    ) -> Option<Arc<Expr>> {
        self.insert(name, Expr::ForeignFunc(ForeignFnRef::NoContext(func)))
    }
}

#[cfg(test)]
mod tests {
    use super::Module;

    #[test]
    fn new_modules_do_not_duplicate_prelude() {
        let module = Module::default();
        assert_eq!(module.scope.bindings.read().unwrap().len(), 0);
    }
}
