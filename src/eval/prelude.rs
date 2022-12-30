use std::rc::Rc;

use crate::{
    expr::Expr,
    ops::{io::write, arithmetic::add},
};

use super::env::Env;

// #TODO make Env::top() -> in fact it's bottom (of the stack)

pub fn setup_prelude(env: Env) -> Env {
    let mut env = env;

    // num

    env.insert("+", Expr::ForeignFunc(Rc::new(add)));

    // io

    env.insert("write", Expr::ForeignFunc(Rc::new(write)));

    env
}
