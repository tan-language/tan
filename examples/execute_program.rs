use tan::{
    error::Error,
    eval::{env::Env, eval, prelude::setup_prelude},
    expr::Expr,
    lexer::Lexer,
    parser::Parser,
};

// #TODO extract this function to the library.
pub fn eval_string(input: impl AsRef<str>, env: &mut Env) -> Result<Expr, Error> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    let value = eval(expr, env)?;

    Ok(value)
}

pub fn main() {
    let input_path = "tests/fixtures/fibonacci.tan";

    let input = std::fs::read_to_string(input_path).expect("cannot read input");

    let mut env = setup_prelude(Env::default());

    let value = eval_string(&input, &mut env);

    if let Ok(value) = value {
        println!("{value}");
    } else {
        eprintln!("{}", value.unwrap_err());
    }
}
