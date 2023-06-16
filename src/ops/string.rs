use crate::{ann::Ann, error::Error, eval::env::Env, expr::Expr};

/// Returns a char iterable for the chars in the string.
pub fn string_chars(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments("`chars` requires `this` argument").into());
    };

    let Ann(Expr::String(this), ..) = this else {
        return Err(Error::invalid_arguments("`this` argument should be a String").into());
    };

    let mut exprs: Vec<Expr> = Vec::new();

    for char in this.chars() {
        exprs.push(Expr::Char(char));
    }

    Ok(Expr::Array(exprs).into())
}

pub fn string_constructor_from_chars(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Error> {
    let [chars] = args else {
        return Err(Error::invalid_arguments("Requires `chars` argument").into());
    };

    let Ann(Expr::Array(exprs), ..) = chars else {
        return Err(Error::invalid_arguments("`chars` argument should be a (Array Char)").into());
    };

    let mut chars: Vec<char> = Vec::new();

    for expr in exprs {
        if let Expr::Char(c) = expr {
            chars.push(*c);
        }
    }

    let string = String::from_iter(chars);

    Ok(Expr::String(string).into())
}

// #TODO overload for string and char!

pub fn char_uppercased(args: &[Ann<Expr>], _env: &Env) -> Result<Ann<Expr>, Ranged<Error>> {
    let [this] = args else {
        return Err(Error::invalid_arguments("`uppercased` requires `this` argument").into());
    };

    let Ann(Expr::Char(this), ..) = this else {
        return Err(Error::invalid_arguments("`this` argument should be a Char").into());
    };

    // #TODO omg...
    let uppercased = this.to_uppercase().next().unwrap();

    Ok(Expr::Char(uppercased).into())
}
