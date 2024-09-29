use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

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
    pub parent: Option<Arc<Scope>>,
    // #todo explain why we have RefCell here.
    // #todo do we need RwLock here?
    pub bindings: RwLock<HashMap<String, Arc<Expr>>>,
    // #idea have separate values/annotations!!!
    // #idea annotate only named expressions/bindings, don't annotate literals! to make the above work.
}

impl Scope {
    // #todo consider renaming to child_of?
    pub fn new(parent: Arc<Scope>) -> Self {
        Self {
            parent: Some(parent),
            bindings: RwLock::new(HashMap::new()),
        }
    }

    // #todo do the `impl Into`s slow down?
    pub fn insert(
        &self,
        name: impl Into<String>,
        value: impl Into<Arc<Expr>>,
    ) -> Option<Arc<Expr>> {
        self.bindings
            .write()
            .expect("poisoned lock")
            .insert(name.into(), value.into())
    }

    // A specialized helper method that inserts invocables and also handles mangled names.
    pub fn insert_invocable(
        &self,
        name: impl Into<String>,
        value: impl Into<Arc<Expr>>,
    ) -> Option<Arc<Expr>> {
        let name = name.into();
        let value = value.into();
        // #todo Extract helper predicate function for mangled names.
        if name.contains("$$") {
            let (base_name, _) = name.split_once("$$").unwrap();
            if !self.contains_name(base_name) {
                self.insert(base_name, value.clone());
            }
        } else {
            // #insight This makes it more fault tolerant, also report an error.
            // println!("*** Non mangled invocable name `{name}`.");
            // #todo Check in "...$$*" already exists.
            self.insert(format!("{name}$$*"), value.clone());
        }
        self.insert(name, value)
    }

    // #todo We need a recursive version!
    // #todo consider `contains_symbol`
    // #todo think about name <> symbol.
    pub fn contains_name(&self, name: impl AsRef<str>) -> bool {
        self.bindings
            .read()
            .expect("poisoned lock")
            .contains_key(name.as_ref())
    }

    // #todo Have delegate in Context?
    // #todo Find a better postfix than recursive.
    pub fn contains_name_recursive(&self, name: impl AsRef<str>) -> bool {
        if self.contains_name(name.as_ref()) {
            true
        } else if let Some(parent) = &self.parent {
            parent.contains_name(name.as_ref())
        } else {
            false
        }
    }

    // #todo Add non-recursive version?
    pub fn get(&self, name: impl AsRef<str>) -> Option<Arc<Expr>> {
        let bindings = self.bindings.read().expect("poisoned lock");

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
        let mut bindings = self.bindings.write().expect("poisoned lock");

        let binding = bindings.get_mut(name.as_ref());

        if let Some(binding) = binding {
            *binding = Arc::new(value.into());
        } else if let Some(parent) = &self.parent {
            parent.update(name, value);
        } else {
            // #todo should report an error here!
        }
    }

    // #todo is this really useful?
    // #todo no need to return anything here?
    pub fn remove(&self, name: impl AsRef<str>) -> Option<Arc<Expr>> {
        let mut bindings = self.bindings.write().expect("poisoned lock");
        bindings.remove(name.as_ref())
    }
}
