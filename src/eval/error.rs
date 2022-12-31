use std::{error::Error, fmt};

// #TODO Use Expr for errors!! enjoy the generality/flexibility!

#[derive(Debug)]
pub enum EvalError {
    UndefinedSymbol(String), // #TODO maybe pass the whole Symbol expression?
    Io(std::io::Error),
    InvalidArguments(String),
    NotInvocable(String), // #TODO maybe the non-invocable Annotated<Expr> should be the param?
    Unknown,              // #TODO remove this!
}

impl Error for EvalError {}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            EvalError::UndefinedSymbol(sym) => format!("`{sym}` is undefined"),
            EvalError::Io(io_err) => format!("i/o error: {io_err}"),
            EvalError::InvalidArguments(text) => text.to_owned(),
            EvalError::NotInvocable(text) => text.to_owned(),
            EvalError::Unknown => "unknown error".to_owned(),
        };
        write!(f, "eval error: {}", err)
    }
}

impl From<std::io::Error> for EvalError {
    fn from(value: std::io::Error) -> Self {
        EvalError::Io(value)
    }
}
