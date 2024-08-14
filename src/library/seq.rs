use std::sync::Arc;

use crate::{
    context::Context,
    error::Error,
    eval::{invoke, invoke_func},
    expr::{expr_clone, format_value, Expr},
    util::{
        args::{unpack_array_arg, unpack_int_arg},
        module_util::require_module,
    },
};

use super::cmp::rust_ordering_from_tan_ordering;

// #insight Iterable is more general than Sequence. For example you could consider
// a Map as an Iterable it's more of a stretch to think of it as a Sequence.

// #todo Rename to `iter.rs`.

// #todo Find a better name
pub fn list_cons(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [head, tail] = args else {
        return Err(Error::invalid_arguments(
            "requires `head` and `tail` arguments",
            None,
        ));
    };

    let Some(tail) = tail.as_list() else {
        return Err(Error::invalid_arguments(
            "`tail` argument should be a List",
            tail.range(),
        ));
    };

    // #todo this is slow!
    let mut cons_items = vec![expr_clone(head.unpack())];
    for expr in tail {
        cons_items.push(expr_clone(expr));
    }

    Ok(Expr::List(cons_items))
}

// #todo find better name, match Array and String.
pub fn list_count(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [list, ..] = args else {
        return Err(Error::invalid_arguments("requires `list` argument", None));
    };

    let Some(list) = list.as_list() else {
        return Err(Error::invalid_arguments(
            "`list` argument should be a List",
            list.range(),
        ));
    };

    Ok(Expr::Int(list.len() as i64))
}

// #todo implement slice _and_ takes

// #todo implement sort! and sort (or sort, to-sorted)
// #todo add put/insert at index

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
    Ok(Expr::None)
}

// #todo generic Seq/extend, append on arrays, prepends on linked-lists.
// #todo support concatenation of more than two arrays.
// #todo find a good name
// #todo consider the `++` operator
pub fn array_concat_mut(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [array1, array2] = args else {
        return Err(Error::invalid_arguments("requires two arguments", None));
    };

    let Some(mut array1) = array1.as_array_mut() else {
        return Err(Error::invalid_arguments(
            "`array1` argument should be a Array",
            array1.range(),
        ));
    };

    let Some(mut array2) = array2.as_array_mut() else {
        return Err(Error::invalid_arguments(
            "`array2` argument should be a Array",
            array2.range(),
        ));
    };

    array1.append(&mut array2);

    // #todo what to return?
    Ok(Expr::None)
}

// #todo consider the name intercalate from haskell?
// #todo can we find a more specific name?
// #todo hm, it joins as strings, not very general, should move to string?
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

// #todo do we really want to support the no-argument case?
/// (skip items 5) ; skips the first 5 elements
/// (skip items) ; skips the first element
pub fn array_skip(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #insight
    // An alternative name could be `drop` but for the moment we reserve this for
    // the memory operation. Additionally, skip is a bit more descriptive.
    let Some(array) = args.first() else {
        return Err(Error::invalid_arguments("requires `array` argument", None));
    };

    let n = args.get(1);
    let n = if n.is_some() {
        let Some(n) = n.unwrap().as_int() else {
            return Err(Error::invalid_arguments("`n` should be an Int", None));
        };
        n
    } else {
        1
    };

    let Some(array) = array.as_array() else {
        return Err(Error::invalid_arguments(
            "`array` argument should be a Array",
            array.range(),
        ));
    };

    let elements: Vec<Expr> = array.iter().skip(n as usize).cloned().collect();

    Ok(Expr::array(elements))
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

// #todo implement with tan code!
pub fn array_is_empty(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [array, ..] = args else {
        return Err(Error::invalid_arguments("requires `array` argument", None));
    };

    let Some(array) = array.as_array() else {
        return Err(Error::invalid_arguments(
            "`array` argument should be a Array",
            array.range(),
        ));
    };

    Ok(Expr::Bool(array.len() == 0))
}

pub fn array_contains(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [array, element] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and `element` argument",
            None,
        ));
    };

    let Some(elements) = array.as_array_mut() else {
        return Err(Error::invalid_arguments(
            "`array` argument should be a Array",
            array.range(),
        ));
    };

    Ok(Expr::Bool(elements.contains(element.unpack())))
}

// #todo add unit tests.
pub fn array_map(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    let [func, seq] = args else {
        return Err(Error::invalid_arguments(
            "requires `func` and `seq` arguments",
            None,
        ));
    };

    // #todo should relax to allow for Iterable.
    let Some(input_values) = seq.as_array() else {
        return Err(Error::invalid_arguments(
            "`seq` must be an `Array`",
            seq.range(),
        ));
    };

    // #insight cannot use map, because of the `?` operator.

    let mut output_values: Vec<Expr> = Vec::new();

    // #todo make sure that errors in the mapping function are propagated.

    for x in input_values.iter() {
        // #todo can we remove this clone somehow?
        let args = vec![expr_clone(x)];
        // let args = vec![eval(x, context)?];
        // #todo #hack need to rething invoke_func/invoke_func_inner!!
        output_values.push(invoke(func, args, context)?);
    }

    Ok(Expr::array(output_values))
}

// #todo this can actually be implemented with invoke_func.
// #todo how to implement this? -> implement with tan code!
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
        // #todo should we evaluate array items?
        // #insight args are already evaluated!
        // let args = vec![eval(x, context).unwrap(), eval(y, context).unwrap()];
        let args = vec![x.clone(), y.clone()];
        let tan_ordering = invoke_func(func, args, context).unwrap();
        rust_ordering_from_tan_ordering(&tan_ordering).unwrap()
    });

    // #insight interesting that we are also returning the input.

    // Ok(Expr::array(array_items.clone()))
    Ok(array.clone())
}

// #todo enforce range within string length
// #todo rename to `cut`? (as in 'cut a slice')
// #todo relation with range?
// #todo pass range as argument?
// #todo support negative index: -1 => length - 1
// #insight negative index _may_ be problematic if the index is computed and returns negative by mistake.
/// (slice arr 2 5)
/// (slice arr 2)
/// (slice arr 2 -2) ; -2 is length - 2
pub fn array_slice(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this, start, ..] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and start arguments",
            None,
        ));
    };

    let Some(elements) = this.as_array() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be an Array",
            this.range(),
        ));
    };

    let Some(start) = start.as_int() else {
        return Err(Error::invalid_arguments(
            "`start` argument should be an Int",
            this.range(),
        ));
    };

    let end = if let Some(end) = args.get(2) {
        let Some(end) = end.as_int() else {
            return Err(Error::invalid_arguments(
                "`end` argument should be an Int",
                this.range(),
            ));
        };
        end
    } else {
        elements.len() as i64
    };

    let start = start as usize;
    let end = if end < 0 {
        // #todo supporting negative index may hide errors if the index is computed
        // #todo offer a link to only support negative values for constant index
        // If the end argument is negative it indexes from the end of the string.
        (elements.len() as i64 + end) as usize
    } else {
        end as usize
    };

    let slice = &elements[start..end];

    Ok(Expr::array(slice))
}

// #todo Consider different names: roll, rolled, rolling
// #todo Consider eager (roll) and lazy (rolling/rolled) versions.
// #todo Is this generic?
pub fn array_roll(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let window_size = unpack_int_arg(args, 0, "window-size")?;
    let items = unpack_array_arg(args, 1, "items")?;

    let windows = items.windows(window_size as usize);

    let mut rolled_items = Vec::new();
    for window in windows {
        rolled_items.push(Expr::array(window));
    }

    Ok(Expr::array(rolled_items))
}

pub fn setup_lib_seq(context: &mut Context) {
    // #todo should put in `seq` module and then into `prelude`.
    let module = require_module("prelude", context);

    // #todo introduce `++` overload?
    module.insert("cons", Expr::ForeignFunc(Arc::new(list_cons)));
    module.insert("count", Expr::ForeignFunc(Arc::new(list_count)));
    module.insert("count$$List", Expr::ForeignFunc(Arc::new(list_count)));

    // #todo add type qualifiers!
    module.insert("push", Expr::ForeignFunc(Arc::new(array_push)));
    // #todo also introduce `++`, `++=`, versions
    module.insert("concat!", Expr::ForeignFunc(Arc::new(array_concat_mut)));

    // (map (Func [x] (+ x 1)) [1 2 3]) ; => [2 3 4]
    // (map (Fn x (+ x 1)) [1 2 3]) ; => [2 3 4]
    // (map (-> x (+ x 1)) [1 2 3]) ; => [2 3 4]
    // (map \(+ % 1) [1 2 3])
    module.insert("map", Expr::ForeignFunc(Arc::new(array_map)));

    module.insert("join", Expr::ForeignFunc(Arc::new(array_join)));
    module.insert("skip", Expr::ForeignFunc(Arc::new(array_skip)));
    // #todo rename to (get-length) or something, match with String and other collection types.
    module.insert("count", Expr::ForeignFunc(Arc::new(array_count)));
    module.insert("count$$Array", Expr::ForeignFunc(Arc::new(array_count)));
    // #todo make contains? generic!
    module.insert("contains?", Expr::ForeignFunc(Arc::new(array_contains)));
    module.insert(
        "contains?$$Array$$Int",
        Expr::ForeignFunc(Arc::new(array_contains)),
    );
    module.insert(
        "contains?$$Array$$String",
        Expr::ForeignFunc(Arc::new(array_contains)),
    );
    module.insert("is-empty?", Expr::ForeignFunc(Arc::new(array_is_empty)));
    module.insert("sort!", Expr::ForeignFunc(Arc::new(array_sort_mut)));

    // #todo slice is to general works both as noun and verb, try to find an explicit verb? e.g. `cut` or `carve`
    // #todo alternatively use something like `get-slice` or `cut-slice` or `carve-slice`.
    // module.insert("slice", Expr::ForeignFunc(Arc::new(array_slice)));
    module.insert(
        "slice$$Array$$Int",
        Expr::ForeignFunc(Arc::new(array_slice)),
    );
    module.insert(
        "slice$$Array$$Int$$Int",
        Expr::ForeignFunc(Arc::new(array_slice)),
    );

    module.insert("roll", Expr::ForeignFunc(Arc::new(array_roll)));

    // let module = require_module("seq", context);

    // println!("--- YO!");
    // // #insight eval additional code implemented in Tan.
    // if let Err(errors) = eval_module("seq", context, true) {
    //     // #todo improve formatting here.
    //     eprintln!("{errors:?}");
    // }
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn list_cons_usage() {
        let input = r#"
            (let h 1)
            (let t '(2 3 4 5))
            (cons h t)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "(1 2 3 4 5)";
        assert_eq!(value, expected);
    }

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
    fn array_concat_mut_usage() {
        let input = r#"
            (let a [1 2])
            (let b [3 4])
            (concat! a b)
            a
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
    fn array_skip_usage() {
        let input = r#"
            (let arr [1 2 3 4 5])
            (skip arr 2)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "[3 4 5]";
        assert_eq!(value, expected);

        let input = r#"
            (let arr [1 2 3 4])
            (skip arr)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "[2 3 4]";
        assert_eq!(value, expected);
    }

    #[test]
    fn array_map_usage() {
        let mut context = Context::new();

        let input = r#"
            (let arr [1 2 3 4])
            (map (Func [x] (+ 5 x)) arr)
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "[6 7 8 9]";
        assert_eq!(value, expected);

        // #regression-test-case
        let input = r#"
            (let arr ['(a 1) '(a 2)])
            (map (Func [x] "*${x}") arr)
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = r#"["*(a 1)" "*(a 2)"]"#;
        assert_eq!(value, expected);

        // #regression-test-case
        let input = r#"
                (let my-func (Func [x] (+ 1 x)))
                (let arr [1 2 3])
                (map my-func arr)
            "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "[2 3 4]";
        assert_eq!(value, expected);

        // #todo introduce FuncMut?
        // #todo maybe (map ...) should emit warning for FuncMut/ForeignFuncMut

        let input = r#"
                (let arr [false true false])
                (map not arr)
            "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = "[true false true]";
        assert_eq!(value, expected);

        // #todo add more map tests.
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

    #[test]
    fn slice_usage() {
        let mut context = Context::new();
        let input = r#"
            (slice [1 2 3 4 5] 2)
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        assert_eq!(format_value(expr), "[3 4 5]");
    }
}
