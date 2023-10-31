use crate::{
    context::Context,
    error::Error,
    expr::{expr_clone, Expr},
};

// #todo version that returns a new sequence
// #todo also consider set, put
// #todo item or element? -> I think for collections item is better.
pub fn dict_insert(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [dict, key, value] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and `item` argument",
            None,
        ));
    };

    let Some(mut items) = dict.as_dict_mut() else {
        return Err(Error::invalid_arguments(
            "`dict` argument should be a Dict",
            dict.range(),
        ));
    };

    // #todo support non-string/symbol keys
    // #todo support string keys also.

    let Expr::KeySymbol(key) = key.unpack() else {
        return Err(Error::invalid_arguments(
            "`key` argument should be a KeySymbol",
            key.range(),
        ));
    };

    // #idea instead convert key to string? or hash?

    items.insert(key.clone(), value.unpack().clone()); // #todo hmm this clone!

    // #todo what to return?
    Ok(Expr::One)
}

// #todo temp method until we have Maybe
// #todo (dict :key <default>) could accept a default value.
// #todo this should be a special form, not evaluate the default value if not needed (short-circuit).
// #todo consider making default optional.
pub fn dict_get_or(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [dict, key, default_value] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and `item` argument",
            None,
        ));
    };

    let Some(items) = dict.as_dict_mut() else {
        return Err(Error::invalid_arguments(
            "`dict` argument should be a Dict",
            dict.range(),
        ));
    };

    // #todo support non-string/symbol keys
    // #todo support string keys also.

    let Expr::KeySymbol(key) = key.unpack() else {
        return Err(Error::invalid_arguments(
            "`key` argument should be a KeySymbol",
            key.range(),
        ));
    };

    // #idea instead convert key to string? or hash?

    let value = items.get(key);

    // #todo can we remove the clones?

    if let Some(value) = value {
        Ok(expr_clone(value))
    } else {
        Ok(expr_clone(default_value))
    }
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn dict_insert_usage() {
        let input = r#"
            (let dict {})
            (insert dict :given-name "Georgios")
            dict
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = r#"{:given-name "Georgios"}"#;
        assert_eq!(value, expected);
    }

    #[test]
    fn dict_get_or_usage() {
        let input = r#"
            (let dict {:name "George"})
            (get-or dict :name "Zonk")
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "George";
        assert_eq!(value, expected);

        let input = r#"
            (let dict {:name "George"})
            (get-or dict :role "Admin")
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "Admin";
        assert_eq!(value, expected);
    }
}