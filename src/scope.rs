use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::expr::Expr;

// #todo should we name this `Env`?
// #todo consider removing `Into`s and `AsRef`s
// #todo extract the stack walking?
// #todo no global, abuse a module for 'global', like CommonJS.

// #insight
// To implement lexical scoping we need multiple shared references to
// scopes, in general to implement a dynamic language we need some kind of automated
// memory management, hence the `Rc`s.

// #insight
// A stack of owned scopes cannot be used like in a previous implementation, as
// multiple other functions can refer to (close-over) upstream (or downstream) scopes.

// #insight
// binding = name -> value
// annotation = name -> meta(-value)

// #insight annotations are defined _statically_, they are static

// #insight only named (bound) values can be annotated, annotations to literals are resolved/handled statically

// #href https://stackoverflow.com/questions/12599965/lexical-environment-and-function-scope

// #think
// context -> dynamic
// scope/environment -> static? what about closure's scope? could merge scope + context?

#[derive(Debug, Default)]
pub struct Scope {
    // #todo add global/session ?
    // #todo support read-only bindings?
    pub parent: Option<Rc<Scope>>,
    pub bindings: RefCell<HashMap<String, Rc<Expr>>>,
    // #idea have separate values/annotations!!!
    // #idea annotate only named expressions/bindings, don't annotate literals! to make the above work.
}

impl Scope {
    // #todo consider renaming to child_of?
    pub fn new(parent: Rc<Scope>) -> Self {
        Self {
            parent: Some(parent),
            bindings: RefCell::new(HashMap::new()),
        }
    }

    // #todo do the `impl Into`s slow down?
    pub fn insert(&self, name: impl Into<String>, value: impl Into<Rc<Expr>>) -> Option<Rc<Expr>> {
        self.bindings.borrow_mut().insert(name.into(), value.into())
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<Rc<Expr>> {
        let bindings = self.bindings.borrow();

        let value = bindings.get(name.as_ref());

        if let Some(value) = value {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    // #todo only allow updating mutable bindings
    // #todo should we even allow this?
    /// Updates an existing binding, walks the environment.
    pub fn update(&self, name: impl AsRef<str>, value: impl Into<Expr>) {
        let mut bindings = self.bindings.borrow_mut();

        let binding = bindings.get_mut(name.as_ref());

        if let Some(binding) = binding {
            *binding = Rc::new(value.into());
        } else if let Some(parent) = &self.parent {
            parent.update(name, value);
        } else {
            // #todo should report an error here!
        }
    }
}
