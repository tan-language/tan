use std::collections::HashMap;

use crate::{ann::Ann, expr::Expr};

use super::prelude::setup_prelude;

// #TODO find another name than `Scope`?
pub type Scope = HashMap<String, Ann<Expr>>;

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
    pub scopes: Vec<Scope>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }

    pub fn prelude() -> Self {
        setup_prelude(Env::default())
    }

    pub fn push(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    pub fn push_new_scope(&mut self) {
        self.push(Scope::default());
    }

    pub fn pop(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    // #TODO better offer get/set interface?

    pub fn insert(
        &mut self,
        name: impl Into<String>,
        value: impl Into<Ann<Expr>>,
    ) -> Option<Ann<Expr>> {
        let last = self.scopes.len() - 1;
        let scope = &mut self.scopes[last];
        scope.insert(name.into(), value.into())
    }

    // #TODO extract the stack walking?

    pub fn get(&self, name: &str) -> Option<&Ann<Expr>> {
        let nesting = self.scopes.len();

        // #TODO optimize here!

        for i in (0..nesting).rev() {
            let scope = &self.scopes[i];
            if let Some(binding) = scope.get(name) {
                return Some(binding);
            }
        }

        None
    }

    /// Updates an existing binding, walks the environment.
    pub fn update(&mut self, name: &str, value: Expr) {
        let nesting = self.scopes.len();

        // #TODO optimize here!
        // #TODO what to return?

        for i in (0..nesting).rev() {
            let scope = &mut self.scopes[i];
            if let Some(binding) = scope.get_mut(name) {
                binding.0 = value;
                break;
            }
        }
    }
}
