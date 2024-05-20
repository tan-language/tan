// A 'method' is a specific override of an operator/function.

// method-related utility functions.

use crate::{
    context::Context,
    expr::{format_value, Expr},
};

// #todo automatically infer the signature from type annotations.
// #insight only apply on invocable exprs.
pub fn compute_signature_from_annotations(expr: &Expr) -> Option<String> {
    // #todo the signature annotation is super-low-level, remove?
    if let Some(signature) = expr.annotation("signature") {
        // #todo validate that signature is a string.
        let signature = format_value(signature);
        // #todo this is temp convention!
        Some(format!("$${signature}"))
    } else {
        None
    }
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
