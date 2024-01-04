use std::cmp::Ordering;

use crate::expr::Expr;

pub fn rust_ordering_from_tan_ordering(tan_ordering: &Expr) -> Option<Ordering> {
    let Expr::Int(ordering) = tan_ordering else {
        return None;
    };

    if *ordering < 0 {
        Some(Ordering::Less)
    } else if *ordering > 0 {
        Some(Ordering::Greater)
    } else {
        Some(Ordering::Equal)
    }
}
