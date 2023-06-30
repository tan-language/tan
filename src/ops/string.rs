use crate::{error::Error, eval::env::Env, expr::Expr};

/// Returns a char iterable for the chars in the string.
pub fn string_chars(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments("`chars` requires `this` argument", None));
    };

    let Expr::String(this) = this.unpack() else {
        return Err(Error::invalid_arguments("`this` argument should be a String", this.range()));
    };

    let mut exprs: Vec<Expr> = Vec::new();

    for char in this.chars() {
        exprs.push(Expr::Char(char));
    }

    Ok(Expr::Array(exprs))
}

pub fn string_constructor_from_chars(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let [chars] = args else {
        return Err(Error::invalid_arguments("Requires `chars` argument", None));
    };

    let Expr::Array(exprs) = chars.unpack() else {
        return Err(Error::invalid_arguments("`chars` argument should be a (Array Char)", chars.range()));
    };

    let mut chars: Vec<char> = Vec::new();

    for expr in exprs {
        if let Expr::Char(c) = expr {
            chars.push(*c);
        }
    }

    let string = String::from_iter(chars);

    Ok(Expr::String(string))
}

// #TODO overload for string and char!

pub fn char_uppercased(args: &[Expr], _env: &Env) -> Result<Expr, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments("`uppercased` requires `this` argument", None));
    };

    let Expr::Char(this) = this.unpack() else {
        return Err(Error::invalid_arguments("`this` argument should be a Char", this.range()));
    };

    // #TODO omg...
    let uppercased = this.to_uppercase().next().unwrap();

    Ok(Expr::Char(uppercased))
}
