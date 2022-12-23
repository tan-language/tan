use std::collections::HashMap;

use crate::{ann::Annotated, expr::Expr};

// #TODO find another name than `Scope`?
pub type Scope = HashMap<String, Annotated<Expr>>;

// #Insight
// It's better to model with a stack instead of pointers to outer environment.
// A stack better describes the actual construct, is easier to reason about (no sea-of-objects)
// is borrow-checker friendly (no lifecycles), and is more efficient on contemporary hardware.

/// An execution/evaluation environments.
/// The environment is a stack of scopes.
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

    pub fn insert(
        &mut self,
        name: impl Into<String>,
        value: impl Into<Annotated<Expr>>,
    ) -> Option<Annotated<Expr>> {
        let last = self.scopes.len() - 1;
        let scope = &mut self.scopes[last];
        scope.insert(name.into(), value.into())
    }

    pub fn get(&self, name: &str) -> Option<&Annotated<Expr>> {
        let nesting = self.scopes.len();

        // TODO: optimize here!

        for i in (0..nesting).rev() {
            let scope = &self.scopes[i];
            if let Some(binding) = scope.get(name) {
                return Some(binding);
            }
        }

        None
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
}
