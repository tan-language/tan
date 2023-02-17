// #TODO combine a vec of expressions into one `do` expression?, in this pass?

use crate::{ann::Ann, expr::Expr};

// #Insight
// The optimizer does not err.

pub fn optimize_fn(expr: Ann<Expr>) -> Ann<Expr> {
    match expr {
        Ann(Expr::List(ref terms), ..) => {
            if !terms.is_empty() {
                if let Ann(Expr::Symbol(s), ..) = &terms[0] {
                    if s == "Array" {
                        let items = terms[1..].iter().map(|ax| ax.0.clone()).collect();
                        return Ann(Expr::Array(items), expr.1);
                    }
                }
            }
            expr
        }
        _ => expr,
    }
}

pub fn optimize(expr: Ann<Expr>) -> Ann<Expr> {
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
}
