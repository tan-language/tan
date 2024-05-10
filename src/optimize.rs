use std::collections::HashMap;

use crate::expr::{format_value, Expr};

// #insight The optimizer does not err.

// #insight
// The optimizer converts general Expr::List expressions into execution-friendly
// expressions like Expr::Array, Expr::Map, etc. It also strips unnecessary
// annotations.

// #todo what does optimize do? I think it just removes some annotations.

// #todo #think hm, we NEED the annotations, especially in let expressions!

pub fn optimize_fn(expr: Expr) -> Expr {
    // #todo let annotations are lost here.
    match expr.unpack() {
        Expr::List(ref terms) => {
            if !terms.is_empty() {
                if let Expr::Symbol(s) = &terms[0].unpack() {
                    if s == "Array" {
                        let items: Vec<Expr> =
                            terms[1..].iter().map(|ax| ax.unpack().clone()).collect();
                        return Expr::maybe_annotated(Expr::array(items), expr.annotations());
                    } else if s == "Map" {
                        let items: Vec<Expr> =
                            terms[1..].iter().map(|ax| ax.unpack().clone()).collect();
                        let mut map = HashMap::new();
                        for pair in items.chunks(2) {
                            let k = pair[0].clone();
                            let v = pair[1].clone();
                            map.insert(format_value(k), v);
                        }
                        return Expr::maybe_annotated(Expr::map(map), expr.annotations());
                    }
                }
            }
            // #insight no annotations stripped.
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

        // assert!(s.contains("Array([Int(1), Int(2), Int(3), Int(4)])"));
        // #todo this is a _very_ unstable/error-prone check.
        assert!(s.contains("Array(RwLock { data: [Int(1), Int(2), Int(3), Int(4)]"));
    }

    // #todo the test is flaky for some reason, temporarily disabled, investigate.
    // #[test]
    // fn optimize_rewrites_map_expressions() {
    //     let input = r#"(let a {:name "George" :age 25})"#;

    //     let expr = parse_string(input).unwrap();

    //     let expr_optimized = optimize(expr);

    //     let s = format!("{expr_optimized:?}");

    //     assert!(s.contains(r#"Map({"name": String("George"), "age": Int(25)})"#));
    // }
}
