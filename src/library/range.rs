use crate::{
    context::Context,
    error::Error,
    expr::Expr,
    util::{
        args::{unpack_float_arg, unpack_float_range_arg},
        module_util::require_module,
    },
};

// #todo Do we need a separate Interval type?
// #todo Range could be like in Rust and _not_ include a step, instead provide a step_by function?

// #todo #hack No need for Range expressions, this is a temp hack.

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

pub fn range_int_new(args: &[Expr]) -> Result<Expr, Error> {
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

pub fn range_float_new(args: &[Expr]) -> Result<Expr, Error> {
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

// #todo Implement those in Tan.

// #todo Add contains.
// #todo Add surrounds.
// #todo Add size/length.
// #todo Constants: empty, universe.
// #todo The name `universe` is weird.

// #todo Support open-ended and close-ended ranges.

// #todo Add unit tests.
// #todo Should take the step into account? Probably yes.
// #todo The term `contains` is a bit ambiguous, could use `includes` or `surrounds`?
// #todo Maybe `surrounds` should be like contains/includes but ignoring the step.
pub fn range_float_contains(args: &[Expr]) -> Result<Expr, Error> {
    let range = unpack_float_range_arg(args, 0, "range")?;
    let value = unpack_float_arg(args, 1, "value")?;
    Ok(Expr::Bool(range.contains(&value)))
}

pub fn setup_lib_range(context: &mut Context) {
    // #todo put in 'range' path, and import selected functionality to prelude.
    let module = require_module("prelude", context);

    module.insert("Range", Expr::foreign_func(&range_int_new));
    module.insert("Range$$Int$$Int", Expr::foreign_func(&range_int_new));
    module.insert("Range$$Int$$Int$$Int", Expr::foreign_func(&range_int_new));
    module.insert("Range$$Float$$Float", Expr::foreign_func(&range_float_new));
    module.insert(
        "Range$$Float$$Float$$Float",
        Expr::foreign_func(&range_float_new),
    );

    // #todo Contains should probably take the step into account.
    module.insert(
        "contains?$$(Range Float)$$Float",
        Expr::foreign_func(&range_float_contains),
    );
    // #todo Find a better name, originally from https://raytracing.github.io/
    module.insert("surrounds?", Expr::foreign_func(&range_float_contains));
    module.insert(
        "surrounds?$$(Range Float)$$Float",
        Expr::foreign_func(&range_float_contains),
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
