use tan::{eval::env::Env, expr::Expr};

#[test]
fn env_binds_names_to_values() {
    let mut env = Env::default();

    env.insert("a", Expr::symbol("hello"));

    // let expr: &Expr = env.get("a").unwrap().as_ref();
    // dbg!(&expr);

    assert!(matches!(env.get("a"), Some(Expr::Symbol(sym)) if sym == "hello"));
    assert!(matches!(env.get("b"), None));
}

#[test]
fn env_bindings_can_be_updated() {
    let mut env = Env::default();

    env.insert("a", Expr::symbol("hello"));
    assert!(matches!(env.get("a"), Some(Expr::Symbol(sym)) if sym == "hello"));

    env.update("a", Expr::symbol("world"));
    assert!(matches!(env.get("a"), Some(Expr::Symbol(sym)) if sym == "world"));
}
