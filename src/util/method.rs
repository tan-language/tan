// A 'method' is a specific override of an operator/function.

// method-related utility functions.

use crate::{
    context::Context,
    expr::{format_value, Expr},
};

// #todo write unit test!
// #todo automatically infer the signature from type annotations.
// #insight only apply on invocable exprs.
pub fn compute_signature_from_annotations(expr: &Expr) -> Option<String> {
    if let Some(typ) = expr.annotation("type") {
        // #todo Add some error checking that the type is for a Func/Invocable.
        // #todo Add error checking that the signature is valid!
        // #todo Perform the check at static-time.
        // #todo add some error checking for the type annotation here.
        // #todo temp hack solution, only handles (Func [...] ...) types.
        let typ = format_value(typ);
        if !typ.starts_with("(Func ") {
            // #todo Raise an error here!
            eprintln!("Invalid invocable type annotation!");
            return None;
        }
        let typ = &typ[6..(typ.len() - 1)];
        // parts = [output, input]
        let parts = typ.rsplitn(2, " ").collect::<Vec<&str>>();
        let mut input = parts[1];

        if input.starts_with("(Array ") {
            // #todo #hack In general rething how to handle arrays of types, conflicts with the Array generic type.
            input = &input[7..(input.len() - 1)];
        }

        let parts = input.split(' ').collect::<Vec<_>>().join("$$");

        Some(format!("$${parts}"))
    } else {
        None
    }
}

// #todo signature should also encode the return type!!
// #todo how to handle VARARG functions ?!?!
pub fn compute_dyn_signature(args: &[Expr], context: &Context) -> String {
    let mut signature = Vec::new();

    for arg in args {
        let typ = arg.dyn_type(context);
        let Expr::Type(typ) = typ else {
            panic!("invalid dynamic type: {typ:?}");
        };
        signature.push(typ)
    }

    signature.join("$$")
}

#[cfg(test)]
mod tests {
    use crate::{api::eval_string, context::Context};

    #[test]
    fn compute_signature_from_annotations_usage() {
        let mut context = Context::new();

        let input = r#"
        #(Func [Vec2 Vec2] Vec2)
        (let + (Func [a b]
            (Vec2
                (+ (a 0) (b 0))
                (+ (a 1) (b 1))
            )
        ))
        "#;
        let _ = eval_string(input, &mut context);
        // dbg!(&context.scope);
        let expr = context.scope.get("+$$Vec2$$Vec2");
        assert!(expr.is_some());
    }
}
