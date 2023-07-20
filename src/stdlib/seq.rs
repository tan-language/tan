use crate::{
    context::Context,
    error::Error,
    expr::{format_value, Expr},
};

// #TODO support separator param.
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
