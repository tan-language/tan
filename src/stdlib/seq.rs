use crate::{
    context::Context,
    error::Error,
    expr::{format_value, Expr},
};

// #todo implement generically for all iterables/countables, etc.

// #todo version that returns a new sequence
// #todo also consider insert, insert-back, append names
// #todo item or element?
pub fn array_push(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
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
pub fn array_join(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let Some(array) = args.get(0) else {
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

    let elements: Vec<String> = array.iter().map(|e| format_value(e)).collect();

    Ok(Expr::String(elements.join(separator)))
}

// #insight use the word Iterable instead of Sequence/Seq, more generic (can handle non-sequences, e.g. maps)
// #insight could also use Countable

// #todo match the corresponding function in String.
// #todo rename to `get-length`?
// #todo implement generically for iterables.
pub fn array_count(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
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
pub fn array_filter(_args: &[Expr], _context: &Context) -> Result<Expr, Error> {
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
}
