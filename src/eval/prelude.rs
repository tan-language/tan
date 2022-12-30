use std::rc::Rc;

use crate::{expr::Expr, ops::num::add};

use super::env::Env;

// #TODO make Env::top() -> in fact it's bottom (of the stack)

pub fn setup_prelude(env: Env) -> Env {
    let mut env = env;

    env.insert("+", Expr::ForeignFunc(Rc::new(add)));

    env
}
