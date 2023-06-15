use std::fmt;

use crate::range::Range;

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
// No need to associate the kind with the lexeme.

// #TODO support #quot annotation?

// #TODO do we ever need a non-ranged Token? should consider Token/TokenKind arrangement?

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
    MultiLineWhitespace, // #TODO use something more general, like `Pragma`.
    // Char(char),
    String,
    Symbol,
    Number,
    Annotation,
    Comment,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #TODO optimize this!
        // #TODO reconsider how tokens are displayed.
        f.write_str(match self {
            TokenKind::LeftParen => "(",
            TokenKind::RightParen => ")",
            TokenKind::LeftBracket => "[",
            TokenKind::RightBracket => "]",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::Quote => "'",
            TokenKind::String => format!("String").as_str(),
            TokenKind::Symbol => format!("Symbol").as_str(),
            TokenKind::Number => format!("Number").as_str(),
            TokenKind::Annotation => format!("Annotation").as_str(),
            TokenKind::Comment => format!("Comment").as_str(),
            TokenKind::MultiLineWhitespace => "MultiLineWhitespace", // #TODO what should we do here? #Idea convert to comment?
        })
    }
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    range: Range,
    lexeme: Option<String>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Token {
    pub fn new(kind: TokenKind, range: Range, lexeme: Option<String>) -> Self {
        Self {
            kind,
            range,
            lexeme,
        }
    }

    pub fn from_kind(kind: TokenKind, range: Range) -> Self {
        Self {
            kind,
            range,
            lexeme: None,
        }
    }

    pub fn string(lexeme: String, range: Range) -> Self {
        Self {
            kind: TokenKind::String,
            range,
            lexeme: Some(lexeme),
        }
    }

    pub fn symbol(lexeme: String, range: Range) -> Self {
        Self {
            kind: TokenKind::Symbol,
            range,
            lexeme: Some(lexeme),
        }
    }

    pub fn number(lexeme: String, range: Range) -> Self {
        Self {
            kind: TokenKind::Number,
            range,
            lexeme: Some(lexeme),
        }
    }

    pub fn annotation(lexeme: String, range: Range) -> Self {
        Self {
            kind: TokenKind::Annotation,
            range,
            lexeme: Some(lexeme),
        }
    }

    pub fn comment(lexeme: String, range: Range) -> Self {
        Self {
            kind: TokenKind::Comment,
            range,
            lexeme: Some(lexeme),
        }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn range(&self) -> Range {
        self.range
    }

    pub fn lexeme(&self) -> String {
        if let Some(lexeme) = self.lexeme {
            lexeme
        } else {
            self.kind.to_string()
        }
    }
}
