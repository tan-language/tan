use std::{error::Error, fmt};

#[derive(Debug)]
pub enum EvalError {
    UnknownError,
}

impl Error for EvalError {}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            EvalError::UnknownError => "unknown error".to_string(),
        };
        write!(f, "eval error: {}", err)
    }
}
