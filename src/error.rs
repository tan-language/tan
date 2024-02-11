use std::fmt;

use crate::range::Range;

// #todo emit the correct file of the error!

// #todo use `CompositeError` or similar to bundle multiple errors, and maintain consistent interface while avoiding allocations?

// #insight The implementation is based on https://doc.rust-lang.org/std/io/struct.Error.html

// #insight It's useful to have different note texts for the same ErrorKind.

// #todo UnexpectedEnd is similar to UnterminatedString/Annotation, could reuse the message?

// #todo keep the error formatting fragments in some reusable form.
// #todo examples of good errors: https://jvns.ca/blog/2022/12/02/a-couple-of-rust-error-messages/

// #todo Split comptime/runtime errors?

// #todo lexer, parser, resolver, etc should be able to return multiple errors
// #todo maybe just use _one_ Error?
// #todo think about how to handle Ranged
// #todo maybe use Ann instead of Ranged?
// #todo maybe use Expr for the errors?

// #insight Eval always returns one error, actually no it can call read/parse/ that can return many errors!

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
// For more information about this error, try `cc --explain E0716`.
// error: could not compile `playground` (bin "playground") due to previous error

#[derive(Debug)]
pub enum ErrorVariant {
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
    MalformedRange,
    MalformedStringTemplate, // #todo better name?

    // Semantic errors
    UndefinedSymbol(String), // #todo maybe pass the whole Symbol expression?
    UndefinedFunction(String, String), // #todo maybe pass the whole Symbol expression?
    InvalidArguments,
    NotInvocable, // #todo maybe the non-invocable Annotated<Expr> should be the param?
    // #todo better name needed.
    // #todo is this a run-time error?
    FailedUse(String, Vec<Error>),

    // Runtime errors
    Io(std::io::Error),
    General(String), // #todo find a better name!
}

impl fmt::Display for ErrorVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            ErrorVariant::UnexpectedEnd => "unexpected end of input".to_owned(),
            ErrorVariant::MalformedInt => "malformed integer number".to_owned(),
            ErrorVariant::MalformedFloat => "malformed float number".to_owned(),
            ErrorVariant::UnterminatedString => "unterminated string".to_owned(),
            ErrorVariant::UnterminatedAnnotation => "unterminated annotation".to_owned(),
            ErrorVariant::InvalidQuote => "invalid quote".to_owned(),
            ErrorVariant::UnexpectedToken => "unexpected token".to_owned(),
            ErrorVariant::UnterminatedList => "unterminated list".to_owned(),
            ErrorVariant::MalformedAnnotation => "malformed annotation".to_owned(),
            ErrorVariant::MalformedRange => "malformed range".to_owned(),
            ErrorVariant::MalformedStringTemplate => "malformed string template".to_owned(),
            ErrorVariant::UndefinedSymbol(sym) => format!("`{sym}` is undefined"),
            ErrorVariant::UndefinedFunction(sym, signature) => {
                format!("function `{sym}` with signature `{signature}` is undefined")
            }
            ErrorVariant::Io(io_err) => format!("i/o error: {io_err}"),
            ErrorVariant::FailedUse(url, _) => format!("failed use `{url}`"),
            ErrorVariant::InvalidArguments => "invalid arguments".to_owned(),
            ErrorVariant::NotInvocable => "not invocable".to_owned(),
            ErrorVariant::General(text) => text.clone(),
        };

        write!(f, "{err}")
    }
}

impl ErrorVariant {
    // #insight
    // We could use a derive macro to generate those, but being explicit is
    // more readable.
    pub fn code(&self) -> u32 {
        // #todo implement me!
        0
    }
}

// #insight
// `note` is a synonym for `annotation`.

#[derive(Eq, PartialEq, Debug)]
pub enum ErrorNoteVariant {
    Note,
    Help,
    Hint,
}

// #todo add ErrorNoteKind?
// #todo keep the source expr instead of range.
#[derive(Debug)]
pub struct ErrorNote {
    /// The variant (type) of the note.
    pub variant: ErrorNoteVariant,
    /// The text of the note
    pub text: String,
    /// The range of the error, within the source.
    pub range: Option<Range>,
}

impl ErrorNote {
    pub fn new(text: &str, range: Option<Range>) -> Self {
        Self {
            variant: ErrorNoteVariant::Note,
            text: text.to_owned(),
            range,
        }
    }
}

// #insight We keep the file url (instead of the module url) for more precise error reporting.

// #todo move this to another place.
// #todo find better pseudo-name.
const INPUT_PSEUDO_FILE_PATH: &str = "<input>";

// #todo keep source expression instead of file_path/range.
// #todo allow for multiple hints.
// #todo support child/sub errors.

// #insight
// This is a general error, used by all the stages of the compiler.

#[derive(Debug)]
pub struct Error {
    /// The variant of the error.
    pub variant: ErrorVariant,
    /// The source text where the error occurred. Typically this field is filled
    /// at a second stage.
    pub file_path: String,
    pub notes: Vec<ErrorNote>,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #todo write more information!
        write!(f, "{}", self.variant)
    }
}

impl Error {
    pub fn new(kind: ErrorVariant) -> Self {
        Self {
            variant: kind,
            file_path: INPUT_PSEUDO_FILE_PATH.to_owned(),
            notes: Vec::new(),
        }
    }

    pub fn invalid_arguments(note: &str, range: Option<Range>) -> Self {
        let mut error = Self::new(ErrorVariant::InvalidArguments);
        error.push_note(note, range);
        error
    }

    pub fn undefined_function(
        symbol: &str,
        method: &str,
        note: &str,
        range: Option<Range>,
    ) -> Self {
        let mut error = Self::new(ErrorVariant::UndefinedFunction(
            symbol.to_owned(),
            method.to_owned(),
        ));
        error.push_note(note, range);
        error
    }

    pub fn undefined_symbol(symbol: &str, note: &str, range: Option<Range>) -> Self {
        let mut error = Self::new(ErrorVariant::UndefinedSymbol(symbol.to_owned()));
        error.push_note(note, range);
        error
    }

    pub fn not_invocable(note: &str, range: Option<Range>) -> Self {
        let mut error = Self::new(ErrorVariant::NotInvocable);
        error.push_note(note, range);
        error
    }

    // #todo make errors Option.
    pub fn failed_use(url: &str, errors: Vec<Error>) -> Self {
        // #todo url is _not_ the error.file_path, we need the caller module path.
        // error.file_path = url.to_owned();
        Self::new(ErrorVariant::FailedUse(url.to_owned(), errors))
    }

    // placeholder error!
    pub fn general(text: &str) -> Self {
        Self::new(ErrorVariant::General(text.to_owned()))
    }

    pub fn kind(&self) -> &ErrorVariant {
        &self.variant
    }

    pub fn code(&self) -> u32 {
        self.variant.code()
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
        // #todo more detailed notes.
        // error.push_note(&format!("I/O error: {value:?}"), None);
        Error::new(ErrorVariant::Io(value))
    }
}
