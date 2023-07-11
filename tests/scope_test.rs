use tan::{expr::Expr, scope::Scope};

// #todo test upstream (recursive) lookups

#[test]
fn scope_binds_names_to_values() {
    let scope = Scope::default();

    scope.insert("a", Expr::symbol("hello"));

    if let Some(a) = scope.get("a") {
        assert!(matches!((*a).clone(), Expr::Symbol(sym) if sym == "hello"));
    }

    assert!(matches!(scope.get("b"), None));
}

#[test]
fn scope_bindings_can_be_updated() {
    let scope = Scope::default();

    scope.insert("a", Expr::symbol("hello"));
    if let Some(a) = scope.get("a") {
        assert!(matches!((*a).clone(), Expr::Symbol(sym) if sym == "hello"));
    }

    scope.update("a", Expr::symbol("world"));
    if let Some(a) = scope.get("a") {
        assert!(matches!((*a).clone(), Expr::Symbol(sym) if sym == "world"));
    }
}
