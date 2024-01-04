use std::{rc::Rc, sync::Arc};

use crate::expr::annotate_type;
use crate::{context::Context, expr::Expr, module::Module};

// #todo remove granular imports
use super::arithmetic;
use super::dict::{
    dict_contains_key, dict_get_keys, dict_get_or, dict_get_values, dict_insert, dict_update_mut,
};
use super::eq::{
    eq_float, eq_string, eq_symbol, not_eq, not_eq_float, not_eq_string, not_eq_symbol,
};
use super::io::{read_string, write, writeln};
use super::seq::{array_count, array_push, array_sort_mut};
use super::string::{
    string_ends_with, string_get_length, string_replace, string_slice, string_slice_range,
    string_split, string_starts_with,
};
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
    scope.insert("=$$Int$$Int", Expr::ForeignFunc(Arc::new(eq)));
    scope.insert("=$$Float$$Float", Expr::ForeignFunc(Arc::new(eq_float)));
    scope.insert("=$$String$$String", Expr::ForeignFunc(Arc::new(eq_string)));
    // scope.insert("=$$Symbol$$Symbol", Expr::ForeignFunc(Arc::new(eq_symbol)));
    scope.insert(
        "=$$KeySymbol$$KeySymbol",
        Expr::ForeignFunc(Arc::new(eq_symbol)),
    );

    scope.insert("!=", Expr::ForeignFunc(Arc::new(not_eq)));
    scope.insert("!=$$Int$$Int", Expr::ForeignFunc(Arc::new(not_eq)));
    scope.insert(
        "!=$$Float$$Float",
        Expr::ForeignFunc(Arc::new(not_eq_float)),
    );
    scope.insert(
        "!=$$String$$String",
        Expr::ForeignFunc(Arc::new(not_eq_string)),
    );
    scope.insert(
        "!=$$Symbol$$Symbol",
        Expr::ForeignFunc(Arc::new(not_eq_symbol)),
    );
    scope.insert(
        "!=$$KeySymbol$$KeySymbol",
        Expr::ForeignFunc(Arc::new(not_eq_symbol)),
    );

    scope.insert(">", Expr::ForeignFunc(Arc::new(gt)));
    scope.insert("<", Expr::ForeignFunc(Arc::new(lt)));

    // #todo `eq` and `Comparable` are related.
    // #todo consider to make sorter: `cmp`.

    scope.insert(
        "compare",
        Expr::ForeignFunc(Arc::new(arithmetic::int_compare)),
    );
    scope.insert(
        "compare$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::int_compare)), "Int"),
    );

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
    scope.insert("sort!", Expr::ForeignFunc(Arc::new(array_sort_mut)));

    // dict

    // #todo add type qualifiers!
    scope.insert(
        "contains-key",
        Expr::ForeignFunc(Arc::new(dict_contains_key)),
    );
    scope.insert("insert", Expr::ForeignFunc(Arc::new(dict_insert)));
    scope.insert("update!", Expr::ForeignFunc(Arc::new(dict_update_mut)));
    scope.insert("get-or", Expr::ForeignFunc(Arc::new(dict_get_or)));
    scope.insert("get-keys", Expr::ForeignFunc(Arc::new(dict_get_keys)));
    scope.insert("get-values", Expr::ForeignFunc(Arc::new(dict_get_values)));

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

    // #todo slice is to general works both as noun and verb, try to find an explicit verb? e.g. `cut` or `carve`
    // #todo alternatively use something like `get-slice` or `cut-slice` or `carve-slice`.
    scope.insert("slice", Expr::ForeignFunc(Arc::new(string_slice)));
    scope.insert(
        "slice$$String$$Int$$Int",
        Expr::ForeignFunc(Arc::new(string_slice)),
    );
    scope.insert(
        "slice$$String$$(Range Int)",
        Expr::ForeignFunc(Arc::new(string_slice_range)),
    );

    // #todo find a bette name, `size`?
    // #insight `count` is _not_ a good name, reserve it for verb/action.
    scope.insert("get-length", Expr::ForeignFunc(Arc::new(string_get_length)));
    scope.insert(
        "get-length$$String",
        Expr::ForeignFunc(Arc::new(string_get_length)),
    );

    scope.insert(
        "starts-with?",
        Expr::ForeignFunc(Arc::new(string_starts_with)),
    );

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
