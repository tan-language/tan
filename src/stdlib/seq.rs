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

    // // #todo ARGH!
    // unsafe {
    //     // #todo the mother of all hacks!
    //     // #todo ARGH!!! interior mutability needed!
    //     let const_ptr = array as *const Vec<Expr>;
    //     println!("+++++ {const_ptr:?}");
    //     let mut_ptr = const_ptr as *mut Vec<Expr>;
    //     let array = &mut *mut_ptr;
    //     // dbg!(&array);
    //     array.push(element.unpack().clone());
    //     // dbg!(&element, &array);
    // }
    // // dbg!(&element, &array);

    elements.push(element.unpack().clone()); // #todo hmm this clone!

    // #todo what to return?
    Ok(Expr::One)
}

// #todo support separator param.
pub fn array_join(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [array, ..] = args else {
        return Err(Error::invalid_arguments("requires `array` argument", None));
    };

    let Some(array) = array.as_array() else {
        return Err(Error::invalid_arguments(
            "`array` argument should be a Array",
            array.range(),
        ));
    };

    let elements: Vec<String> = array.iter().map(|e| format_value(e)).collect();

    Ok(Expr::String(elements.join("")))
}

// #insight use the word Iterable instead of Sequence/Seq, more generic (can handle non-sequences, e.g. maps)
// #insight could also use Countable

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

// #todo implement first, last
