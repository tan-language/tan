use std::fmt;

// #Insight
// There is no need for an EOF Token. The end of the Token list marks the end
// of the input.

// #Insight
// It's not worth it to have token variants for reserved words.
// The is a benefit, to differentiate reserved words from other symbols, but
// we will do it in another way.

// #Insight
// Reserved _word_ is a bad name because it can be more than one word.

// #Insight
// A general Number token is used, classification is postponed to a later stage.

// #Insight
// Tan intentionally doesn't provide a Char literal, as chars are not used that
// often to deserve a dedicated sigil.

// #TODO support #quot annotation?

/// A lexical Token gives semantic meaning to a Lexeme.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
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
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #TODO optimize this!
        // #TODO reconsider how tokens are displayed.
        f.write_str(
            (match self {
                Token::LeftParen => "(".to_owned(),
                Token::RightParen => ")".to_owned(),
                Token::LeftBracket => "[".to_owned(),
                Token::RightBracket => "]".to_owned(),
                Token::LeftBrace => "{".to_owned(),
                Token::RightBrace => "}".to_owned(),
                Token::Quote => "'".to_owned(),
                // Token::Char(c) => c.to_string(), // #TODO should show the delimiters?
                Token::String(s) => s.clone(), // #TODO should show the delimiters?
                Token::Symbol(s) => s.clone(),
                Token::Number(s) => s.clone(),
                Token::Annotation(s) => s.clone(),
                Token::Comment(s) => s.clone(),
            })
            .as_str(),
        )
    }
}
