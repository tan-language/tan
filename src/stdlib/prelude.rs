use std::{rc::Rc, sync::Arc};

use crate::expr::annotate_type;
use crate::{context::Context, expr::Expr, module::Module};

// #todo remove granular imports
use super::arithmetic;
use super::dict::{dict_get_or, dict_insert};
use super::io::{read_string, write, writeln};
use super::seq::{array_count, array_push};
use super::string::{string_ends_with, string_replace, string_split};
use super::{
    eq::{eq, gt, lt},
    seq::array_join,
    string::{char_uppercased, format, string_chars, string_constructor_from_chars},
};

// #todo instead of evaluating in prelude maybe it's better to use the functions from the actual modules?
pub fn setup_std_prelude(context: &mut Context) {
    let module = Module::new("prelude", context.top_scope.clone());

    let scope = &module.scope;

    // num

    // #todo forget the mangling, implement with a dispatcher function, multi-function.
    scope.insert(
        "+",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_int)), "Int"),
    );
    scope.insert(
        "+$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_int)), "Int"),
    );
    scope.insert(
        "+$$Float$$Float",
        // #todo add the proper type: (Func Float Float Float)
        // #todo even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_float)), "Float"),
    );
    #[cfg(feature = "dec")]
    scope.insert(
        "+$$Dec$$Dec",
        // #todo add the proper type: (Func Dec Dec Dec)
        // #todo even better: (Func (Many Dec) Dec)
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_dec)), "Dec"),
    );
    scope.insert("-", Expr::ForeignFunc(Arc::new(arithmetic::sub_int)));
    scope.insert(
        "-$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::sub_int)), "Int"),
    );
    scope.insert(
        "-$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::sub_float)), "Float"),
    );
    scope.insert("*", Expr::ForeignFunc(Arc::new(arithmetic::mul_int)));
    scope.insert(
        "*$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::mul_int)), "Int"),
    );
    scope.insert(
        "*$$Float$$Float",
        // #todo add the proper type: (Func Float Float Float)
        // #todo even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::mul_float)), "Float"),
    );
    scope.insert(
        "/",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::div_float)), "Float"),
    );
    // #todo ultra-hack
    scope.insert(
        "/$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::div_float)), "Float"),
    );
    // #todo ultra-hack
    scope.insert(
        "/$$Float$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::div_float)), "Float"),
    );
    scope.insert(
        "sin",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::sin_float)), "Float"),
    );
    scope.insert(
        "cos",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::cos_float)), "Float"),
    );
    scope.insert(
        "**",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::powi_float)), "Float"),
    );

    // eq

    scope.insert("=", Expr::ForeignFunc(Arc::new(eq)));
    scope.insert(">", Expr::ForeignFunc(Arc::new(gt)));
    scope.insert("<", Expr::ForeignFunc(Arc::new(lt)));

    // io

    // #todo grab those from /std/io module

    scope.insert("read", Expr::ForeignFunc(Arc::new(read_string)));
    scope.insert("read$$String", Expr::ForeignFunc(Arc::new(read_string)));

    scope.insert("write", Expr::ForeignFunc(Arc::new(write)));
    scope.insert("write$$String", Expr::ForeignFunc(Arc::new(write)));

    scope.insert("writeln", Expr::ForeignFunc(Arc::new(writeln)));
    scope.insert("writeln$$String", Expr::ForeignFunc(Arc::new(writeln)));

    // seq

    // #todo add type qualifiers!
    scope.insert("push", Expr::ForeignFunc(Arc::new(array_push)));
    scope.insert("join", Expr::ForeignFunc(Arc::new(array_join)));
    scope.insert("count", Expr::ForeignFunc(Arc::new(array_count)));

    // dict

    // #todo add type qualifiers!
    scope.insert("insert", Expr::ForeignFunc(Arc::new(dict_insert)));
    scope.insert("get-or", Expr::ForeignFunc(Arc::new(dict_get_or)));

    // string

    // #todo define string functions in string.rs

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

    scope.insert("split", Expr::ForeignFunc(Arc::new(string_split)));

    scope.insert("replace", Expr::ForeignFunc(Arc::new(string_replace)));

    /*
    (if (ends-with filename ".png")
    (if (ends-with? filename ".png")
        (handle-image filename)
        (handle filename)
    )
     */
    // #todo: consider 'ends-with' without '?'.
    scope.insert("ends-with?", Expr::ForeignFunc(Arc::new(string_ends_with)));

    // ...

    // #todo this is a hack.
    let module_path = format!("{}/std/prelude", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}
