use std::sync::Arc;

use regex::Regex;

use crate::{
    context::Context,
    error::Error,
    expr::{annotate_type, Expr},
    util::{args::unpack_stringable_arg, module_util::require_module},
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

    // #todo consider Regexpr? or Regexp, or Reg-Expr? naah...

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

pub fn regex_split(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo What would be a good name? What about `re_pattern`?
    let regex = unpack_stringable_arg(args, 0, "sep-regex")?;
    // #todo Consider the name `input`.
    let text = unpack_stringable_arg(args, 1, "text")?;

    // #todo Verify that is Regex, not just string?
    // #todo Extract the above as function (as_regex).

    // #todo proper error reporting here!
    let Ok(re) = Regex::new(regex) else {
        return Err(Error::invalid_arguments(
            &format!("invalid regex pattern: `{regex}`"),
            args[0].range(),
        ));
    };

    let parts: Vec<Expr> = re.split(text).map(Expr::string).collect();

    Ok(Expr::array(parts))
}

// #todo introduce find/find-all that return matches.

// #insight the regext matches the parts, not the separator.
// #todo this is a peculiar function that probably should go away.
pub fn regex_split_matches(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo What would be a good name? What about `re_pattern`?
    let regex = unpack_stringable_arg(args, 0, "match-regex")?;
    // #todo Consider the name `input`.
    let text = unpack_stringable_arg(args, 1, "text")?;

    // #todo Verify that is Regex, not just string?
    // #todo Extract the above as function (as_regex).

    // #todo proper error reporting here!
    let Ok(re) = Regex::new(regex) else {
        return Err(Error::invalid_arguments(
            &format!("invalid regex pattern: `{regex}`"),
            args[0].range(),
        ));
    };

    let parts: Vec<Expr> = re
        .find_iter(text)
        .map(|m| Expr::string(m.as_str()))
        .collect();

    Ok(Expr::array(parts))
}

// #todo support named captures? nah, too much.
// #todo capture-one <> capture-many, or capture <> capture*, or use xxxx* for generators/coroutines?
pub fn regex_capture(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let re_pattern = unpack_stringable_arg(args, 0, "re")?;
    let string = unpack_stringable_arg(args, 1, "string")?;

    // #todo proper error reporting here!
    let Ok(re) = Regex::new(re_pattern) else {
        return Err(Error::invalid_arguments(
            &format!("invalid regex pattern: {re_pattern}"),
            args[0].range(),
        ));
    };

    let mut captures: Vec<Expr> = Vec::new();

    for cap in re.captures_iter(string) {
        // #todo #fixme temporarily only returns the first capture.
        // eprintln!("---> {} {}", cap.get(0).unwrap().as_str(), cap.len());
        let value = cap.get(1).unwrap().as_str();
        captures.push(Expr::string(value));
    }

    Ok(Expr::array(captures))
}

pub fn setup_lib_regex(context: &mut Context) {
    // #todo find a better module-path
    let module = require_module("regex", context);

    // #todo should we introduce a special Regex literal? probably not.
    // #todo maybe a tagged string though, e.g. `"pattern"rx` or `"pattern"r` ?

    module.insert("Regex", Expr::ForeignFunc(Arc::new(regex_new)));

    // #todo consider is-matching?, nah, let's make the `?` suffix useful.
    module.insert("matching?", Expr::ForeignFunc(Arc::new(regex_is_matching)));
    module.insert(
        "split$$Regex$$String",
        Expr::ForeignFunc(Arc::new(regex_split)),
    );
    module.insert(
        "split-matches",
        Expr::ForeignFunc(Arc::new(regex_split_matches)),
    );

    //
    module.insert("capture", Expr::ForeignFunc(Arc::new(regex_capture)));
}

// #todo move these tests to Tan.

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn is_matching_usage() {
        let mut context = Context::new();
        let expr = eval_string(
            r#"
            (use [Regex matching?] regex)
            (let rx (Regex "\\d+"))
            (matching? rx "1234")
        "#,
            &mut context,
        )
        .unwrap();
        assert_eq!(format_value(expr), "true");

        let expr = eval_string(
            r#"
            (let rx (Regex "\\d+"))
            (matching? rx "hello")
        "#,
            &mut context,
        )
        .unwrap();
        assert_eq!(format_value(expr), "false");
    }

    #[test]
    fn capture_usage() {
        let mut context = Context::new();
        let expr = eval_string(
            r#"
            (use [Regex capture] regex)
            (let rx (Regex "\\s*(\\d+)\\s*"))
            (capture rx " 12,  345 6 7890    11 ")
        "#,
            &mut context,
        )
        .unwrap();
        assert_eq!(format_value(expr), r#"["12" "345" "6" "7890" "11"]"#);

        let expr = eval_string(
            r#"
            (use [Regex capture] regex)
            (let rx (Regex "class=\"([\w ]*)\""))
            (capture rx "<div class=\"cool stuff\"><i class=\"nice\">works</i><i class=\"work\">works</i></div>")
        "#,
            &mut context,
        )
        .unwrap();
        assert_eq!(format_value(expr), r#"["cool stuff" "nice" "work"]"#);
    }
}
