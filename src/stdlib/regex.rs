use std::sync::Arc;

use regex::Regex;

use crate::{
    context::Context,
    error::Error,
    expr::{annotate_type, Expr},
    util::module_util::require_module,
};

// #todo have more sophisticated patterns and matching.
// #toto check swift patterns.

// #todo introduce escaping
// #todo introduce capturing
// #todo introduce matching
// #todo introduce is-matching? or just matching?

pub fn regex_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo name this just `expr`?
    let [pattern] = args else {
        return Err(Error::invalid_arguments(
            "requires `pattern` argument",
            None,
        ));
    };

    let Some(pattern) = pattern.as_stringable() else {
        return Err(Error::invalid_arguments(
            "`pattern` argument should be a String",
            pattern.range(),
        ));
    };

    // #todo can we keep an opaque pointer to an actual Rust Regex instead?

    let rx = Expr::string(pattern);

    // #todo consider Regexpr? or Reg-Expr? naah...

    Ok(annotate_type(rx, "Regex"))
}

pub fn regex_is_matching(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this, string] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and `string` arguments",
            None,
        ));
    };

    // #todo verify that is Regex, not just string?
    let Some(re_pattern) = this.as_string() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be a Regex",
            this.range(),
        ));
    };

    let Some(string) = string.as_stringable() else {
        return Err(Error::invalid_arguments(
            "`string` argument should be a Stringable",
            string.range(),
        ));
    };

    // #todo proper error reporting here!
    let Ok(re) = Regex::new(re_pattern) else {
        return Err(Error::invalid_arguments(
            &format!("invalid regex pattern: {re_pattern}"),
            this.range(),
        ));
    };

    Ok(Expr::Bool(re.is_match(string)))
}

pub fn setup_lib_regex(context: &mut Context) {
    // #todo find a better module-path
    let module = require_module("regex", context);

    // #todo should we introduce a special Regex literal? probably not.
    // #todo maybe a tagged string though, e.g. `"pattern"rx` or `"pattern"r` ?

    module.insert("Regex", Expr::ForeignFunc(Arc::new(regex_new)));

    // #todo  consider is-matching?, nah, let's make the `?` suffix useful.
    module.insert("matching?", Expr::ForeignFunc(Arc::new(regex_is_matching)));
}
