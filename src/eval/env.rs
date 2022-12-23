// #Insight
// It's better to model with a stack instead of pointers to outer environment.
// A stack better describes the actual construct, is easier to reason about (no sea-of-objects)
// and is more efficient on contemporary hardware.

/// The environment is a stack of scopes.
pub struct Env {
    // #TODO
}
