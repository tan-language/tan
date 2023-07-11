use tan::{api::eval_string, context::Context};

pub fn main() {
    let input_path = "tests/fixtures/fibonacci.tan";

    let input = std::fs::read_to_string(input_path).expect("cannot read input");

    let mut context = Context::new();

    let value = eval_string(&input, &mut context);

    if let Ok(value) = value {
        println!("{value}");
    } else {
        eprintln!("{:?}", value.unwrap_err());
    }
}
