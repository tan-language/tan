use crate::{context::Context, error::Error, expr::Expr};

use super::eval;

// #insight
// This is not the same as let, it also traverses the scope stack to find
// bindings to update in parent scopes.

// #insight
// In the past this function was called `set!`. It was renamed to `assign` to
// avoid ambiguity with mathematical sets, and to move away from the (too) noisy
// trailing `!` convention.

// #insight
// Don't use `poke` for this, reseve peek and poke.

// #insight
// Maybe the full `assign` name should be recommended, to add more friction.

// #insight
// Originally we the operator `:=` was used as an alias for assignment, like
// Pascal and Go. However, the `:=` conflicts with key-symbols :(
// So the operator `<-` is used instead, like R, Math, etc. The `<-` is somehow
// related with the `->` map/function operator.

// #insight
// The operator `:=` may be confusing with `+=`, `*=`, etc. On the other hand,
// if we use `<-` the other operators should be `+<-`, `*<-`, etc.

// #todo introduce +=, *=, etc. or +<-, *<-, etc.
// #todo introduce map=, map<-, etc. (let a (map= a adder))

pub fn eval_assign(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo this is a temp hack
    // #todo write unit tests
    // #todo support mutating multiple variables.

    let [name, value] = args else {
        return Err(Error::invalid_arguments("malformed `assign`", None));
    };

    let Some(name) = name.as_stringable() else {
        return Err(Error::invalid_arguments(
            "requires a symbol as the first argument",
            name.range(),
        ));
    };

    let value = eval(value, context)?;

    // #todo should we check that the symbol actually exists?
    context.scope.update(name, value.clone());

    // #todo what should this return? One/Unit (i.e. nothing useful) or the actual value?
    Ok(Expr::None)
}

// #todo eval_assign_add (+<- value delta)
// #todo eval_assign_add (+= value delta)

// #insight the conversion needs to happen in macro expand!
// #todo consider `eval_assing_plus`, plus is more general as the operator name.
// (+<- value delta)
// pub fn eval_assign_add(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
//     todo!()
// }

// #insight `+<-` and other assignment operators are expanded in macro_expand.
// #insight `plus` is a more general name than `add` for the operator.

// This is converted to a tan-test.
// #[cfg(test)]
// mod tests {
//     use crate::{api::eval_string, context::Context, expr::format_value};

//     #[test]
//     fn eval_assign_plus_usage() {
//         let mut context = Context::new();

//         let input = r#"
//         (let #var a 1)
//         (+<- a 5)
//         a
//         "#;
//         let value = eval_string(input, &mut context).unwrap();
//         assert_eq!(format_value(&value), "6");

//         let input = r#"
//         (+<- a 3)
//         a
//         "#;
//         let value = eval_string(input, &mut context).unwrap();
//         assert_eq!(format_value(&value), "9");
//     }
// }
