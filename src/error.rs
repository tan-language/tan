use std::{
    fmt,
    num::{ParseFloatError, ParseIntError},
};

use crate::{lexer::token::Token, range::Range};

// #insight The implementation is based on https://doc.rust-lang.org/std/io/struct.Error.html

// #insight It's useful to have different note texts for the same ErrorKind.

// #TODO UnexpectedEnd is similar to UnterminatedString/Annotation, could reuse the message?

// #TODO keep the error formatting fragments in some reusable form.
// #TODO examples of good errors: https://jvns.ca/blog/2022/12/02/a-couple-of-rust-error-messages/

// #TODO Split comptime/runtime errors?

// #TODO lexer, parser, resolver, etc should be able to return multiple errors
// #TODO maybe just use _one_ Error?
// #TODO think about how to handle Ranged
// #TODO maybe use Ann instead of Ranged?
// #TODO maybe use Expr for the errors?

// #Insight Eval always returns one error, actually no it can call read/parse/ that can return many errors!

// Example of a good error:
//
// error[E0716]: temporary value dropped while borrowed
//  --> src/main.rs:5:18
//   |
// 5 |     let scores = inputs().iter().map(|(a, b)| {
//   |                  ^^^^^^^^ creates a temporary value which is freed while still in use
// 6 |         a + b
// 7 |     });
//   |       - temporary value is freed at the end of this statement
// 8 |     println!("{}", scores.sum::<i32>());
//   |                    ------ borrow later used here
//   |
// help: consider using a `let` binding to create a longer lived value
//   |
// 5 ~     let binding = inputs();
// 6 ~     let scores = binding.iter().map(|(a, b)| {
//   |
//
// For more information about this error, try `rustc --explain E0716`.
// error: could not compile `playground` (bin "playground") due to previous error

#[derive(Debug)]
pub enum ErrorKind {
    // Lexical errors
    UnexpectedEnd,
    MalformedInt(ParseIntError),
    MalformedFloat(ParseFloatError),
    UnterminatedString,
    UnterminatedAnnotation,

    // Syntactic (parse) errors
    InvalidQuote,
    UnexpectedToken(Token),
    UnterminatedList,
    MalformedAnnotation(String),

    // Semantic errors
    UndefinedSymbol(String), // #TODO maybe pass the whole Symbol expression?
    UndefinedFunction(String, String), // #TODO maybe pass the whole Symbol expression?
    InvalidArguments(String),
    NotInvocable(String), // #TODO maybe the non-invocable Annotated<Expr> should be the param?
    FailedUse,            // #TODO temp, better name needed, rethink!

    // Runtime errors
    Io(std::io::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            ErrorKind::UnexpectedEnd => "unexpected end of input".to_owned(),
            ErrorKind::MalformedInt(pie) => format!("malformed integer number: {pie}"),
            ErrorKind::MalformedFloat(pie) => format!("malformed float number: {pie}"),
            ErrorKind::UnterminatedString => "unterminated string".to_owned(),
            ErrorKind::UnterminatedAnnotation => "unterminated annotation".to_owned(),
            ErrorKind::InvalidQuote => "invalid quote".to_owned(),
            ErrorKind::UnexpectedToken(token) => format!("unexpected `{token}`"),
            ErrorKind::UnterminatedList => "unterminated list".to_owned(),
            ErrorKind::MalformedAnnotation(ann) => format!("malformed annotation `{ann}`"),
            ErrorKind::UndefinedSymbol(sym) => format!("`{sym}` is undefined"),
            ErrorKind::UndefinedFunction(sym, signature) => {
                format!("function `{sym}` with signature `{signature}` is undefined")
            }
            ErrorKind::Io(io_err) => format!("i/o error: {io_err}"),
            ErrorKind::FailedUse => "failed use".to_owned(),
            ErrorKind::InvalidArguments(text) => text.to_owned(),
            ErrorKind::NotInvocable(text) => text.to_owned(),
        };

        write!(f, "{err}")
    }
}

impl ErrorKind {
    // #insight
    // We could use a derive macro to generate those, but being explicit is
    // more readable.
    pub fn code(&self) -> u32 {
        // #TODO implement me!
        0
    }
}

// #TODO add ErrorNoteKind?
// #TODO consider using Ranged<String> instead?
// #TODO think of a better name.
#[derive(Debug)]
pub struct ErrorNote {
    /// The text of the note
    note: String,
    /// The range of the error, within the source.
    range: Option<Range>,
}

impl ErrorNote {
    pub fn new(note: &str, range: Option<Range>) -> Self {
        Self {
            note: note.to_owned(),
            range,
        }
    }
}

// #insight We keep the file url (instead of the module url) for more precise error reporting.

#[derive(Debug)]
pub struct Error {
    /// The kind of the error.
    kind: ErrorKind,
    /// The source text where the error occurred. Typically this field is filled
    /// at a second stage.
    pub file_url: Option<String>,
    pub notes: Vec<ErrorNote>,
    // #TODO leave hints for a future revision.
    // hint: Option<String>,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #TODO write more information!
        write!(f, "{}", self.kind)
    }
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            file_url: None,
            notes: Vec::new(),
        }
    }

    // #TODO add helper constructors.

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn code(&self) -> u32 {
        self.kind.code()
    }

    // pub fn file_url(&self) -> &String {
    //     &self.file_url
    // }

    // pub fn notes(&self) -> &[ErrorNote] {
    //     &self.notes
    // }

    pub fn push_note(&mut self, note: &str, range: Option<Range>) {
        self.notes.push(ErrorNote::new(note, range));
    }
}

// impl From<std::io::Error> for Error {
//     fn from(value: std::io::Error) -> Self {
//         Error::new(ErrorKind::Io(value), None, None)
//     }
// }
