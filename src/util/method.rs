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
    // #todo the signature annotation is super-low-level, remove?
    if let Some(typ) = expr.annotation("type") {
        // #todo add some error checking for the type annotation here.
        // #todo temp hack solution, only handles (Func [...] ...) types.
        let typ = format_value(typ);
        let typ = typ.replace(['(', ')'], "");
        let parts: Vec<&str> = typ.split(' ').collect();
        let mut typ = parts[1..parts.len() - 1].join("$$");
        // #todo #hack ultra nasty!
        if typ.starts_with("Array") {
            typ = typ.chars().skip(5 /* "Array".len() */).collect();
        }
        Some(typ)
    } else {
        None
    }

    // if let Some(signature) = expr.annotation("signature") {
    //     // #todo validate that signature is a string.
    //     let signature = format_value(signature);
    //     // #todo this is temp convention!
    //     Some(format!("$${signature}"))
    // } else {
    //     None
    // }
}

// #todo signature should also encode the return type!!
// #todo how to handle VARARG functions ?!?!
pub fn compute_signature(args: &[Expr]) -> String {
    let mut signature = Vec::new();

    for arg in args {
        signature.push(arg.static_type().to_string())
    }

    signature.join("$$")
}

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
