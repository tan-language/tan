use crate::{context::Context, error::Error, expr::Expr};

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

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn dict_insert_usage() {
        let input = r#"
            (let dict {})
            (insert dict :given-name "Georgios")
            (insert dict :family-name "Moschovitis")
            dict
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = r#"{:given-name "Georgios" :family-name "Moschovitis"}"#;
        assert_eq!(value, expected);
    }
}
