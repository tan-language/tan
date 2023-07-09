use std::collections::HashMap;

use crate::expr::Expr;

use super::prelude::setup_prelude;

// #TODO separate global_scope.
// #TODO global <> local scope.
// #TODO {insert/update}_{global/local}
// #TODO support namespaces

// #todo rename Scope to Bindings, rename Env to Scope?

// #TODO find another name than `Scope`?
pub type Scope = HashMap<String, Expr>;

// #TODO support global scope + lexical/static scope + dynamic scope.

// #Insight
// It's better to model with a stack instead of pointers to outer environment.
// A stack better describes the actual construct, is easier to reason about (no sea-of-objects)
// is borrow-checker friendly (no lifecycles), and is more efficient on contemporary hardware.

// ~~Scope is static, Environment is dynamic~~ <-- nah (static/dynamic scoping)

/// An evaluation environment.
///
/// An environment is a stack of scopes.
/// A scope is a a collection of bindings.
/// A binding binds a symbol to a value/expr.
#[derive(Debug)]
pub struct Env {
    pub global: Scope, // #TODO no global, abuse a module for 'global', like CommonJS.
    pub local: Vec<Scope>,
    // #TODO maybe even keep the inner local scope as field?
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        Self {
            global: Scope::default(),
            local: vec![Scope::default()],
        }
    }

    // #TODO definitely move externally, we can have multiple preludes, even versioned prelude.
    pub fn prelude() -> Self {
        setup_prelude(Env::default())
    }

    pub fn push(&mut self, scope: Scope) {
        self.local.push(scope);
    }

    // #TODO maybe remove this?
    pub fn push_new_scope(&mut self) {
        self.push(Scope::default());
    }

    pub fn pop(&mut self) -> Option<Scope> {
        self.local.pop()
    }

    // #TODO better offer get/set interface?

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<Expr>) -> Option<Expr> {
        let last = self.local.len() - 1;
        let scope = &mut self.local[last];
        scope.insert(name.into(), value.into())
    }

    // #TODO extract the stack walking?

    pub fn get(&self, name: &str) -> Option<&Expr> {
        let nesting = self.local.len();

        // #TODO optimize here!

        for i in (0..nesting).rev() {
            let scope = &self.local[i];
            if let Some(binding) = scope.get(name) {
                return Some(binding);
            }
        }

        self.global.get(name)
    }

    /// Updates an existing binding, walks the environment.
    pub fn update(&mut self, name: &str, value: impl Into<Expr>) {
        let nesting = self.local.len();

        // #TODO optimize here!
        // #TODO what to return?

        for i in (0..nesting).rev() {
            let scope = &mut self.local[i];
            if let Some(binding) = scope.get_mut(name) {
                *binding = value.into();
                break;
            }
        }
    }
}
