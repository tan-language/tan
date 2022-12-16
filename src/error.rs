use std::error::Error;

use crate::{lexer::LexicalError, spanned::Spanned};

impl Error for Spanned<LexicalError> {}
