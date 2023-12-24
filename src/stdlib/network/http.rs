// #todo find good module path: network/http?
// network/http
// network/http/ws
// network/smtp

// #insight network/http is better than protocol/http, more specific.

// #ref https://tokio.rs/tokio/topics/bridging
// #ref https://crates.io/crates/reqwest

// #todo in the future consider using the lower-level hyper library.
// #todo in the future consider an async implementation, bring-in the tokio runtime.
// #todo introduce StatusCode, canonical reason.

use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::{context::Context, error::Error, expr::Expr, module::Module};

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

pub fn setup_lib_http(context: &mut Context) {
    let module = Module::new("http", context.top_scope.clone());

    let scope = &module.scope;

    // (let http/get "https://tan-language.org");

    scope.insert("get", Expr::ForeignFunc(Arc::new(http_get)));

    // #todo another name than dialect? (language, lang, flavor, dsl)
    // (use dialect/css-expr) (use dialect/css) (use dialect/html)
    // #todo this is a hack.
    let module_path = format!("{}/@std/network/http", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module)); // #todo use Arc everywhere!
}
