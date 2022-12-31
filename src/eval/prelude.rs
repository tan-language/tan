use std::rc::Rc;

use crate::{
    expr::Expr,
    ops::{
        arithmetic::{add, mul, sub},
        eq::{eq, gt, lt},
        io::{write, writeln},
    },
};

use super::env::Env;

// #TODO make Env::top() -> in fact it's bottom (of the stack)

pub fn setup_prelude(env: Env) -> Env {
    let mut env = env;

    // num

    env.insert("+", Expr::ForeignFunc(Rc::new(add)));
    env.insert("-", Expr::ForeignFunc(Rc::new(sub)));
    env.insert("*", Expr::ForeignFunc(Rc::new(mul)));

    // eq

    env.insert("=", Expr::ForeignFunc(Rc::new(eq)));
    env.insert(">", Expr::ForeignFunc(Rc::new(gt)));
    env.insert("<", Expr::ForeignFunc(Rc::new(lt)));

    // io

    env.insert("write", Expr::ForeignFunc(Rc::new(write)));
    env.insert("writeln", Expr::ForeignFunc(Rc::new(writeln)));

    env
}
