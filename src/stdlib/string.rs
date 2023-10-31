use crate::{
    context::Context,
    error::Error,
    expr::{format_value, Expr},
};

/// Returns a char iterable for the chars in the string.
pub fn string_chars(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments(
            "`chars` requires `this` argument",
            None,
        ));
    };

    let Some(this) = this.as_string() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be a String",
            this.range(),
        ));
    };

    let mut exprs: Vec<Expr> = Vec::new();

    for char in this.chars() {
        exprs.push(Expr::Char(char));
    }

    Ok(Expr::array(exprs))
}

pub fn string_constructor_from_chars(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [chars] = args else {
        return Err(Error::invalid_arguments("requires `chars` argument", None));
    };

    let Some(exprs) = chars.as_array() else {
        return Err(Error::invalid_arguments(
            "`chars` argument should be a (Array Char)",
            chars.range(),
        ));
    };

    // #todo verify Array item type!

    let mut chars: Vec<char> = Vec::new();

    for expr in exprs.iter() {
        if let Some(c) = expr.as_char() {
            chars.push(c);
        }
    }

    let string = String::from_iter(chars);

    Ok(Expr::String(string))
}

// #todo overload for string and char!

pub fn char_uppercased(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments(
            "`uppercased` requires `this` argument",
            None,
        ));
    };

    let Expr::Char(this) = this.unpack() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be a Char",
            this.range(),
        ));
    };

    // #todo omg...
    let uppercased = this.to_uppercase().next().unwrap();

    Ok(Expr::Char(uppercased))
}

// #todo make this a String constructor?
// #todo 'join' and 'format' versions?

// #todo find another name, this is too common: `fmt`? `stringf`?
pub fn format(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let output = args.iter().fold(String::new(), |mut str, x| {
        str.push_str(&format_value(x));
        str
    });

    Ok(Expr::String(output))
}

// name: split
// type: (Func (String String) String)
// macro annotation: (this: String, separator: String) -> String
// (Func (this separator) ..)
// (Func (#String this #String separator) String)
pub fn string_split(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [this, separator] = args else {
        return Err(Error::invalid_arguments(
            "`split` requires `this` and `separator` arguments",
            None,
        ));
    };

    let Some(this) = this.as_string() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be a String",
            this.range(),
        ));
    };

    let Some(separator) = separator.as_string() else {
        return Err(Error::invalid_arguments(
            "`separator` argument should be a String",
            separator.range(),
        ));
    };

    // #todo should return iterator

    let parts: Vec<Expr> = this.split(separator).map(Expr::string).collect();

    Ok(Expr::array(parts))
}

// #todo have FFI functions without Context?

pub fn string_ends_with(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    // #todo consider `suffix` instead of `postfix`.
    let [this, postfix] = args else {
        return Err(Error::invalid_arguments(
            "`ends-with` requires `this` and `postfix` arguments",
            None,
        ));
    };

    let Some(this) = this.as_string() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be a String",
            this.range(),
        ));
    };

    let Some(postfix) = postfix.as_string() else {
        return Err(Error::invalid_arguments(
            "`postfix` argument should be a String",
            postfix.range(),
        ));
    };

    Ok(Expr::Bool(this.ends_with(postfix)))
}

// #todo could allow for multiple replacements (i.e. paris of rules)
// #todo different name? e.g. rewrite?
pub fn string_replace(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [this, from, to] = args else {
        return Err(Error::invalid_arguments(
            "`replace` requires `this`, `from`, and `to` arguments",
            None,
        ));
    };

    let Some(this) = this.as_string() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be a String",
            this.range(),
        ));
    };

    let Some(from) = from.as_string() else {
        return Err(Error::invalid_arguments(
            "`from` argument should be a String",
            from.range(),
        ));
    };

    let Some(to) = to.as_string() else {
        return Err(Error::invalid_arguments(
            "`to` argument should be a String",
            to.range(),
        ));
    };

    Ok(Expr::String(this.replace(from, to)))
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn ends_with_usage() {
        let mut context = Context::new();
        let input = r#"
            (let filename "/path/to/image.png")
            (ends-with? filename ".png")
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = expr.as_bool().unwrap();
        assert!(value);

        let input = r#"
            (let filename "/path/to/image.png")
            (ends-with? filename ".gif")
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = expr.as_bool().unwrap();
        assert_eq!(value, false);
    }

    #[test]
    fn split_usage() {
        let mut context = Context::new();
        let input = r#"(split "path/to/my/secret/file.ext" "/")"#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = r#"["path" "to" "my" "secret" "file.ext"]"#;
        assert_eq!(value, expected);
    }

    #[test]
    fn replace_usage() {
        let mut context = Context::new();
        let input = r#"(replace "Hello George" "George" "Alex")"#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "Hello Alex";
        assert_eq!(value, expected);
    }
}
