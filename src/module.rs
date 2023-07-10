use std::{collections::HashMap, sync::OnceLock};

use crate::expr::Expr;

pub static MODULES: OnceLock<HashMap<String, Expr>> = OnceLock::new();

// #idea ModuleRegistry
// #idea ModuleLoader
