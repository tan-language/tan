// #todo rename to env or something else?
// #todo this is a temp hack.

// #todo just put this
// #todo consider a function that nests a new scope.
/// Inserts a dict to the scope.
pub fn scope_insert(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [dict] = args else {
        return Err(Error::invalid_arguments("requires `dict` argument", None));
    };
}
