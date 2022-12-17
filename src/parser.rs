use crate::{lexer::token::Token, span::Spanned};

use self::{error::ParseError, expr::Expr};

pub mod error;
pub mod expr;

pub struct Parser<'a> {
    tokens: &'a [Spanned<Token>],
    index: usize,
    // #TODO use stack to support 'unlimited' lookahead?
    lookahead: Option<&'a Spanned<Token>>, // #TODO find better name!
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Spanned<Token>]) -> Self {
        Self {
            tokens,
            index: 0,
            lookahead: None,
        }
    }

    // #TODO peek_token() (next+put_back)

    fn next_token(&mut self) -> Option<&'a Spanned<Token>> {
        if let Some(ch) = self.lookahead {
            self.lookahead = None;
            return Some(ch);
        }

        // #TODO iterator
        let token = self.tokens.get(self.index);
        self.index += 1;
        token
    }

    fn put_back_token(&mut self, token: &'a Spanned<Token>) {
        self.lookahead = Some(token);
        self.index -= 1;
    }

    pub fn parse_atom() {
        todo!()
    }

    pub fn parse_list() {
        todo!()
    }

    // #TODO returns AST
    pub fn parse(&mut self) -> Result<Expr, Spanned<ParseError>> {
        let mut ast = Expr::List(Vec::new());

        let mut token = self.next_token();

        while let Some(st) = token {
            let Spanned { value: t, span } = st;
            match t {
                Token::Comment(..) => (),
                Token::LParen => (),
                Token::RParen => (),
                _ => print!("Unhandled"),
            }

            token = self.next_token();
        }

        Ok(ast)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{token::Token, Lexer},
        parser::expr::Expr,
        span::Spanned,
    };

    use super::Parser;

    fn read_tokens(filename: &str) -> Vec<Spanned<Token>> {
        let input = std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap();
        let mut lexer = Lexer::new(&input);
        lexer.lex().unwrap()
    }

    #[test]
    fn parse_handles_an_empty_token_list() {
        let tokens = read_tokens("empty.tan");
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, Expr::List(x) if x.is_empty()));
    }
}
