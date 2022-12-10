use crate::lexer::LexicalError;

pub fn pretty_print_error(error: &LexicalError, input: &str) -> String {
    let chars = input.chars();

    match error {
        LexicalError::NumberError(_pie, span) => {
            let mut index: usize = 0;
            let mut line = 0;
            let mut line_start: usize = 0;
            let mut line_str = String::new();

            for c in chars {
                index += 1;

                if c == '\n' {
                    println!("{index} {line} {} {line_str}", span.start);
                    if index > span.start {
                        break;
                    }

                    line += 1;
                    line_start = index;

                    line_str.clear();

                    continue;
                }

                line_str.push(c);
            }

            let line_space = " ".repeat(format!("{}", line + 1).len());

            let len = span.len();

            let indicator = if len == 1 {
                "^--- near here".to_owned()
            } else {
                "^".repeat(len)
            };

            let col = span.start - line_start;
            let indicator_space = " ".repeat(col);

            let description = "malformed number";

            format!(
                "parse error: {}\n{}at input:{}:{}\n{}|\n{}| {}\n{}|{} {}",
                description,
                line_space,
                line + 1,
                col + 1,
                line_space,
                line + 1,
                line_str,
                line_space,
                indicator_space,
                indicator,
            )
        }
        _ => "error".to_owned(),
    }
}
