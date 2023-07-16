// #insight one instance of context per thread/process of execution.

use std::{collections::HashMap, rc::Rc};

use crate::{expr::Expr, module::Module, scope::Scope, stdlib::setup_std};

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
    pub specials: HashMap<String, Rc<Expr>>, // not used yet
    pub scope: Rc<Scope>,
    pub top_scope: Rc<Scope>, // #todo find better name, e.g. prelude_scope?
}

impl Context {
    pub fn new() -> Self {
        // #todo move this somewhere else.
        // #todo how to handle missing TAN_ROOT variable?
        // #todo expose as special tan variable? at least in 'dev' profile?
        let root_path = std::env::var(ROOT_PATH_ENV_VAR).unwrap();

        let mut context = Self {
            root_path,
            module_registry: HashMap::new(),
            specials: HashMap::new(),
            scope: Rc::new(Scope::default()),
            top_scope: Rc::new(Scope::default()),
        };

        // #todo should setup_std externally!
        // #todo refactor the remaining!

        setup_std(&mut context);

        // Setup prelude as the ultimate scope.

        let prelude_path = format!("{}/std/prelude", context.root_path);
        let prelude = context.module_registry.get(&prelude_path).unwrap();

        // #todo reuse `use` code here or extract helper!
        let bindings = prelude.scope.bindings.borrow().clone();
        for (name, value) in bindings {
            context.top_scope.insert(name, value.clone());
        }

        // #todo nasty, temp hack, makes older api functions work, CLEANUP!

        context.scope = Rc::new(Scope::new(context.top_scope.clone()));

        context
    }
}
