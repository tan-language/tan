use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    expr::{expr_clone, format_value, Expr},
    util::{
        args::{unpack_map_mut_arg, unpack_stringable_arg},
        module_util::require_module,
    },
};

// #todo implement some of those functions: https://www.programiz.com/python-programming/methods/mapionary

// #insight use `contains-key` so that `contains` refers to the value, consistent with other collections.
// #todo consider other names: has, has-key, contains-key, includes, etc.
// #todo consider appending a `?`
pub fn map_contains_key(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [map, key] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and `key` argument",
            None,
        ));
    };

    let Some(items) = map.as_map_mut() else {
        return Err(Error::invalid_arguments(
            "`map` argument should be a Map",
            map.range(),
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
pub fn map_put(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [map, key, value] = args else {
        return Err(Error::invalid_arguments(
            "requires `this`, `key`, and `value` arguments",
            None,
        ));
    };

    let Some(mut items) = map.as_map_mut() else {
        return Err(Error::invalid_arguments(
            "`map` argument should be a Map",
            map.range(),
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
    Ok(Expr::None)
}

// #todo how is this related with HTTP PATCH?
// #todo alternative names: `merge`, `patch`, `extend` (from Rust)
// #todo I think `extend` is better, more descriptive.
// #todo have draining and non-draining versions (drain other.) (consuming is better than draining)
// #todo have mutating and non-mutating versions.
pub fn map_update_mut(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this, other] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and `other` argument",
            None,
        ));
    };

    let Some(mut this_items) = this.as_map_mut() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be a Map",
            this.range(),
        ));
    };

    let Some(other_items) = other.as_map() else {
        return Err(Error::invalid_arguments(
            "`other` argument should be a Map",
            other.range(),
        ));
    };

    // #todo expensive clone
    // let it = other_items.clone().into_iter();
    // this_items.extend(it);

    // #todo still expensive
    for (key, value) in other_items.iter() {
        this_items.insert(key.clone(), value.clone());
    }

    // #todo what to return?
    // Ok(this.clone()) // #todo this is expensive, just use Rc/Arc everywhere.
    Ok(Expr::None)
}

// #todo could be replaced with `some-or` or Maybe functions.
// #todo temp method until we have Maybe
// #todo (map :key <default>) could accept a default value.
// #todo this should be a special form, not evaluate the default value if not needed (short-circuit).
// #todo consider making default optional.
pub fn map_get_or(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo rename `default_value` to `fallback`, more descriptive.
    let [map, key, default_value] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and `key` argument",
            None,
        ));
    };

    let Some(items) = map.as_map_mut() else {
        return Err(Error::invalid_arguments(
            "`map` argument should be a Map",
            map.range(),
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

// #todo Also consider the name `delete` (or even `yank`)?
pub fn map_remove(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let mut map = unpack_map_mut_arg(args, 0, "map")?;
    let key = unpack_stringable_arg(args, 1, "key")?;

    // #todo Should return None if nothing removed!
    // #todo Should this return the value? -> Yes make maximally useful!
    let value = map.remove(key);

    // #insight Returning the value is cheap.
    Ok(value.unwrap_or(Expr::None))
}

// #todo consider name `keys-of` to avoid clash with variable keys? -> get-keys
// #todo document the above in a decision file
// #todo keys is problematic if it's in the prelude!
pub fn map_get_keys(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [map] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    let Some(items) = map.as_map_mut() else {
        return Err(Error::invalid_arguments(
            "`map` argument should be a Map",
            map.range(),
        ));
    };

    let keys: Vec<_> = items.keys().map(Expr::string).collect();

    Ok(Expr::array(keys))
}

pub fn map_get_values(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [map] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    let Some(items) = map.as_map_mut() else {
        return Err(Error::invalid_arguments(
            "`map` argument should be a Map",
            map.range(),
        ));
    };

    let keys: Vec<_> = items.values().map(expr_clone).collect();

    Ok(Expr::array(keys))
}

// #todo consider other names, e.g. `items`.
// #todo introduce entries/get-entries for other collections/containers, even Array/List.
pub fn map_get_entries(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [map] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    let Some(items) = map.as_map_mut() else {
        return Err(Error::invalid_arguments(
            "`map` argument should be a Map",
            map.range(),
        ));
    };

    // #todo why does map return k as String?
    // #todo wow, this is incredibly inefficient.
    // #todo #hack temp fix we add the a `:` prefix to generate keys
    let entries: Vec<_> = items
        .iter()
        .map(|(k, v)| Expr::array(vec![Expr::KeySymbol(k.clone()), expr_clone(v)]))
        .collect();

    Ok(Expr::array(entries))
}

pub fn setup_lib_map(context: &mut Context) {
    let module = require_module("prelude", context);

    // #todo add something like `get-or-init`` or `update-with-default` or `get-and-update`

    // #todo add type qualifiers!
    module.insert(
        "contains-key",
        Expr::ForeignFunc(Arc::new(map_contains_key)),
    );
    module.insert("put", Expr::ForeignFunc(Arc::new(map_put)));
    module.insert("put$$Map", Expr::ForeignFunc(Arc::new(map_put)));
    module.insert("update!", Expr::ForeignFunc(Arc::new(map_update_mut)));
    module.insert("get-or", Expr::ForeignFunc(Arc::new(map_get_or)));

    // #(Func [(Map T) Hashable] T)
    module.insert("remove", Expr::ForeignFunc(Arc::new(map_remove)));

    // #todo Remove older get-* functions {
    module.insert("get-keys", Expr::ForeignFunc(Arc::new(map_get_keys)));
    module.insert("get-values", Expr::ForeignFunc(Arc::new(map_get_values)));
    module.insert("get-entries", Expr::ForeignFunc(Arc::new(map_get_entries)));
    // }
    module.insert("keys-of", Expr::ForeignFunc(Arc::new(map_get_keys)));
    module.insert("values-of", Expr::ForeignFunc(Arc::new(map_get_values)));
    module.insert("entries-of", Expr::ForeignFunc(Arc::new(map_get_entries)));
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::{
        api::eval_string,
        context::Context,
        expr::{format_value, Expr},
    };

    #[test]
    fn map_put_usage() {
        let input = r#"
            (let user {})
            (put user :given-name "Georgios")
            user
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = r#"{:given-name "Georgios"}"#;
        assert_eq!(value, expected);
    }

    #[test]
    fn map_update_mut() {
        let input = r#"
            (let map1 {:given-name "George" :nationality "Greek"})
            (let map2 {:family-name "Moschovitis"})
            (update! map1 map2)
            map1
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let map = expr.as_map().unwrap();

        assert_matches!(map.get("given-name"), Some(Expr::String(s)) if s == "George");
        assert_matches!(map.get("family-name"), Some(Expr::String(s)) if s == "Moschovitis");
        assert_matches!(map.get("nationality"), Some(Expr::String(s)) if s == "Greek");

        assert_eq!(map.len(), 3);
    }

    #[test]
    fn map_contains_key_usage() {
        let input = r#"
            (let user {:name "George"})
            (contains-key user :name)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let contains_key = expr.as_bool().unwrap();
        assert!(contains_key);

        let input = r#"
            (let user {:name "George"})
            (get-or user :role "Admin")
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "Admin";
        assert_eq!(value, expected);
    }

    #[test]
    fn map_get_or_usage() {
        let input = r#"
            (let user {:name "George"})
            (get-or user :name "Zonk")
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "George";
        assert_eq!(value, expected);

        let input = r#"
            (let user {:name "George"})
            (get-or user :role "Admin")
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "Admin";
        assert_eq!(value, expected);
    }

    #[test]
    fn map_get_keys() {
        // #todo `:role :admin` looks weird, not that `role: :admin` looks much better though
        let input = r#"
            (let user {:name "George" :role :admin})
            (get-keys user)
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
    fn map_get_values() {
        let input = r#"
            (let user {:name "George" :role :admin}) ; `:role :admin` is confusing!
            (get-values user)
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

    #[test]
    fn map_get_entries() {
        let input = r#"
            (let user {:name "George" :role :admin}) ; `:role :admin` is confusing!
            (get-entries user)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let expr = expr.to_string();
        // #todo nasty hack to handle unordered map, maybe should sort the keys?
        assert!(
            expr == r#"[[:name "George"] [:role :admin]]"#
                || expr == r#"[[:role :admin] [:name "George"]]"#
        );
    }
}
