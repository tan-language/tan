use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::Request,
    handler::HandlerWithoutStateExt,
    http::{header, StatusCode},
};

use crate::{
    context::Context,
    error::Error,
    eval::invoke_func,
    expr::{annotate_type, Expr},
    util::module_util::require_module,
};

static DEFAULT_ADDRESS: &str = "127.0.0.1";
static DEFAULT_PORT: i64 = 8000; // #todo what should be the default port?

// #see https://docs.rs/axum/latest/axum/response/index.html

async fn run_server(options: HashMap<String, Expr>, handler: Expr, context: &mut Context) {
    // #todo #think should have separate context per thread? per task/fiber?
    let mut context = context.clone();

    let axum_handler = |axum_req: Request| async move {
        // #todo what else to pass to tan_req? (headers, method, ...)
        let mut map = HashMap::new();
        map.insert("uri".to_string(), Expr::string(axum_req.uri().to_string()));
        // #todo consider "/http/Request".
        let req = annotate_type(Expr::map(map), "http/Request");

        // #todo handle conversion of more return types.
        if let Ok(value) = invoke_func(&handler, &[req], &mut context) {
            // #todo set content type depending on output.
            if let Some(value) = value.as_stringable() {
                return (
                    StatusCode::FOUND,
                    [(header::CONTENT_TYPE, "text/html")],
                    value.to_string(),
                );
            }
        }
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain")],
            "internal server error".to_string(),
        )
    };

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

    axum::serve(listener, axum_handler.into_make_service())
        .await
        .unwrap();
}

// #todo investigate the Go http-serve API.

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

    let rt = tokio::runtime::Runtime::new().unwrap();
    // #todo can we remove the clones?
    rt.block_on(run_server(options.clone(), handler.clone(), context));

    // #insight never returns!
    Ok(Expr::Never)
}

pub fn setup_lib_http_server(context: &mut Context) {
    let module = require_module("network/http", context);
    module.insert("serve", Expr::ForeignFunc(Arc::new(http_serve)));
}
