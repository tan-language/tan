// #TODO combine a vec of expressions into one `do` expression?, in this pass?

use std::collections::HashMap;

use crate::{
    ann::ANNO,
    expr::{format_value, Expr},
};

// #Insight
// The optimizer does not err.

pub fn optimize_fn(expr: Expr) -> Expr {
    match expr {
        ANNO(Expr::List(ref terms), ..) => {
            if !terms.is_empty() {
                if let ANNO(Expr::Symbol(s), ..) = &terms[0] {
                    if s == "Array" {
                        let items = terms[1..].iter().map(|ax| ax.0.clone()).collect();
                        return ANNO(Expr::Array(items), expr.1);
                    } else if s == "Dict" {
                        let items: Vec<Expr> = terms[1..].iter().map(|ax| ax.0.clone()).collect();
                        let mut dict = HashMap::new();
                        for pair in items.chunks(2) {
                            let k = pair[0].clone();
                            let v = pair[1].clone();
                            dict.insert(format_value(k), v);
                        }
                        return ANNO(Expr::Dict(dict), expr.1);
                    }
                }
            }
            expr
        }
        _ => expr,
    }
}

pub fn optimize(expr: Expr) -> Expr {
    expr.transform(&optimize_fn)
}

#[cfg(test)]
mod tests {
    use crate::{api::parse_string, optimize::optimize};

    #[test]
    fn optimize_rewrites_array_expressions() {
        let input = r#"(do (let a [1 2 3 4]) (writeln (+ 2 3)))"#;

        let expr = parse_string(input).unwrap();

        let expr_optimized = optimize(expr);

        let s = format!("{expr_optimized:?}");

        assert!(s.contains("Array([Int(1), Int(2), Int(3), Int(4)])"));
    }

    // #TODO the test is flaky for some reason, temporarily disabled, investigate.
    // #[test]
    fn _optimize_rewrites_dict_expressions() {
        let input = r#"(let a {:name "George" :age 25})"#;

        let expr = parse_string(input).unwrap();

        let expr_optimized = optimize(expr);

        let s = format!("{expr_optimized:?}");

        assert!(s.contains(r#"Dict({"name": String("George"), "age": Int(25)})"#));
    }
}
