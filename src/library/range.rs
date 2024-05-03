use std::sync::Arc;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

// #todo make some fields optional.
// struct ForeignRange<T> {
//     pub start: T,
//     pub end: T,
//     pub step: T,
// }

// #todo Range is Immutable, Copy.

// #todo not used yet.
// fn make_range<T: Send + Sync + 'static>(start: T, end: T, step: T) -> Expr {
//     // #todo use IntRange, FloatRange.
//     let foreign_range = ForeignRange { start, end, step };
//     let expr = Expr::ForeignStruct(Arc::new(foreign_range));
//     // #todo should annotate (Range Int) or (Range Float)
//     annotate_type(expr, "Range")
// }

pub fn range_int_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo make some of the arguments optional, e.g. step.
    let [start, end, ..] = args else {
        return Err(Error::invalid_arguments(
            "requires `start`, `end` arguments",
            None,
        ));
    };

    // #todo create a helper.
    let Some(start) = start.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("start=`{start}` is not Int"),
            start.range(),
        ));
    };

    let Some(end) = end.as_int() else {
        return Err(Error::invalid_arguments(
            &format!("end=`{end}` is not Int"),
            end.range(),
        ));
    };

    let step = if let Some(step) = args.get(2) {
        let Some(step) = step.as_int() else {
            return Err(Error::invalid_arguments(
                &format!("step=`{step}` is not Int"),
                step.range(),
            ));
        };
        step
    } else if end >= start {
        1
    } else {
        -1
    };

    // #todo use Expr::ForeignStruct
    Ok(Expr::IntRange(start, end, step))
}

pub fn range_float_new(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo make some of the arguments optional, e.g. step.
    let [start, end, ..] = args else {
        return Err(Error::invalid_arguments(
            "requires `start`, `end`, and `step` arguments",
            None,
        ));
    };

    // #todo create a helper.
    let Some(start) = start.as_float() else {
        return Err(Error::invalid_arguments(
            &format!("start=`{start}` is not Float"),
            start.range(),
        ));
    };

    let Some(end) = end.as_float() else {
        return Err(Error::invalid_arguments(
            &format!("end=`{end}` is not Float"),
            end.range(),
        ));
    };

    let step = if let Some(step) = args.get(2) {
        let Some(step) = step.as_float() else {
            return Err(Error::invalid_arguments(
                &format!("step=`{step}` is not Float"),
                step.range(),
            ));
        };
        step
    } else if end >= start {
        1.0
    } else {
        -1.0
    };

    // #todo use Expr::ForeignStruct
    Ok(Expr::FloatRange(start, end, step))
}

pub fn setup_lib_range(context: &mut Context) {
    // #todo put in 'range' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    module.insert("Range", Expr::ForeignFunc(Arc::new(range_int_new)));
    module.insert(
        "Range$$Int$$Int",
        Expr::ForeignFunc(Arc::new(range_int_new)),
    );
    module.insert(
        "Range$$Int$$Int$$Int",
        Expr::ForeignFunc(Arc::new(range_int_new)),
    );
    module.insert(
        "Range$$Float$$Float",
        Expr::ForeignFunc(Arc::new(range_float_new)),
    );
    module.insert(
        "Range$$Float$$Float$$Float",
        Expr::ForeignFunc(Arc::new(range_float_new)),
    );
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context, expr::format_value};

    #[test]
    fn range_new_usage() {
        let mut context = Context::new();

        let input = r#"
            (let #mut vals [])
            (for [i 0..5]
                (push vals i)
            )
            vals
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = r#"[0 1 2 3 4]"#;
        assert_eq!(value, expected);

        let input = r#"
            (let start 1)
            (let end 4)
            (let #mut vals [])
            (for [i start..end]
                (push vals i)
            )
            vals
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        let value = format_value(expr);
        let expected = r#"[1 2 3]"#;
        assert_eq!(value, expected);
    }
}
