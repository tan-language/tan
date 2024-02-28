// #insight one instance of context per thread/process of execution.

use std::{collections::HashMap, rc::Rc};

use crate::{
    eval::util::canonicalize_path, expr::Expr, module::Module, scope::Scope, stdlib::setup_lib,
};

// #insight Context is the instance of a Tan 'machine'.

// #todo consider renaming to Env again?
// #todo keep `specials` or `special-vars`, e.g. *current-module*.
// #todo rethink what Context is
// #todo initialize *current-module-path*? e.g. to '.'

const ROOT_PATH_ENV_VAR: &str = "TAN_ROOT";

// #todo consider process/thread context?
/// An execution context
pub struct Context {
    // #todo what else should we add here?
    // #todo consider the name `module_map`
    pub root_path: String,
    pub module_registry: HashMap<String, Rc<Module>>,
    pub specials: HashMap<&'static str, Rc<Expr>>, // not used yet
    // #insight named just scope instead of static_scope, to match module.scope.
    /// The static scope.
    pub scope: Rc<Scope>,
    /// The dynamic scope.
    pub dynamic_scope: Rc<Scope>,
    // #todo find better name, e.g. prelude_scope?
    // #todo what about `global_scope`? nah...
    pub top_scope: Rc<Scope>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    // #todo consider removing new and just use default?
    pub fn new() -> Self {
        // #todo move this somewhere else.
        // #todo how to handle missing TAN_ROOT variable?
        // #todo expose as special tan variable? at least in 'dev' profile?
        let root_path = std::env::var(ROOT_PATH_ENV_VAR)
            .unwrap_or_else(|_| panic!("env variable `{ROOT_PATH_ENV_VAR}` should be set"));

        let mut context = Self {
            root_path,
            module_registry: HashMap::new(),
            specials: HashMap::new(),
            scope: Rc::new(Scope::default()),
            dynamic_scope: Rc::new(Scope::default()),
            top_scope: Rc::new(Scope::default()),
        };

        // #todo should setup_std externally!
        // #todo refactor the remaining!

        setup_lib(&mut context);

        // Setup prelude as the ultimate scope.

        // let prelude_path = format!("{}/std/prelude", context.root_path);
        // let prelude = context.module_registry.get(&prelude_path).unwrap();

        // #todo could use a non-mut version of require_module.
        let prelude = context
            .get_module("prelude")
            .expect("prelude should be defined");

        // #todo reuse `use` code here or extract helper!
        let bindings = prelude.scope.bindings.borrow().clone();
        for (name, value) in bindings {
            context.top_scope.insert(name, value.clone());
        }

        // #todo nasty, temp hack, makes older api functions work, CLEANUP!

        context.scope = Rc::new(Scope::new(context.top_scope.clone()));

        context
    }

    // #todo get_module_mut
    // #todo require_module
    pub fn get_module(&self, path: &str) -> Option<&Rc<Module>> {
        // #todo this is a hack.
        // #todo extract as function.
        let url = format!("{}/@std/{}", self.root_path, path);

        // #todo rethink about this canonicalization.
        let url = canonicalize_path(url);

        self.module_registry.get(&url)
    }

    pub fn get_module_mut(&mut self, path: &str) -> Option<&mut Rc<Module>> {
        // #todo this is a hack.
        // #todo extract as function.
        let url = format!("{}/@std/{}", self.root_path, path);

        // #todo rethink about this canonicalization.
        let url = canonicalize_path(url);

        self.module_registry.get_mut(&url)
    }

    pub fn insert_special(&mut self, key: &'static str, value: Expr) {
        self.specials.insert(key, Rc::new(value));
    }

    pub fn get_special(&self, key: &'static str) -> Option<Rc<Expr>> {
        self.specials.get(key).cloned()
    }

    // #todo do the `impl Into`s slow down?
    pub fn insert(&self, name: impl Into<String>, value: impl Into<Rc<Expr>>) -> Option<Rc<Expr>> {
        // #todo add support for dynamic scoping.
        self.scope.insert(name, value)
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<Rc<Expr>> {
        // #todo add support for dynamic scoping.
        self.scope.get(name)
    }

    // #todo only allow updating mutable bindings
    // #todo should we even allow this?
    /// Updates an existing binding, walks the environment.
    pub fn update(&self, name: impl AsRef<str>, value: impl Into<Expr>) {
        // #todo no update support for dynamic scoping.
        self.scope.update(name, value)
    }
}
