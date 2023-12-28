use crate::{
    context::Context,
    error::Error,
    expr::{expr_clone, format_value, Expr},
};

// #insight use `contains-key` so that `contains` refers to the value, consistent with other collections.
// #todo consider other names: has, has-key, contains-key, includes, etc.
// #todo consider appending a `?`
pub fn dict_contains_key(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [dict, key] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and `key` argument",
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

    // #todo temp solution!
    let key = format_value(key);

    // #idea instead convert key to string? or hash?

    Ok(Expr::Bool(items.contains_key(&key)))
}

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

    // let Expr::KeySymbol(key) = key.unpack() else {
    //     return Err(Error::invalid_arguments(
    //         "`key` argument should be a KeySymbol",
    //         key.range(),
    //     ));
    // };

    // #todo temp solution!
    let key = format_value(key);

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
            "requires `this` and `key` argument",
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

    // let Expr::KeySymbol(key) = key.unpack() else {
    //     return Err(Error::invalid_arguments(
    //         "`key` argument should be a KeySymbol",
    //         key.range(),
    //     ));
    // };

    // #todo temp solution!
    let key = format_value(key);

    // #idea instead convert key to string? or hash?

    let value = items.get(&key);

    // #todo can we remove the clones?

    if let Some(value) = value {
        Ok(expr_clone(value))
    } else {
        Ok(expr_clone(default_value))
    }
}

// #todo consider name `keys-of` to avoid clash with variable keys? -> get-keys
// #todo document the above in a decision file
// #todo keys is problematic if it's in the prelude!
pub fn dict_get_keys(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [dict] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    let Some(items) = dict.as_dict_mut() else {
        return Err(Error::invalid_arguments(
            "`dict` argument should be a Dict",
            dict.range(),
        ));
    };

    let keys: Vec<_> = items.keys().map(Expr::string).collect();

    Ok(Expr::array(keys))
}

pub fn dict_get_values(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [dict] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    let Some(items) = dict.as_dict_mut() else {
        return Err(Error::invalid_arguments(
            "`dict` argument should be a Dict",
            dict.range(),
        ));
    };

    let keys: Vec<_> = items.values().map(expr_clone).collect();

    Ok(Expr::array(keys))
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
    fn dict_contains_key_usage() {
        let input = r#"
            (let dict {:name "George"})
            (contains-key dict :name)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let contains_key = expr.as_bool().unwrap();
        assert!(contains_key);

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

    #[test]
    fn dict_get_keys() {
        // #todo `:role :admin` looks weird, not that `role: :admin` looks much better though
        let input = r#"
            (let dict {:name "George", :role :admin})
            (get-keys dict)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let keys: Vec<String> = expr
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.to_string())
            .collect();
        // #fixme unfortunately it currently looses the ':' prefix.
        assert!(keys.contains(&r#""name""#.to_string()));
        assert!(keys.contains(&r#""role""#.to_string()));
    }

    #[test]
    fn dict_get_values() {
        let input = r#"
            (let dict {:name "George", :role :admin}) ; `:role :admin` is confusing!
            (get-values dict)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let values: Vec<String> = expr
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.to_string())
            .collect();
        assert!(values.contains(&r#""George""#.to_string()));
        assert!(values.contains(&":admin".to_string()));
    }
}
