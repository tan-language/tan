use std::{error::Error, fmt};

#[derive(Debug)]
pub enum EvalError {
    UndefinedSymbolError(String), // #TODO maybe pass the whole Symbol expression?
    IoError(std::io::Error),
    ArgumentError(String),
    NotInvocableError(String), // #TODO maybe the non-invocable Annotated<Expr> should be the param?
    UnknownError,
}

impl Error for EvalError {}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            EvalError::UndefinedSymbolError(sym) => format!("`{sym}` is undefined"),
            EvalError::IoError(io_err) => format!("i/o error: {io_err}"),
            EvalError::ArgumentError(text) => text.to_owned(),
            EvalError::NotInvocableError(text) => text.to_owned(),
            EvalError::UnknownError => "unknown error".to_owned(),
        };
        write!(f, "eval error: {}", err)
    }
}

impl From<std::io::Error> for EvalError {
    fn from(value: std::io::Error) -> Self {
        EvalError::IoError(value)
    }
}
