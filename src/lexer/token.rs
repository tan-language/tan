// #Insight
// There is no need for an EOF Token. The end of the Token list marks the end
// of the input.

use std::fmt;

/// A lexical Token gives semantic meaning to a Lexeme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Number(i64),
    String(String),
    Symbol(String),
    If,
    Using,
    Annotation(String),
    // True,
    // False,
    Comment(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #TODO optimize this!
        f.write_str(
            (match self {
                Token::LParen => "(".to_owned(),
                Token::RParen => ")".to_owned(),
                Token::LBracket => "[".to_owned(),
                Token::RBracket => "]".to_owned(),
                Token::LBrace => "{".to_owned(),
                Token::RBrace => "}".to_owned(),
                Token::Number(n) => format!("{}", n),
                Token::String(s) => s.clone(),
                Token::Symbol(s) => s.clone(),
                Token::If => "if".to_owned(),
                Token::Using => "using".to_owned(),
                Token::Annotation(s) => s.clone(),
                Token::Comment(s) => s.clone(),
            })
            .as_str(),
        )
    }
}
