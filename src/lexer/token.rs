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

// #TODO support #quot annotation?
// #TODO only have a general Numeric token?

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
    Int(i64),
    Float(f64),
    String(String),
    Symbol(String),
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
                Token::Int(n) => n.to_string(),
                Token::Float(n) => n.to_string(),
                Token::String(s) => s.clone(), // #TODO should show the delimiters?
                Token::Symbol(s) => s.clone(),
                Token::Annotation(s) => s.clone(),
                Token::Comment(s) => s.clone(),
            })
            .as_str(),
        )
    }
}
