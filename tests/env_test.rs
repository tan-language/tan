use tan::{ann::Annotated, eval::env::Env, expr::Expr};

#[test]
fn env_binds_names_to_values() {
    let mut env = Env::default();

    env.insert("a", Expr::Symbol("hello".to_string()));

    // let expr: &Expr = env.get("a").unwrap().as_ref();
    // dbg!(&expr);

    assert!(matches!(env.get("a"), Some(Annotated(Expr::Symbol(sym), ..)) if sym == "hello"));
    assert!(matches!(env.get("b"), None));
}

#[test]
fn env_bindings_can_be_updated() {
    let mut env = Env::default();

    env.insert("a", Expr::Symbol("hello".to_string()));
    assert!(matches!(env.get("a"), Some(Annotated(Expr::Symbol(sym), ..)) if sym == "hello"));

    env.update("a", Expr::Symbol("world".to_string()));
    assert!(matches!(env.get("a"), Some(Annotated(Expr::Symbol(sym), ..)) if sym == "world"));
}
