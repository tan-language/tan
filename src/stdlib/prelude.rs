use std::sync::Arc;

use crate::expr::annotate_type;
use crate::util::module_util::require_module;
use crate::{context::Context, expr::Expr};

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
    string_compare, string_ends_with, string_get_length, string_replace, string_slice,
    string_slice_range, string_split, string_starts_with,
};
use super::{
    eq::{eq, gt, lt},
    seq::array_join,
    string::{char_uppercased, format, string_chars, string_constructor_from_chars},
};

// #todo instead of evaluating in prelude maybe it's better to use the functions from the actual modules?
pub fn setup_lib_prelude(context: &mut Context) {
    // #todo maybe context.require_module(path) instead?
    let module = require_module("prelude", context);

    // num

    // #todo forget the mangling, implement with a dispatcher function, multi-function.
    module.insert(
        "+",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_int)), "Int"),
    );
    module.insert(
        "+$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_int)), "Int"),
    );
    module.insert(
        "+$$Float$$Float",
        // #todo add the proper type: (Func Float Float Float)
        // #todo even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_float)), "Float"),
    );
    #[cfg(feature = "dec")]
    module.insert(
        "+$$Dec$$Dec",
        // #todo add the proper type: (Func Dec Dec Dec)
        // #todo even better: (Func (Many Dec) Dec)
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::add_dec)), "Dec"),
    );
    module.insert("-", Expr::ForeignFunc(Arc::new(arithmetic::sub_int)));
    module.insert(
        "-$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::sub_int)), "Int"),
    );
    module.insert(
        "-$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::sub_float)), "Float"),
    );
    module.insert("*", Expr::ForeignFunc(Arc::new(arithmetic::mul_int)));
    module.insert(
        "*$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::mul_int)), "Int"),
    );
    module.insert(
        "*$$Float$$Float",
        // #todo add the proper type: (Func Float Float Float)
        // #todo even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::mul_float)), "Float"),
    );
    module.insert(
        "/",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::div_float)), "Float"),
    );
    // #todo ultra-hack
    module.insert(
        "/$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::div_float)), "Float"),
    );
    // #todo ultra-hack
    module.insert(
        "/$$Float$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::div_float)), "Float"),
    );
    module.insert(
        "sin",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::sin_float)), "Float"),
    );
    module.insert(
        "cos",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::cos_float)), "Float"),
    );
    module.insert(
        "**",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::powi_float)), "Float"),
    );

    // eq

    module.insert("=", Expr::ForeignFunc(Arc::new(eq)));
    module.insert("=$$Int$$Int", Expr::ForeignFunc(Arc::new(eq)));
    module.insert("=$$Float$$Float", Expr::ForeignFunc(Arc::new(eq_float)));
    module.insert("=$$String$$String", Expr::ForeignFunc(Arc::new(eq_string)));
    // module.insert("=$$Symbol$$Symbol", Expr::ForeignFunc(Arc::new(eq_symbol)));
    module.insert(
        "=$$KeySymbol$$KeySymbol",
        Expr::ForeignFunc(Arc::new(eq_symbol)),
    );

    module.insert("!=", Expr::ForeignFunc(Arc::new(not_eq)));
    module.insert("!=$$Int$$Int", Expr::ForeignFunc(Arc::new(not_eq)));
    module.insert(
        "!=$$Float$$Float",
        Expr::ForeignFunc(Arc::new(not_eq_float)),
    );
    module.insert(
        "!=$$String$$String",
        Expr::ForeignFunc(Arc::new(not_eq_string)),
    );
    module.insert(
        "!=$$Symbol$$Symbol",
        Expr::ForeignFunc(Arc::new(not_eq_symbol)),
    );
    module.insert(
        "!=$$KeySymbol$$KeySymbol",
        Expr::ForeignFunc(Arc::new(not_eq_symbol)),
    );

    module.insert(">", Expr::ForeignFunc(Arc::new(gt)));
    module.insert("<", Expr::ForeignFunc(Arc::new(lt)));

    // cmp

    // #todo `eq` and `Comparable` are related.
    // #todo consider to make sorter: `cmp`.

    module.insert(
        "compare",
        Expr::ForeignFunc(Arc::new(arithmetic::int_compare)),
    );
    module.insert(
        "compare$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(arithmetic::int_compare)), "Int"),
    );
    module.insert(
        "compare$$String$$String",
        annotate_type(Expr::ForeignFunc(Arc::new(string_compare)), "String"),
    );

    // io

    // #todo grab those from /std/io module

    module.insert("read", Expr::ForeignFunc(Arc::new(read_string)));
    module.insert("read$$String", Expr::ForeignFunc(Arc::new(read_string)));

    module.insert("write", Expr::ForeignFunc(Arc::new(write)));
    module.insert("write$$String", Expr::ForeignFunc(Arc::new(write)));

    module.insert("writeln", Expr::ForeignFunc(Arc::new(writeln)));
    module.insert("writeln$$String", Expr::ForeignFunc(Arc::new(writeln)));

    // seq

    // #todo add type qualifiers!
    module.insert("push", Expr::ForeignFunc(Arc::new(array_push)));
    module.insert("join", Expr::ForeignFunc(Arc::new(array_join)));
    module.insert("count", Expr::ForeignFunc(Arc::new(array_count)));
    module.insert("sort!", Expr::ForeignFunc(Arc::new(array_sort_mut)));

    // dict

    // #todo add type qualifiers!
    module.insert(
        "contains-key",
        Expr::ForeignFunc(Arc::new(dict_contains_key)),
    );
    module.insert("insert", Expr::ForeignFunc(Arc::new(dict_insert)));
    module.insert("update!", Expr::ForeignFunc(Arc::new(dict_update_mut)));
    module.insert("get-or", Expr::ForeignFunc(Arc::new(dict_get_or)));
    module.insert("get-keys", Expr::ForeignFunc(Arc::new(dict_get_keys)));
    module.insert("get-values", Expr::ForeignFunc(Arc::new(dict_get_values)));

    // string

    // #todo define string functions in string.rs

    module.insert(
        "String",
        Expr::ForeignFunc(Arc::new(string_constructor_from_chars)),
    );
    // env.insert("String$$Array", Expr::ForeignFunc(Arc::new(string_constructor_from_chars)));

    module.insert("chars", Expr::ForeignFunc(Arc::new(string_chars)));
    module.insert("chars$$String", Expr::ForeignFunc(Arc::new(string_chars)));

    module.insert("uppercased", Expr::ForeignFunc(Arc::new(char_uppercased)));
    module.insert(
        "uppercases$$Char",
        Expr::ForeignFunc(Arc::new(char_uppercased)),
    );

    module.insert("format", Expr::ForeignFunc(Arc::new(format)));

    module.insert("split", Expr::ForeignFunc(Arc::new(string_split)));

    module.insert("replace", Expr::ForeignFunc(Arc::new(string_replace)));

    // #todo slice is to general works both as noun and verb, try to find an explicit verb? e.g. `cut` or `carve`
    // #todo alternatively use something like `get-slice` or `cut-slice` or `carve-slice`.
    module.insert("slice", Expr::ForeignFunc(Arc::new(string_slice)));
    module.insert(
        "slice$$String$$Int$$Int",
        Expr::ForeignFunc(Arc::new(string_slice)),
    );
    module.insert(
        "slice$$String$$(Range Int)",
        Expr::ForeignFunc(Arc::new(string_slice_range)),
    );

    // #todo find a bette name, `size`?
    // #insight `count` is _not_ a good name, reserve it for verb/action.
    module.insert("get-length", Expr::ForeignFunc(Arc::new(string_get_length)));
    module.insert(
        "get-length$$String",
        Expr::ForeignFunc(Arc::new(string_get_length)),
    );

    module.insert(
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
    module.insert("ends-with?", Expr::ForeignFunc(Arc::new(string_ends_with)));

    // setup_lib_eq(context);
}
