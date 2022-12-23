use super::scope::Scope;

// #Insight
// It's better to model with a stack instead of pointers to outer environment.
// A stack better describes the actual construct, is easier to reason about (no sea-of-objects)
// is borrow-checker friendly (no lifecycles), and is more efficient on contemporary hardware.

// #TODO find another name than `Scope`?

/// An execution/evaluation environments.
/// The environment is a stack of scopes.
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

    // #TODO hmm..
    pub fn push_scope(&mut self) {
        self.push(Scope::default());
    }

    pub fn pop(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }
}

#[cfg(test)]
mod tests {
    // #TODO write unit tests!
}
