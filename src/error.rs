use std::fmt;

use crate::{
    context::Context, eval::util::get_current_file_path, expr::Expr, range::Range,
    util::constants::INPUT_PSEUDO_FILE_PATH,
};

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

// #todo consider renaming back Variant -> Kind.
#[derive(Debug)]
pub enum ErrorVariant {
    // Lexical errors
    UnexpectedEnd,
    MalformedInt,
    MalformedFloat,
    MalformedEscapeCode,
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
    PoisonedLock,    // #todo find a better name!
    General(String), // #todo find a better name!

    // Panic
    Panic(String),

    // Control-Flow
    // #todo try to avoid having return, break, continue in the language
    // #todo or at least add linter warnings to avoid them
    // #insight return, break, continue can improve readability though

    // #todo #hack this is a temp hackish solution!
    // #todo the Error is abused, maybe should use Exprs instead?
    // #todo consider using named return value(s) instead of the return keyword? (assignment is implicit return?)
    // #todo add custom reporting if used outside of a function (not catched in eval_func).
    // Returned by ..return
    ReturnCF(Expr),
    // #todo add custom reporting if used outside of a loop (not catched in eval_for)
    // Signals a break statement in a loop.
    BreakCF(Expr),
    // #todo add custom reporting if used outside of a loop (not catched in eval_for)
    // Signals a continue statement in a loop.
    ContinueCF,
    // GotoCF(Expr),
}

impl fmt::Display for ErrorVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            ErrorVariant::UnexpectedEnd => "unexpected end of input".to_owned(),
            ErrorVariant::MalformedInt => "malformed integer number".to_owned(),
            ErrorVariant::MalformedFloat => "malformed float number".to_owned(),
            ErrorVariant::MalformedEscapeCode => "malformed escape code".to_owned(),
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
            ErrorVariant::PoisonedLock => "poisoned lock".to_owned(),
            ErrorVariant::NotInvocable => "not invocable".to_owned(),
            ErrorVariant::General(text) => text.clone(),
            ErrorVariant::Panic(_) => "panic".to_owned(),
            ErrorVariant::ReturnCF(_) => "return".to_owned(),
            ErrorVariant::ContinueCF => "continue".to_owned(),
            ErrorVariant::BreakCF(_) => "break".to_owned(),
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
    pub fn new(variant: ErrorVariant) -> Self {
        Self {
            variant,
            file_path: INPUT_PSEUDO_FILE_PATH.to_owned(),
            notes: Vec::new(),
        }
    }

    pub fn invalid_arguments(note: &str, range: Option<Range>) -> Self {
        let mut error = Self::new(ErrorVariant::InvalidArguments);
        error.push_note(note, range);
        error
    }

    pub fn poisoned_lock(note: &str, range: Option<Range>) -> Self {
        let mut error = Self::new(ErrorVariant::PoisonedLock);
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

    pub fn io(io_error: std::io::Error, note: &str, range: Option<Range>) -> Self {
        let mut error = Self::new(ErrorVariant::Io(io_error));
        error.push_note(note, range);
        error
    }

    // #todo make errors Option.
    pub fn failed_use(url: &str, source_errors: Vec<Error>) -> Self {
        // #todo formatting of the error should not happen here.
        // #todo needs two steps: and abstract formatter that creates notes etc and multiple renderers: text, colored, html, etc.

        // #todo url is _not_ the error.file_path, we need the caller module path.
        // error.file_path = url.to_owned();
        let mut notes = Vec::new();
        for e in &source_errors {
            // #todo should also include source range!!
            notes.push(ErrorNote::new(&e.to_string(), e.range().cloned()));
        }
        let mut error = Self::new(ErrorVariant::FailedUse(url.to_owned(), source_errors));
        error.notes = notes;
        error
    }

    // #todo add range!
    // placeholder error!
    pub fn general(text: &str) -> Self {
        Self::new(ErrorVariant::General(text.to_owned()))
    }

    pub fn return_cf(value: Expr) -> Self {
        Self::new(ErrorVariant::ReturnCF(value))
    }

    pub fn break_cf(value: Expr) -> Self {
        Self::new(ErrorVariant::BreakCF(value))
    }

    pub fn continue_cf() -> Self {
        Self::new(ErrorVariant::ContinueCF)
    }

    #[deprecated]
    pub fn panic(text: &str) -> Self {
        Self::new(ErrorVariant::Panic(text.to_owned()))
    }

    // #todo could also take a range!
    pub fn panic_with_context(text: &str, context: &Context) -> Self {
        // #todo encode location.

        let mut error = Self {
            variant: crate::error::ErrorVariant::Panic(text.to_string()),
            file_path: get_current_file_path(context),
            notes: vec![],
        };

        error.push_note(text, None);

        error
    }

    pub fn variant(&self) -> &ErrorVariant {
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

    pub fn is_panic(&self) -> bool {
        matches!(self.variant, ErrorVariant::Panic(..))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        // #todo more detailed notes.
        // error.push_note(&format!("I/O error: {value:?}"), None);
        Error::new(ErrorVariant::Io(value))
    }
}
