use std::cmp::Ordering;

use crate::expr::Expr;

pub fn rust_ordering_from_tan_ordering(tan_ordering: &Expr) -> Option<Ordering> {
    let Expr::Int(ordering) = tan_ordering else {
        return None;
    };

    Some(ordering.cmp(&0))
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use assert_matches::assert_matches;

    use crate::{expr::Expr, stdlib::cmp::rust_ordering_from_tan_ordering};

    #[test]
    fn rust_ordering_from_tan_ordering_usage() {
        assert_matches!(
            rust_ordering_from_tan_ordering(&Expr::Int(20)),
            Some(Ordering::Greater)
        );
        assert_matches!(
            rust_ordering_from_tan_ordering(&Expr::Int(0)),
            Some(Ordering::Equal)
        );
    }
}
