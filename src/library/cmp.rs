use std::cmp::Ordering;

use crate::{
    context::Context,
    expr::{annotate_type, Expr},
    util::module_util::require_module,
};

use super::{arithmetic, string::string_compare};

pub fn rust_ordering_from_tan_ordering(tan_ordering: &Expr) -> Option<Ordering> {
    let ordering = tan_ordering.as_int()?;
    Some(ordering.cmp(&0))
}

pub fn setup_lib_cmp(context: &mut Context) {
    let module = require_module("prelude", context);

    // cmp

    // #todo `eq` and `Comparable` are related.
    // #todo consider to make sorter: `cmp`.

    module.insert_invocable("compare", Expr::foreign_func(&arithmetic::int_compare));
    module.insert_invocable(
        "compare$$Int$$Int",
        annotate_type(Expr::foreign_func(&arithmetic::int_compare), "Int"),
    );
    module.insert_invocable(
        "compare$$String$$String",
        annotate_type(Expr::foreign_func(&string_compare), "String"),
    );
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use assert_matches::assert_matches;

    use crate::{expr::Expr, library::cmp::rust_ordering_from_tan_ordering};

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
