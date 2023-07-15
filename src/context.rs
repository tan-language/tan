// #insight one instance of context per thread/process of execution.

use std::{collections::HashMap, rc::Rc};

use crate::{expr::Expr, module::Module, scope::Scope};

// #insight Context is the instance of a Tan 'machine'.

// #todo consider renaming to Env again?
// #todo keep `specials` or `special-vars`, e.g. *current-module*.
// #todo rethink what Context is

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
}

impl Context {
    pub fn new() -> Self {
        // #todo move this somewhere else.
        // #todo how to handle missing TAN_ROOT variable?
        // #todo expose as special tan variable? at least in 'dev' profile?
        let root_path = std::env::var(ROOT_PATH_ENV_VAR).unwrap();

        Self {
            root_path,
            module_registry: HashMap::new(),
            specials: HashMap::new(),
            scope: Rc::new(Scope::prelude()),
        }

        // #todo should setup_std here?
    }
}
