use std::fmt;

use crate::range::Range;

use super::comment::CommentKind;

// #insight
// There is no need for an EOF Token. The end of the Token list marks the end
// of the input.

// #insight
// It's not worth it to have token variants for reserved words.
// The is a benefit, to differentiate reserved words from other symbols, but
// we will do it in another way.

// #insight
// Reserved _word_ is a bad name because it can be more than one word.

// #insight
// A general Number token is used, classification is postponed to a later stage.

// #insight
// Tan intentionally doesn't provide a Char literal, as chars are not used that
// often to deserve a dedicated sigil.

// #insight
// No need to associate the kind with the lexeme, but we do it, for consistency with Error.

// #todo support #quot annotation?

// #todo do we ever need a non-ranged Token? should consider Token/TokenKind arrangement?

// #insight
// Ranges are represented as a Symbol token, will be handled during syntactical analysis.

/// A lexical Token gives semantic meaning to a Lexeme.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Quote,
    /// MultiLineWhitespace tokens are leveraged by the formatter to maintain
    /// 'paragraphs' of text.
    MultiLineWhitespace, // #todo use something more general, like `Pragma`.
    // Char(char),
    String(String),
    Symbol(String),
    Number(String),
    Annotation(String),
    Comment(String, CommentKind),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #todo optimize this!
        // #todo reconsider how tokens are displayed.
        f.write_str(match self {
            TokenKind::LeftParen => "(",
            TokenKind::RightParen => ")",
            TokenKind::LeftBracket => "[",
            TokenKind::RightBracket => "]",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::Quote => "'",
            TokenKind::String(lexeme) => lexeme,
            TokenKind::Symbol(lexeme) => lexeme,
            TokenKind::Number(lexeme) => lexeme,
            TokenKind::Annotation(lexeme) => lexeme,
            TokenKind::Comment(lexeme, _) => lexeme,
            TokenKind::MultiLineWhitespace => "MultiLineWhitespace", // #todo what should we do here? #Idea convert to comment?
        })
    }
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    range: Range,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Token {
    pub fn new(kind: TokenKind, range: Range) -> Self {
        Self { kind, range }
    }

    pub fn string(lexeme: String, range: Range) -> Self {
        Self {
            kind: TokenKind::String(lexeme),
            range,
        }
    }

    pub fn symbol(lexeme: String, range: Range) -> Self {
        Self {
            kind: TokenKind::Symbol(lexeme),
            range,
        }
    }

    pub fn number(lexeme: String, range: Range) -> Self {
        Self {
            kind: TokenKind::Number(lexeme),
            range,
        }
    }

    pub fn annotation(lexeme: String, range: Range) -> Self {
        Self {
            kind: TokenKind::Annotation(lexeme),
            range,
        }
    }

    pub fn comment(lexeme: String, range: Range, comment_kind: CommentKind) -> Self {
        Self {
            kind: TokenKind::Comment(lexeme, comment_kind),
            range,
        }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn range(&self) -> Range {
        self.range.clone()
    }

    pub fn lexeme(&self) -> String {
        // match &self.kind {
        //     TokenKind::Symbol(lexeme) => lexeme,
        //     TokenKind::Number(lexeme) => lexeme,
        //     TokenKind::Annotation(lexeme) => lexeme,
        //     TokenKind::Comment(lexeme) => lexeme,
        //     _ => self.kind.to_string(),
        // }

        self.kind.to_string()
    }
}
