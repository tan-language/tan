use std::fmt;

use crate::range::Range;

// #TODO use `CompositeError` or similar to bundle multiple errors, and maintain consistent interface while avoiding allocations?

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
    MalformedInt,
    MalformedFloat,
    UnterminatedString,
    UnterminatedAnnotation,

    // Syntactic (parse) errors
    InvalidQuote,
    UnexpectedToken,
    UnterminatedList,
    MalformedAnnotation,

    // Semantic errors
    UndefinedSymbol(String), // #TODO maybe pass the whole Symbol expression?
    UndefinedFunction(String, String), // #TODO maybe pass the whole Symbol expression?
    InvalidArguments,
    NotInvocable, // #TODO maybe the non-invocable Annotated<Expr> should be the param?
    // #TODO better name needed.
    // #TODO is this a run-time error?
    FailedUse(String, Vec<Error>),

    // Runtime errors
    Io(std::io::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            ErrorKind::UnexpectedEnd => "unexpected end of input".to_owned(),
            ErrorKind::MalformedInt => "malformed integer number".to_owned(),
            ErrorKind::MalformedFloat => "malformed float number".to_owned(),
            ErrorKind::UnterminatedString => "unterminated string".to_owned(),
            ErrorKind::UnterminatedAnnotation => "unterminated annotation".to_owned(),
            ErrorKind::InvalidQuote => "invalid quote".to_owned(),
            ErrorKind::UnexpectedToken => format!("unexpected token"),
            ErrorKind::UnterminatedList => "unterminated list".to_owned(),
            ErrorKind::MalformedAnnotation => "malformed annotation".to_owned(),
            ErrorKind::UndefinedSymbol(sym) => format!("`{sym}` is undefined"),
            ErrorKind::UndefinedFunction(sym, signature) => {
                format!("function `{sym}` with signature `{signature}` is undefined")
            }
            ErrorKind::Io(io_err) => format!("i/o error: {io_err}"),
            ErrorKind::FailedUse(url, _) => format!("failed use `{url}`"),
            ErrorKind::InvalidArguments => "invalid arguments".to_owned(),
            ErrorKind::NotInvocable => "not invocable".to_owned(),
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

// #insight
// `note` is a synonym for `annotation`.

// #TODO add ErrorNoteKind?
#[derive(Debug)]
pub struct ErrorNote {
    /// The text of the note
    pub text: String,
    /// The range of the error, within the source.
    pub range: Option<Range>,
}

impl ErrorNote {
    pub fn new(text: &str, range: Option<Range>) -> Self {
        Self {
            text: text.to_owned(),
            range,
        }
    }
}

// #insight We keep the file url (instead of the module url) for more precise error reporting.

// #TODO find better pseudo-name.
const INPUT_PSEUDO_FILE_PATH: &str = "<input>";

#[derive(Debug)]
pub struct Error {
    /// The kind of the error.
    pub kind: ErrorKind,
    /// The source text where the error occurred. Typically this field is filled
    /// at a second stage.
    pub file_path: String,
    pub notes: Vec<ErrorNote>,
    pub hint: Option<String>,
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
            file_path: INPUT_PSEUDO_FILE_PATH.to_owned(),
            notes: Vec::new(),
            hint: None,
        }
    }

    pub fn invalid_arguments(note: &str, range: Option<Range>) -> Self {
        let mut error = Self::new(ErrorKind::InvalidArguments);
        error.push_note(note, range);
        error
    }

    pub fn undefined_function(
        symbol: &str,
        method: &str,
        note: &str,
        range: Option<Range>,
    ) -> Self {
        let mut error = Self::new(ErrorKind::UndefinedFunction(
            symbol.to_owned(),
            method.to_owned(),
        ));
        error.push_note(note, range);
        error
    }

    pub fn undefined_symbol(symbol: &str, note: &str, range: Option<Range>) -> Self {
        let mut error = Self::new(ErrorKind::UndefinedSymbol(symbol.to_owned()));
        error.push_note(note, range);
        error
    }

    pub fn not_invocable(note: &str, range: Option<Range>) -> Self {
        let mut error = Self::new(ErrorKind::NotInvocable);
        error.push_note(note, range);
        error
    }

    pub fn failed_use(url: &str, errors: Vec<Error>) -> Self {
        let error = Self::new(ErrorKind::FailedUse(url.to_owned(), errors));
        // #TODO url is _not_ the error.file_path, we need the caller module path.
        // error.file_path = url.to_owned();
        error
    }

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

    pub fn has_file_path(&self) -> bool {
        self.file_path != INPUT_PSEUDO_FILE_PATH
    }

    pub fn push_note(&mut self, note: &str, range: Option<Range>) {
        self.notes.push(ErrorNote::new(note, range));
    }

    pub fn range(&self) -> Option<&Range> {
        let note = self.notes.first();
        if let Some(note) = note {
            return note.range.as_ref();
        }
        None
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        let mut error = Error::new(ErrorKind::Io(value));
        // #TODO more detailed notes.
        error.push_note("I/O error: {value}", None);
        error
    }
}
