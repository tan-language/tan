// #insight random is _not_ part of math!

// #todo move out of standard library into the common library.

// #todo random_int, random_float
// #todo should take a range trait.
// #todo support random-number-generator 'object', with more specialization
// #todo fast, crypto-secure, very-random, per-thread, etc versions.
// #todo but also a couple of helper functions.
// #todo better module name: stochastic, rnd, rng? `rng` is interesting, nah 'rng' is not great.

// #todo use OnceLock to cache the RNG

use std::{rc::Rc, sync::Arc};

use crate::{context::Context, error::Error, expr::Expr, module::Module};
use rand::Rng;

/// (random 100) returns a random integer in the range 0..100
pub fn random_int(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    if let Some(end) = args.first() {
        let Some(end) = end.as_int() else {
            return Err(Error::invalid_arguments(
                "expected Int argument",
                end.range(),
            ));
        };

        let mut rng = rand::thread_rng();

        Ok(Expr::Int(rng.gen_range(0..end)))
    } else {
        Err(Error::invalid_arguments(
            "expected at least one argument",
            None,
        ))
    }
}

pub fn setup_std_rand(context: &mut Context) {
    let module = Module::new("rng", context.top_scope.clone());

    let scope = &module.scope;

    scope.insert("random", Expr::ForeignFunc(Arc::new(random_int)));

    // #todo this is a hack.
    // #todo what happens if there are multiple root_paths?
    let module_path = format!("{}/@std/std/rng", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}
