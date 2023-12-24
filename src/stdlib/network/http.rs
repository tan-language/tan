// #todo find good module path: network/http?
// network/http
// network/http/ws
// network/smtp

// #insight network/http is better than protocol/http, more specific.
// #insight use https://httpbin.org/ for testing.

// #ref https://tokio.rs/tokio/topics/bridging
// #ref https://crates.io/crates/reqwest

// #todo in the future consider using the lower-level hyper library.
// #todo in the future consider an async implementation, bring-in the tokio runtime.
// #todo introduce StatusCode, canonical reason.

use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::{context::Context, error::Error, expr::Expr, module::Module};

pub fn build_tan_response(
    resp: reqwest::Result<reqwest::blocking::Response>,
) -> Result<Expr, Error> {
    let Ok(resp) = resp else {
        // #todo should return Error::Io, ideally wrap the lower-level error.
        // #todo return a better error.
        // #todo more descriptive error needed here.
        return Err(Error::general("failed http request"));
    };

    let status = resp.status().as_u16() as i64;

    let Ok(body) = resp.text() else {
        // #todo return a better error.
        // #todo more descriptive error needed here.
        return Err(Error::general("cannot read http response body"));
    };

    let mut tan_response = HashMap::new();
    tan_response.insert("status".to_string(), Expr::Int(status));
    tan_response.insert("body".to_string(), Expr::string(body));
    // #todo also include response headers.

    Ok(Expr::dict(tan_response))
}

pub fn http_get(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [url] = args else {
        return Err(Error::invalid_arguments(
            "`get` requires `url` argument",
            None,
        ));
    };

    let Some(url) = url.as_stringable() else {
        return Err(Error::invalid_arguments(
            "`url` argument should be a Stringable",
            url.range(),
        ));
    };

    let resp = reqwest::blocking::get(url);

    build_tan_response(resp)
}

// #todo implement me.
// #todo support non-string bodies.
pub fn http_post(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [url, body] = args else {
        return Err(Error::invalid_arguments(
            "`post` requires `url` and `body` argument",
            None,
        ));
    };

    let Some(url) = url.as_stringable() else {
        return Err(Error::invalid_arguments(
            "`url` argument should be a Stringable",
            url.range(),
        ));
    };

    // #insight
    // the following doesn't work:
    // let Some(body) = body.as_stringable() else {

    // #todo support stringables and streaming.
    let Expr::String(body) = body.unpack() else {
        return Err(Error::invalid_arguments(
            "`body` argument should be a Stringable",
            body.range(),
        ));
    };

    let body = body.clone();

    // #todo support streaming.
    // #todo use async

    let client = reqwest::blocking::Client::new();
    let resp = client.post(url).body(body).send();

    build_tan_response(resp)
}

// (http/send :POST "https://api.site.com/create" )
// (let resp (http/post "https://api.site.com/create" "body" { :content-encoding "application/json" }))
// (resp :status)

pub fn setup_lib_http(context: &mut Context) {
    let module = Module::new("http", context.top_scope.clone());

    let scope = &module.scope;

    // (let http/get "https://tan-language.org");

    scope.insert("get", Expr::ForeignFunc(Arc::new(http_get)));

    scope.insert("post", Expr::ForeignFunc(Arc::new(http_post)));

    // #todo another name than dialect? (language, lang, flavor, dsl)
    // (use dialect/css-expr) (use dialect/css) (use dialect/html)
    // #todo this is a hack.
    let module_path = format!("{}/@std/network/http", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module)); // #todo use Arc everywhere!
}

// #todo add a unit test that at least exercises these functions.
// #todo use https://httpbin.org/ for testing.
