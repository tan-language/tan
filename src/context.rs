// #insight one instance of context per thread/process of execution.

use std::{collections::HashMap, rc::Rc};

use crate::{module::Module, scope::Scope};

// #insight Context is the instance of a Tan 'machine'.

// #todo keep `specials` or `special-vars`, e.g. *current-module*.

// #todo consider process/thread context?
/// An execution context
pub struct Context {
    // #todo what else should we add here?
    // #todo consider the name `module_map`
    pub module_registry: HashMap<String, Module>,
    pub scope: Rc<Scope>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            module_registry: HashMap::new(),
            scope: Rc::new(Scope::prelude()),
        }
    }
}
