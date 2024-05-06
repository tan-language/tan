// #insight random is _not_ part of math! (are you sure?)

// #todo move out of standard library into the common library

// #todo random_int, random_float
// #todo should take a range trait.
// #todo support random-number-generator 'object', with more specialization
// #todo fast, crypto-secure, very-random, per-thread, etc versions.
// #todo but also a couple of helper functions.
// #todo better module name: stochastic, rnd, rng? `rng` is interesting, nah 'rng' is not great.

// #todo use OnceLock to cache the RNG

// #todo better (?) api:
// (use random)
// (let n (random/int 5))
// (let n (random/float 5))
// (let n (random/num 5)) ; generic<

// #todo add support for seeding.

use std::sync::Arc;

use rand::Rng;

use crate::{context::Context, error::Error, expr::Expr, util::module_util::require_module};

/// (random 100) returns a random integer in the range 0..100
pub fn random_int(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
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

pub fn setup_lib_rand(context: &mut Context) {
    // #todo what is a good path? should avoid math?
    let module = require_module("rng", context);

    // #todo better name?
    module.insert("random", Expr::ForeignFunc(Arc::new(random_int)));
}

// #todo add unit tests.
