use std::rc::Rc;

use crate::{
    expr::Expr,
    ops::{
        arithmetic::{add_float, add_int, mul, sub},
        eq::{eq, gt, lt},
        io::{write, writeln},
        process::exit,
    },
};

use super::env::Env;

// #TODO make Env::top() -> in fact it's bottom (of the stack)

pub fn setup_prelude(env: Env) -> Env {
    let mut env = env;

    // #TODO temp hack
    // #TODO implement as ForeignFunc or ForeignMacro
    // #TODO alternatively could be encoded as keywords by the lexer or the parser?

    env.insert("do", Expr::symbol("do"));
    env.insert("quot", Expr::symbol("quot"));
    env.insert("for", Expr::symbol("for"));
    env.insert("let", Expr::symbol("let"));
    env.insert("Func", Expr::symbol("Func"));

    // num

    env.insert("+", Expr::ForeignFunc(Rc::new(add_int)));
    env.insert("+$$Int$$Int", Expr::ForeignFunc(Rc::new(add_int)));
    env.insert("+$$Float$$Float", Expr::ForeignFunc(Rc::new(add_float)));
    env.insert("-", Expr::ForeignFunc(Rc::new(sub)));
    env.insert("*", Expr::ForeignFunc(Rc::new(mul)));

    // eq

    env.insert("=", Expr::ForeignFunc(Rc::new(eq)));
    env.insert(">", Expr::ForeignFunc(Rc::new(gt)));
    env.insert("<", Expr::ForeignFunc(Rc::new(lt)));

    // io

    env.insert("write", Expr::ForeignFunc(Rc::new(write)));
    env.insert("write$$String", Expr::ForeignFunc(Rc::new(write)));
    env.insert("writeln", Expr::ForeignFunc(Rc::new(write)));
    env.insert("writeln$$String", Expr::ForeignFunc(Rc::new(writeln)));

    // process
    env.insert("exit", Expr::ForeignFunc(Rc::new(exit)));
    env.insert("exit$$", Expr::ForeignFunc(Rc::new(exit)));

    env
}
