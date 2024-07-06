// #insight one instance of context per thread/process of execution.

use std::{collections::HashMap, sync::Arc};

use crate::{
    eval::util::canonicalize_path, expr::Expr, library::setup_lib, module::Module, scope::Scope,
    util::standard_names::PROFILE,
};

// #insight Context is the instance of a Tan 'machine'.

// #todo Context should provide access both to the compiler and the evaluator.
// #todo should provide also (compile ...) and (eval ...) functions.
// #todo (eval ...) should reuse (compile ...)

// #todo consider renaming to Env again?
// #todo rethink what Context is
// #todo initialize *current-module-path*? e.g. to '.'
// #todo in the past we had a ...special `specials` map for system variables, is this useful?

const ROOT_PATH_ENV_VAR: &str = "TAN_ROOT";

// #insight the Clone is used for the http-server
// #todo consider removing the Clone, it will give more flexibility.

// #todo consider process/thread context?
// #todo then we could remove Arcs and simplify our life?
/// An execution context
#[derive(Clone, Debug)]
pub struct Context {
    // #todo what else should we add here?
    pub root_path: String,
    // #todo consider the name `module_map`
    pub module_registry: HashMap<String, Arc<Module>>,
    // #insight named just scope instead of static_scope, to match module.scope.
    /// The static scope.
    pub scope: Arc<Scope>,
    /// The dynamic scope.
    pub dynamic_scope: Arc<Scope>,
    // #todo find better name, e.g. prelude_scope?
    // #todo what about `global_scope`? nah...
    pub top_scope: Arc<Scope>,
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

        let root_path = canonicalize_path(root_path);

        let top_scope = Arc::new(Scope::default());

        let mut context = Self {
            root_path,
            module_registry: HashMap::new(),
            scope: top_scope.clone(),
            dynamic_scope: Arc::new(Scope::default()),
            top_scope: top_scope.clone(),
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
            .expect("prelude should be defined")
            .clone();

        // #todo reuse `use` code here or extract helper!
        let bindings = prelude.scope.bindings.read().expect("poisoned lock");
        for (name, value) in bindings.iter() {
            top_scope.insert(name, value.clone());
        }

        // #todo nasty, temp hack, makes older api functions work, CLEANUP!

        // #todo we need scope-stack visualization.
        // #todo do we really need this intermediate scope? for some reason this is needed! investigate why!
        context.scope = Arc::new(Scope::new(top_scope.clone()));

        context
    }

    // #todo get_current_module

    // #todo require_module
    pub fn get_module(&self, path: &str) -> Option<&Arc<Module>> {
        // #todo this is a hack.
        // #todo extract as function.
        let url = format!("{}/@std/{}", self.root_path, path);

        // #todo rethink about this canonicalization.
        let url = canonicalize_path(url);

        self.module_registry.get(&url)
    }

    pub fn get_module_mut(&mut self, path: &str) -> Option<&mut Arc<Module>> {
        // #todo this is a hack.
        // #todo extract as function.
        let url = format!("{}/@std/{}", self.root_path, path);

        // #todo rethink about this canonicalization.
        let url = canonicalize_path(url);

        self.module_registry.get_mut(&url)
    }

    // #todo do the `impl Into`s slow down?
    #[inline]
    pub fn insert(
        &self,
        name: impl Into<String>,
        value: impl Into<Arc<Expr>>,
        is_dynamically_scoped: bool,
    ) -> Option<Arc<Expr>> {
        if is_dynamically_scoped {
            self.dynamic_scope.insert(name, value)
        } else {
            self.scope.insert(name, value)
        }
    }

    #[inline]
    pub fn get(&self, name: impl AsRef<str>, is_dynamically_scoped: bool) -> Option<Arc<Expr>> {
        let name = name.as_ref();
        if is_dynamically_scoped {
            self.dynamic_scope.get(name)
        } else {
            self.scope.get(name)
        }
    }

    // #todo have a get_profile methods that returns some kind of enum?

    pub fn is_test_profile(&self) -> bool {
        if let Some(profile) = self.top_scope.get(PROFILE) {
            if let Some(profile) = profile.as_string() {
                // #todo nasty, do something proper here.
                return profile == "test";
            }
        }
        false
    }

    pub fn contains_name(&self, name: impl AsRef<str>) -> bool {
        self.scope.contains_name_recursive(name.as_ref())
    }
}
