use std::rc::Rc;

use crate::{
    ann::ANNO,
    expr::Expr,
    ops::{
        arithmetic::{add_float, add_int, mul, sub},
        eq::{eq, gt, lt},
        io::{file_read_as_string, file_write_string, write, writeln},
        process::exit,
        string::{char_uppercased, string_chars, string_constructor_from_chars},
    },
};

use super::env::Env;

// #TODO use typeclasses (== traits) for overloading
// #TODO make Env::top() -> in fact it's bottom (of the stack)
// #TODO alternative Env::prelude()
// #TODO this part needs _complete_ rewrite.
// #TODO better syntax for type annotations needed.

pub fn setup_prelude(env: Env) -> Env {
    let mut env = env;

    // num

    // #TODO forget the mangling, implement with a dispatcher function, multi-function.
    env.insert(
        "+",
        ANNO::with_type(Expr::ForeignFunc(Rc::new(add_int)), Expr::symbol("Int")),
    );
    env.insert(
        "+$$Int$$Int",
        ANNO::with_type(Expr::ForeignFunc(Rc::new(add_int)), Expr::symbol("Int")),
    );
    env.insert(
        "+$$Float$$Float",
        // #TODO add the proper type: (Func Float Float Float)
        // #TODO even better: (Func (Many Float) Float)
        ANNO::with_type(Expr::ForeignFunc(Rc::new(add_float)), Expr::symbol("Float")),
    );
    env.insert("-", Expr::ForeignFunc(Rc::new(sub)));
    env.insert("*", Expr::ForeignFunc(Rc::new(mul)));

    // eq

    env.insert("=", Expr::ForeignFunc(Rc::new(eq)));
    env.insert(">", Expr::ForeignFunc(Rc::new(gt)));
    env.insert("<", Expr::ForeignFunc(Rc::new(lt)));

    // io

    env.insert("write", Expr::ForeignFunc(Rc::new(write)));
    env.insert("write$$String", Expr::ForeignFunc(Rc::new(write)));

    env.insert("writeln", Expr::ForeignFunc(Rc::new(writeln)));
    env.insert("writeln$$String", Expr::ForeignFunc(Rc::new(writeln)));

    env.insert(
        "File:read-string",
        Expr::ForeignFunc(Rc::new(file_read_as_string)),
    );
    env.insert(
        "File:read-string$$String",
        Expr::ForeignFunc(Rc::new(file_read_as_string)),
    );

    // #TODO consider just `write`.
    env.insert(
        // #TODO alternatives: "std:fs:write_string", "std:url:write_string", "str.url.write-string"
        "File:write-string",
        Expr::ForeignFunc(Rc::new(file_write_string)),
    );
    env.insert(
        "File:write-string$$String",
        Expr::ForeignFunc(Rc::new(file_write_string)),
    );

    // process

    env.insert("exit", Expr::ForeignFunc(Rc::new(exit)));
    env.insert("exit$$", Expr::ForeignFunc(Rc::new(exit)));

    // string

    env.insert(
        "String",
        Expr::ForeignFunc(Rc::new(string_constructor_from_chars)),
    );
    // env.insert("String$$Array", Expr::ForeignFunc(Rc::new(string_constructor_from_chars)));

    env.insert("chars", Expr::ForeignFunc(Rc::new(string_chars)));
    env.insert("chars$$String", Expr::ForeignFunc(Rc::new(string_chars)));

    env.insert("uppercased", Expr::ForeignFunc(Rc::new(char_uppercased)));
    env.insert(
        "uppercases$$Char",
        Expr::ForeignFunc(Rc::new(char_uppercased)),
    );

    env
}
