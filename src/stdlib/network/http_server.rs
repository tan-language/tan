use std::{collections::HashMap, sync::Arc};

use axum::{routing::get, Router};

use crate::{
    context::Context, error::Error, eval::invoke_func, expr::Expr,
    util::module_util::require_module,
};

async fn handler() -> &'static str {
    "Hello, World!"
}

static DEFAULT_ADDRESS: &str = "127.0.0.1";
static DEFAULT_PORT: i64 = 8000; // #todo what should be the default port?

async fn run_server(options: HashMap<String, Expr>) {
    let app = Router::new().route("/", get(handler));

    let address = if options.contains_key("address") {
        if let Some(address) = options["address"].as_stringable() {
            address
        } else {
            DEFAULT_ADDRESS
        }
    } else {
        DEFAULT_ADDRESS
    };

    let port = if options.contains_key("port") {
        if let Some(port) = options["port"].as_int() {
            port
        } else {
            DEFAULT_PORT
        }
    } else {
        DEFAULT_PORT
    };

    let addr = format!("{address}:{port}");

    // run it
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // #todo add some kind of tracing?
    // println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

// (http/serve {:port 8000} (Func [] "hello world!"))
pub fn http_serve(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    // #todo consider other name instead of handler, e.g. `callback`.
    let [options, handler] = args else {
        return Err(Error::invalid_arguments(
            "`serve` requires `options` and `handler` arguments",
            None,
        ));
    };

    let Some(options) = options.as_map() else {
        return Err(Error::invalid_arguments(
            "`options` argument should be a Map",
            options.range(),
        ));
    };

    let Expr::Func(..) = handler.unpack() else {
        return Err(Error::invalid_arguments(
            "`handler` argument should be a Func",
            handler.range(),
        ));
    };

    let result = invoke_func(handler, &Vec::new(), context)?;

    eprintln!("=== {options:?} : {result}");

    let rt = tokio::runtime::Runtime::new().unwrap();
    // #todo argh, this clone is nasty!s
    rt.block_on(run_server(options.clone()));

    // #insight never returns!
    Ok(Expr::Zero)
}

pub fn setup_lib_http_server(context: &mut Context) {
    let module = require_module("network/http", context);
    module.insert("serve", Expr::ForeignFunc(Arc::new(http_serve)));
}
