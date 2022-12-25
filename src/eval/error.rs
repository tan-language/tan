use std::{error::Error, fmt};

#[derive(Debug)]
pub enum EvalError {
    UndefinedSymbol(String),
    IoError(std::io::Error),
    UnknownError,
}

impl Error for EvalError {}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            EvalError::UndefinedSymbol(sym) => format!("`{sym}` is undefined"),
            EvalError::IoError(io_err) => format!("I/O error: {io_err}"),
            EvalError::UnknownError => "unknown error".to_string(),
        };
        write!(f, "eval error: {}", err)
    }
}

impl From<std::io::Error> for EvalError {
    fn from(value: std::io::Error) -> Self {
        EvalError::IoError(value)
    }
}
