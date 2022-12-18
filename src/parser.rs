use crate::{lexer::token::Token, span::Spanned};

use self::{error::ParseError, expr::Expr};

pub mod error;
pub mod expr;

// #TODO consider moving the tokens into the parser (tokens are useless outside of parser)

/// The Parser performs the syntax analysis stage of the compilation pipeline.
/// The input token stream is reduced into and Abstract Syntax Tree (AST).
/// The nodes of the AST are associated with annotations.
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

    // fn put_back_token(&mut self, token: &'a Spanned<Token>) {
    //     self.lookahead = Some(token);
    //     self.index -= 1;
    // }

    // #TODO better name!
    // #TODO this could be parse if we automatically add pseudo `(`, `)` tokens.
    pub fn parse_tokens(
        &mut self,
        exprs: Vec<Expr>,
        in_list: bool,
    ) -> Result<Vec<Expr>, Spanned<ParseError>> {
        let mut exprs = exprs;

        let mut token: Option<&Spanned<Token>>;

        loop {
            token = self.next_token();

            let Some(st) = token  else {
                break;
            };

            let Spanned { value: t, span } = st;

            match t {
                Token::Comment(..) => (),
                Token::String(s) => exprs.push(Expr::String(s.clone())),
                Token::Symbol(s) => exprs.push(Expr::Symbol(s.clone())),
                Token::LeftParen => {
                    let list_exprs = self.parse_tokens(Vec::new(), true)?;
                    exprs.push(Expr::List(list_exprs))
                }
                Token::RightParen => {
                    if in_list {
                        return Ok(exprs);
                    } else {
                        // #TODO custom error here?
                        return Err(Spanned::new(
                            ParseError::UnexpectedToken(t.clone()),
                            span.clone(),
                        ));
                    }
                }
                _ => {
                    return Err(Spanned::new(
                        ParseError::UnexpectedToken(t.clone()),
                        span.clone(),
                    ));
                }
            }

            println!("-- {exprs:?}");
        }

        Ok(exprs)
    }

    // #TODO returns AST
    // #TODO handle annotations
    pub fn parse(&mut self) -> Result<Expr, Spanned<ParseError>> {
        let exprs = Vec::new();

        let exprs = self.parse_tokens(exprs, false)?;

        Ok(Expr::List(exprs))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{token::Token, Lexer},
        parser::expr::Expr,
        span::Spanned,
        util::format::format_pretty_spanned_error,
    };

    use super::Parser;

    fn read_input(filename: &str) -> String {
        std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
    }

    fn lex_tokens(input: &str) -> Vec<Spanned<Token>> {
        let mut lexer = Lexer::new(input);
        lexer.lex().unwrap()
    }

    #[test]
    fn parse_handles_an_empty_token_list() {
        let input = &read_input("empty.tan");
        let tokens = lex_tokens(input);
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, Expr::List(x) if x.is_empty()));
    }

    #[test]
    fn parse_reports_unexpected_tokens() {
        let input = ")";
        let tokens = lex_tokens(input);
        let mut parser = Parser::new(&tokens);

        let result = parser.parse();
        assert!(result.is_err());

        let err = result.unwrap_err();

        eprintln!("{}", format_pretty_spanned_error(&err, input));
    }

    #[test]
    fn parse_handles_a_simple_expression() {
        let input = r#"
        ; Simple expression
        (print "hello")
        "#;
        let tokens = lex_tokens(input);
        let mut parser = Parser::new(&tokens);

        let result = parser.parse();
        dbg!(&result);
    }
}