// A 'method' is a specific override of an operator/function.

// method-related utility functions.

use crate::{context::Context, expr::Expr};

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
