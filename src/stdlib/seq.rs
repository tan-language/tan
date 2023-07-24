use crate::{
    context::Context,
    error::Error,
    expr::{format_value, Expr},
};

// #todo implement generically for all iterables/countables, etc.

// #todo support separator param.
pub fn array_join(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [array, ..] = args else {
        return Err(Error::invalid_arguments("requires `array` argument", None));
    };

    let Some(array) = array.as_array() else {
        return Err(Error::invalid_arguments("`array` argument should be a Array", array.range()));
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
        return Err(Error::invalid_arguments("`array` argument should be a Array", array.range()));
    };

    Ok(Expr::Int(array.len() as i64))
}

// #todo implement first, last
