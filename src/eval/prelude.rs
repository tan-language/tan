use std::sync::Arc;

// #TODO implement as prelude module! put it as parent scope for modules! add unit-test.

use crate::{
    expr::{annotate_type, Expr},
    ops::{
        arithmetic::{
            add_float, add_int, cos_float, div_float, mul_float, mul_int, powi_float, sin_float,
            sub_float, sub_int,
        },
        eq::{eq, gt, lt},
        seq::array_join,
        string::{char_uppercased, format, string_chars, string_constructor_from_chars},
    },
    scope::Scope,
    stdlib::io::{write, writeln},
};

// #TODO use typeclasses (== traits) for overloading
// #TODO make Env::top() -> in fact it's bottom (of the stack)
// #TODO alternative Env::prelude()
// #TODO this part needs _complete_ rewrite.
// #TODO better syntax for type annotations needed.
// #TODO organize in modules.

pub fn setup_prelude(scope: Scope) -> Scope {
    let scope = scope;

    // num

    // #TODO forget the mangling, implement with a dispatcher function, multi-function.
    scope.insert(
        "+",
        annotate_type(Expr::ForeignFunc(Arc::new(add_int)), "Int"),
    );
    scope.insert(
        "+$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(add_int)), "Int"),
    );
    scope.insert(
        "+$$Float$$Float",
        // #TODO add the proper type: (Func Float Float Float)
        // #TODO even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(add_float)), "Float"),
    );
    scope.insert("-", Expr::ForeignFunc(Arc::new(sub_int)));
    scope.insert(
        "-$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(sub_int)), "Int"),
    );
    scope.insert(
        "-$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(sub_float)), "Float"),
    );
    scope.insert("*", Expr::ForeignFunc(Arc::new(mul_int)));
    scope.insert(
        "*$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(mul_int)), "Int"),
    );
    scope.insert(
        "*$$Float$$Float",
        // #TODO add the proper type: (Func Float Float Float)
        // #TODO even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(mul_float)), "Float"),
    );
    scope.insert(
        "/",
        annotate_type(Expr::ForeignFunc(Arc::new(div_float)), "Float"),
    );
    // #TODO ultra-hack
    scope.insert(
        "/$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(div_float)), "Float"),
    );
    // #TODO ultra-hack
    scope.insert(
        "/$$Float$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(div_float)), "Float"),
    );
    scope.insert(
        "sin",
        annotate_type(Expr::ForeignFunc(Arc::new(sin_float)), "Float"),
    );
    scope.insert(
        "cos",
        annotate_type(Expr::ForeignFunc(Arc::new(cos_float)), "Float"),
    );
    scope.insert(
        "**",
        annotate_type(Expr::ForeignFunc(Arc::new(powi_float)), "Float"),
    );

    // eq

    scope.insert("=", Expr::ForeignFunc(Arc::new(eq)));
    scope.insert(">", Expr::ForeignFunc(Arc::new(gt)));
    scope.insert("<", Expr::ForeignFunc(Arc::new(lt)));

    // io

    // #todo grab those from /std/io module

    scope.insert("write", Expr::ForeignFunc(Arc::new(write)));
    scope.insert("write$$String", Expr::ForeignFunc(Arc::new(write)));

    scope.insert("writeln", Expr::ForeignFunc(Arc::new(writeln)));
    scope.insert("writeln$$String", Expr::ForeignFunc(Arc::new(writeln)));

    // seq

    scope.insert("join", Expr::ForeignFunc(Arc::new(array_join)));

    // string

    scope.insert(
        "String",
        Expr::ForeignFunc(Arc::new(string_constructor_from_chars)),
    );
    // env.insert("String$$Array", Expr::ForeignFunc(Arc::new(string_constructor_from_chars)));

    scope.insert("chars", Expr::ForeignFunc(Arc::new(string_chars)));
    scope.insert("chars$$String", Expr::ForeignFunc(Arc::new(string_chars)));

    scope.insert("uppercased", Expr::ForeignFunc(Arc::new(char_uppercased)));
    scope.insert(
        "uppercases$$Char",
        Expr::ForeignFunc(Arc::new(char_uppercased)),
    );

    scope.insert("format", Expr::ForeignFunc(Arc::new(format)));

    scope
}
