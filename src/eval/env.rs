use std::collections::HashMap;

use crate::{ann::Annotated, expr::Expr};

// #TODO find another name than `Scope`?
pub type Scope = HashMap<String, Annotated<Expr>>;

// #Insight
// It's better to model with a stack instead of pointers to outer environment.
// A stack better describes the actual construct, is easier to reason about (no sea-of-objects)
// is borrow-checker friendly (no lifecycles), and is more efficient on contemporary hardware.

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
        value: impl Into<Annotated<Expr>>,
    ) -> Option<Annotated<Expr>> {
        let last = self.scopes.len() - 1;
        let scope = &mut self.scopes[last];
        scope.insert(name.into(), value.into())
    }

    // #TODO extract the stack walking?

    pub fn get(&self, name: &str) -> Option<&Annotated<Expr>> {
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

#[cfg(test)]
mod tests {
    use crate::{ann::Annotated, eval::env::Env, expr::Expr};

    #[test]
    fn env_binds_names_to_values() {
        let mut env = Env::default();

        env.insert("a", Expr::Symbol("hello".to_string()));

        // let expr: &Expr = env.get("a").unwrap().as_ref();
        // dbg!(&expr);

        assert!(matches!(env.get("a"), Some(Annotated(Expr::Symbol(sym), ..)) if sym == "hello"));
        assert!(matches!(env.get("b"), None));
    }

    #[test]
    fn env_bindings_can_be_updated() {
        let mut env = Env::default();

        env.insert("a", Expr::Symbol("hello".to_string()));
        assert!(matches!(env.get("a"), Some(Annotated(Expr::Symbol(sym), ..)) if sym == "hello"));

        env.update("a", Expr::Symbol("world".to_string()));
        assert!(matches!(env.get("a"), Some(Annotated(Expr::Symbol(sym), ..)) if sym == "world"));
    }
}
