use std::rc::Rc;

use crate::scope::Scope;

// pub static MODULES: OnceLock<HashMap<String, Expr>> = OnceLock::new();

// #idea ModuleRegistry
// #idea ModuleLoader

#[derive(Debug, Clone)]
pub struct Module {
    pub scope: Rc<Scope>,
}
