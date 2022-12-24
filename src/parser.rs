use crate::{
    ann::Annotated,
    expr::Expr,
    lexer::token::Token,
    range::{Range, Ranged},
};

use self::error::ParseError;

pub mod error;

// #TODO no need to keep iterator as state in parser!
// #TODO can the parser be just a function? -> yes, if we use a custom iterator to keep the parsing state.

// #Insight
// The syntax of the language is explicitly designed to _not_ require a lookahead buffer.

// #Insight
// We move the tokens into the parser to simplify the code. The tokens are useless outside the parser.

/// The Parser performs the syntax analysis stage of the compilation pipeline.
/// The input token stream is reduced into and Abstract Syntax Tree (AST).
/// The nodes of the AST are associated with annotations.
pub struct Parser<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
{
    tokens: I::IntoIter,
    index: usize,
    active_annotations: Option<Vec<Ranged<String>>>,
}

impl<I> Parser<I>
where
    I: IntoIterator<Item = Ranged<Token>>,
{
    pub fn new(tokens: I) -> Self {
        let tokens = tokens.into_iter();

        Self {
            tokens,
            index: 0,
            active_annotations: None,
        }
    }

    fn next_token(&mut self) -> Option<Ranged<Token>> {
        self.index += 1;
        self.tokens.next()
    }

    pub fn apply_annotations(&mut self, expr: Expr) -> Annotated<Expr> {
        Annotated(expr, self.active_annotations.take())
    }

    // #TODO AST = Vec<Spanned<Annotated<Expr>>>

    // #TODO better name!
    pub fn parse_tokens(
        &mut self,
        exprs: Vec<Annotated<Expr>>,
        list_span: Option<Range>,
    ) -> Result<Vec<Annotated<Expr>>, Ranged<ParseError>> {
        let mut exprs = exprs;

        let mut token: Option<Ranged<Token>>;

        loop {
            token = self.next_token();

            let Some(st) = token  else {
                if let Some(span) = list_span {
                    return Err(Ranged(
                        ParseError::UnterminatedList,
                        span,
                    ));
                }
                break;
            };

            let Ranged(t, span) = st;

            match t {
                Token::Comment(..) => (),
                Token::String(s) => {
                    let expr = self.apply_annotations(Expr::String(s));
                    exprs.push(expr);
                }
                Token::Symbol(s) => {
                    let expr = self.apply_annotations(Expr::Symbol(s));
                    exprs.push(expr);
                }
                Token::Number(n) => {
                    let expr = self.apply_annotations(Expr::Int(n));
                    exprs.push(expr);
                }
                Token::Annotation(s) => {
                    if self.active_annotations.is_none() {
                        self.active_annotations = Some(Vec::new());
                    }

                    self.active_annotations
                        .as_mut()
                        .unwrap()
                        .push(Ranged(s, span));
                }
                Token::LeftParen => {
                    let list_exprs = self.parse_tokens(Vec::new(), Some(span))?;
                    let expr = self.apply_annotations(Expr::List(list_exprs));
                    exprs.push(expr);
                }
                Token::RightParen => {
                    if list_span.is_some() {
                        return Ok(exprs);
                    } else {
                        // #TODO custom error here?
                        return Err(Ranged(ParseError::UnexpectedToken(t), span));
                    }
                }
                _ => {
                    return Err(Ranged(ParseError::UnexpectedToken(t), span));
                }
            }
        }

        Ok(exprs)
    }

    // #TODO handle annotations
    pub fn parse(&mut self) -> Result<Annotated<Expr>, Ranged<ParseError>> {
        let exprs = self.parse_tokens(Vec::new(), None)?;

        Ok(Annotated::new(Expr::Do(exprs)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ann::Annotated,
        expr::Expr,
        lexer::{token::Token, Lexer},
        range::Ranged,
        util::format::format_pretty_error,
    };

    use super::Parser;

    fn read_input(filename: &str) -> String {
        std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
    }

    fn lex_tokens(input: &str) -> Vec<Ranged<Token>> {
        let mut lexer = Lexer::new(input);
        lexer.lex().unwrap()
    }

    #[test]
    fn parse_handles_an_empty_token_list() {
        let input = &read_input("empty.tan");
        let tokens = lex_tokens(input);
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, Annotated(Expr::Do(x), ..) if x.is_empty()));
    }

    #[test]
    fn parse_reports_unexpected_tokens() {
        let input = ")";
        let tokens = lex_tokens(input);
        let mut parser = Parser::new(tokens);

        let result = parser.parse();
        assert!(result.is_err());

        let err = result.unwrap_err();

        eprintln!("{}", format_pretty_error(&err, input, None));
    }

    #[test]
    fn parse_handles_a_simple_expression() {
        let input = &read_input("hello_world.tan");
        let tokens = lex_tokens(input);
        let mut parser = Parser::new(tokens);

        let result = parser.parse();
        dbg!(&result);
    }

    #[test]
    fn parse_reports_unterminated_lists() {
        let filename = "unterminated_list_expr.tan";
        let input = &read_input(filename);
        let tokens = lex_tokens(input);
        let mut parser = Parser::new(tokens);

        let result = parser.parse();
        assert!(result.is_err());

        let err = result.unwrap_err();

        eprintln!("{}", format_pretty_error(&err, input, Some(filename)));
    }

    #[test]
    fn parse_handles_annotations() {
        let input = r#"
        (let a #zonk #Int8 25 b #(inline true) 1)
        "#;
        let tokens = lex_tokens(input);
        let mut parser = Parser::new(tokens);

        let expr = parser.parse().unwrap();
        // println!("{expr:?}");
        dbg!(&expr);
    }
}
