use tan::{api::eval_string, eval::env::Env};

pub fn main() {
    let input_path = "tests/fixtures/fibonacci.tan";

    let input = std::fs::read_to_string(input_path).expect("cannot read input");

    let mut env = Env::prelude();

    let value = eval_string(&input, &mut env);

    if let Ok(value) = value {
        println!("{value}");
    } else {
        eprintln!("{}", value.unwrap_err());
    }
}
