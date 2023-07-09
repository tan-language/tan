use std::sync::Arc;

use crate::{
    expr::{annotate_type, Expr},
    ops::{
        arithmetic::{
            add_float, add_int, cos_float, div_float, mul_float, mul_int, powi_float, sin_float,
            sub_float, sub_int,
        },
        eq::{eq, gt, lt},
        io::{file_read_as_string, file_write_string, write, writeln},
        process::exit,
        seq::array_join,
        string::{char_uppercased, format, string_chars, string_constructor_from_chars},
    },
};

use super::env::Env;

// #TODO use typeclasses (== traits) for overloading
// #TODO make Env::top() -> in fact it's bottom (of the stack)
// #TODO alternative Env::prelude()
// #TODO this part needs _complete_ rewrite.
// #TODO better syntax for type annotations needed.
// #TODO organize in modules.

pub fn setup_prelude(env: Env) -> Env {
    let mut env = env;

    // num

    // #TODO forget the mangling, implement with a dispatcher function, multi-function.
    env.insert(
        "+",
        annotate_type(Expr::ForeignFunc(Arc::new(add_int)), "Int"),
    );
    env.insert(
        "+$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(add_int)), "Int"),
    );
    env.insert(
        "+$$Float$$Float",
        // #TODO add the proper type: (Func Float Float Float)
        // #TODO even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(add_float)), "Float"),
    );
    env.insert("-", Expr::ForeignFunc(Arc::new(sub_int)));
    env.insert(
        "-$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(sub_int)), "Int"),
    );
    env.insert(
        "-$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(sub_float)), "Float"),
    );
    env.insert("*", Expr::ForeignFunc(Arc::new(mul_int)));
    env.insert(
        "*$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(mul_int)), "Int"),
    );
    env.insert(
        "*$$Float$$Float",
        // #TODO add the proper type: (Func Float Float Float)
        // #TODO even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(mul_float)), "Float"),
    );
    env.insert(
        "/",
        annotate_type(Expr::ForeignFunc(Arc::new(div_float)), "Float"),
    );
    // #TODO ultra-hack
    env.insert(
        "/$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(div_float)), "Float"),
    );
    // #TODO ultra-hack
    env.insert(
        "/$$Float$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(div_float)), "Float"),
    );
    env.insert(
        "sin",
        annotate_type(Expr::ForeignFunc(Arc::new(sin_float)), "Float"),
    );
    env.insert(
        "cos",
        annotate_type(Expr::ForeignFunc(Arc::new(cos_float)), "Float"),
    );
    env.insert(
        "**",
        annotate_type(Expr::ForeignFunc(Arc::new(powi_float)), "Float"),
    );

    // eq

    env.insert("=", Expr::ForeignFunc(Arc::new(eq)));
    env.insert(">", Expr::ForeignFunc(Arc::new(gt)));
    env.insert("<", Expr::ForeignFunc(Arc::new(lt)));

    // io

    env.insert("write", Expr::ForeignFunc(Arc::new(write)));
    env.insert("write$$String", Expr::ForeignFunc(Arc::new(write)));

    env.insert("writeln", Expr::ForeignFunc(Arc::new(writeln)));
    env.insert("writeln$$String", Expr::ForeignFunc(Arc::new(writeln)));

    env.insert(
        "File:read-string",
        Expr::ForeignFunc(Arc::new(file_read_as_string)),
    );
    env.insert(
        "File:read-string$$String",
        Expr::ForeignFunc(Arc::new(file_read_as_string)),
    );

    // #TODO consider just `write`.
    env.insert(
        // #TODO alternatives: "std:fs:write_string", "std:url:write_string", "str.url.write-string"
        "File:write-string",
        Expr::ForeignFunc(Arc::new(file_write_string)),
    );
    env.insert(
        "File:write-string$$String",
        Expr::ForeignFunc(Arc::new(file_write_string)),
    );

    // process

    env.insert("exit", Expr::ForeignFunc(Arc::new(exit)));
    env.insert("exit$$", Expr::ForeignFunc(Arc::new(exit)));

    // seq

    env.insert("join", Expr::ForeignFunc(Arc::new(array_join)));

    // string

    env.insert(
        "String",
        Expr::ForeignFunc(Arc::new(string_constructor_from_chars)),
    );
    // env.insert("String$$Array", Expr::ForeignFunc(Arc::new(string_constructor_from_chars)));

    env.insert("chars", Expr::ForeignFunc(Arc::new(string_chars)));
    env.insert("chars$$String", Expr::ForeignFunc(Arc::new(string_chars)));

    env.insert("uppercased", Expr::ForeignFunc(Arc::new(char_uppercased)));
    env.insert(
        "uppercases$$Char",
        Expr::ForeignFunc(Arc::new(char_uppercased)),
    );

    env.insert("format", Expr::ForeignFunc(Arc::new(format)));

    env
}
