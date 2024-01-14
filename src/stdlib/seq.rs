use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    eval::invoke_func,
    expr::{format_value, Expr},
    util::module_util::require_module,
};

use super::cmp::rust_ordering_from_tan_ordering;

// #todo implement sort! and sort

// #todo implement generically for all iterables/countables, etc.

// #todo version that returns a new sequence
// #todo also consider insert, insert-back, append names
// #todo item or element?
pub fn array_push(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [array, element] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and `element` argument",
            None,
        ));
    };

    let Some(mut elements) = array.as_array_mut() else {
        return Err(Error::invalid_arguments(
            "`array` argument should be a Array",
            array.range(),
        ));
    };

    elements.push(element.unpack().clone()); // #todo hmm this clone!

    // #todo what to return?
    Ok(Expr::One)
}

// #todo hm, it joins as strings, not very general, should move to string?
// #todo support separator param.
/// (join names "\n")
pub fn array_join(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let Some(array) = args.first() else {
        return Err(Error::invalid_arguments("requires `array` argument", None));
    };

    let separator = args.get(1);
    let separator = if separator.is_some() {
        let Some(str) = separator.unwrap().as_stringable() else {
            return Err(Error::invalid_arguments(
                "the `separator` should be a Stringable",
                None,
            ));
        };
        str
    } else {
        ""
    };

    let Some(array) = array.as_array() else {
        return Err(Error::invalid_arguments(
            "`array` argument should be a Array",
            array.range(),
        ));
    };

    let elements: Vec<String> = array.iter().map(format_value).collect();

    Ok(Expr::String(elements.join(separator)))
}

// #insight use the word Iterable instead of Sequence/Seq, more generic (can handle non-sequences, e.g. maps)
// #insight could also use Countable

// #todo match the corresponding function in String.
// #todo rename to `get-length`?
// #todo implement generically for iterables.
pub fn array_count(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [array, ..] = args else {
        return Err(Error::invalid_arguments("requires `array` argument", None));
    };

    let Some(array) = array.as_array() else {
        return Err(Error::invalid_arguments(
            "`array` argument should be a Array",
            array.range(),
        ));
    };

    Ok(Expr::Int(array.len() as i64))
}

// #todo how to implement this?
pub fn array_filter(_args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    todo!();

    // // #todo
    // let [seq, predicate_fn] = args else {
    //     return Err(Error::invalid_arguments(
    //         "requires `this` and `predicate-fn` arguments",
    //         None,
    //     ));
    // };

    // let Some(arr) = seq.as_array() else {
    //     return Err(Error::invalid_arguments(
    //         "`filter` requires a `Seq` as the first argument",
    //         seq.range(),
    //     ));
    // };

    // let prev_scope = context.scope.clone();
    // // context.scope = Rc::new(Scope::new(prev_scope.clone()));

    // let mut results: Vec<Expr> = Vec::new();

    // for x in arr.iter() {
    //     // #todo how to call a closure?

    //     // // #todo array should have Ann<Expr> use Ann<Expr> everywhere, avoid the clones!
    //     // context.scope.insert(sym, x.clone());
    //     // let result = eval(body, context)?;
    //     // // #todo replace the clone with custom expr::ref/copy?
    //     // results.push(result.unpack().clone());
    // }

    // // context.scope = prev_scope.clone();

    // // #todo intentionally don't return a value, reconsider this?
    // Ok(Expr::array(results).into())
}

// #todo implement first, last

// #todo implement sort!, sort, sort-by!, sort-by
// #todo need to introduce Comparable trait and (cmp ...) or (compare ...)
// #todo need to introduce Ordering trait
// (sort [9 2 7] (Func [a b] (- a b)))
// (sort [9 2 7] (-> [a b] (- a b)))
pub fn array_sort_mut(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let [array, func] = args else {
        return Err(Error::invalid_arguments(
            "requires `array` and `func` arguments",
            None,
        ));
    };

    let Some(mut array_items) = array.as_array_mut() else {
        return Err(Error::invalid_arguments(
            "`array` argument should be a Array",
            array.range(),
        ));
    };

    // #todo validate func is a comparator.
    // #todo validate that params has the correct structure.

    array_items.sort_by(|x, y| {
        // #todo how to handle errors here?
        let tan_ordering = invoke_func(func, &[x.clone(), y.clone()], context).unwrap();
        rust_ordering_from_tan_ordering(&tan_ordering).unwrap()
    });

    // #insight interesting that we are also returning the input.

    // Ok(Expr::array(array_items.clone()))
    Ok(array.clone())
}

pub fn setup_lib_seq(context: &mut Context) {
    let module = require_module("prelude", context);

    // #todo add type qualifiers!
    module.insert("push", Expr::ForeignFunc(Arc::new(array_push)));
    module.insert("join", Expr::ForeignFunc(Arc::new(array_join)));
    module.insert("count", Expr::ForeignFunc(Arc::new(array_count)));
    module.insert("sort!", Expr::ForeignFunc(Arc::new(array_sort_mut)));
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn array_push_usage() {
        let input = r#"
            (let arr [1 2])
            (push arr 3)
            (push arr 4)
            arr
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "[1 2 3 4]";
        assert_eq!(value, expected);
    }

    #[test]
    fn array_join_usage() {
        let input = r#"
            (let arr ["george" "chris" "alex"])
            (join arr ", ")
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "george, chris, alex";
        assert_eq!(value, expected);

        let input = r#"
            (let arr ["*" "*" "*"])
            (join arr)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "***";
        assert_eq!(value, expected);
    }

    #[test]
    fn array_filter_usage() {}

    #[test]
    fn array_sort_mut_usage() {
        let mut context = Context::new();

        let input = r#"
            (sort! [5 1 6 4 3] (Func [x y] (- x y)))
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "[1 3 4 5 6]";
        assert_eq!(value, expected);

        let input = r#"
            (sort! [5 1 6 4 3] (Func [x y] (- y x)))
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "[6 5 4 3 1]";
        assert_eq!(value, expected);
    }
}
