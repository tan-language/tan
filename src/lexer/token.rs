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
    // Char(char),
    String(String),
    Symbol(String),
    Number(String),
    Annotation(String),
    Comment(String),
    /// MultiLineWhitespace tokens are leveraged by the formatter to maintain
    /// 'paragraphs' of text.
    MultiLineWhitespace, // #TODO use something more general, like `Pragma`.
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
            TokenKind::String(lexeme) => format!("String({lexeme})").as_str(),
            TokenKind::Symbol(lexeme) => format!("Symbol({lexeme})").as_str(),
            TokenKind::Number(lexeme) => format!("Number({lexeme})").as_str(),
            TokenKind::Annotation(lexeme) => format!("Annotation({lexeme})").as_str(),
            TokenKind::Comment(lexeme) => format!("Comment({lexeme})").as_str(),
            TokenKind::MultiLineWhitespace => "MultiLineWhitespace", // #TODO what should we do here? #Idea convert to comment?
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
}
