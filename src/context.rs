// #insight one instance of context per thread/process of execution.

use std::collections::HashMap;

use crate::{module::Module, scope::Scope};

// #todo consider process/thread context?
/// An execution context
pub struct Context {
    pub module_registry: HashMap<String, Module>,
    pub env: Scope,
}
