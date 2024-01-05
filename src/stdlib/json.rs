use std::{collections::HashMap, rc::Rc, sync::Arc};

use serde_json::Value;

use crate::{
    context::Context, error::Error, expr::Expr, module::Module, util::module_util::require_module,
};

// #todo text/json or codec/json?
// #todo support json with comments.
// #todo functions should not panic on invalid arguments!!

/// Converts a JSON Value to a symbolic Expr.
fn json_value_to_expr(json: Value) -> Expr {
    match json {
        Value::Array(items) => {
            let mut arr = Vec::new();
            for item in items {
                arr.push(json_value_to_expr(item));
            }
            // #todo should generate base AST
            Expr::array(arr)
        }
        Value::Object(obj) => {
            let mut dict: HashMap<String, Expr> = HashMap::new();
            for (key, value) in obj {
                // #todo should support more key types.
                // #todo should convert k from camelCase, PascalCase, snake_case, etc.
                let key = key.replace('_', "-");
                // let key = Expr::KeySymbol(key);
                dict.insert(key, json_value_to_expr(value));
            }
            Expr::dict(dict)
        }
        Value::String(s) => Expr::String(s),
        Value::Number(n) => Expr::Float(n.as_f64().unwrap()), // #todo handle Int, Float, remove unwrap!
        Value::Bool(b) => Expr::Bool(b),
        Value::Null => Expr::One, // #todo is Unit the correct mapping?
    }
}

// #todo implement write/encode.

// #todo find a better name.
pub fn json_read_string(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo try to coerce to String.

    let [this] = args else {
        return Err(Error::invalid_arguments(
            "`read` requires `json` argument",
            None,
        ));
    };

    let Expr::String(json) = this.unpack() else {
        return Err(Error::invalid_arguments(
            "`json` argument should be a String",
            this.range(),
        ));
    };

    let Ok(value) = serde_json::from_str::<Value>(json) else {
        // #todo what is the correct error type?
        return Err(Error::invalid_arguments(
            "`json` string is not valid JSON",
            this.range(),
        ));
    };

    Ok(json_value_to_expr(value))
}

pub fn setup_lib_codec_json2(context: &mut Context) {
    let module = Module::new("json-codec", context.top_scope.clone());

    let scope = &module.scope;

    // #todo find a good path and name.
    // #todo codec or serder? codec is more general.
    // #todo use `encode`/`decode` instead of `write`/`read`

    // (use /codec/json-codec)
    // (let value (/json-codec/read json))
    scope.insert("read", Expr::ForeignFunc(Arc::new(json_read_string)));

    // #todo this is a hack.
    let module_path = format!("{}/@std/codec/json-codec", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}

pub fn setup_lib_codec_json(context: &mut Context) {
    // #todo find a good path and name.
    // #todo codec or serder? codec is more general.
    // #todo use `encode`/`decode` instead of `write`/`read`

    let module = require_module("codec/json-codec", context);

    // (use codec/json-codec)
    // (let value (json-codec/read json))
    module.insert("read", Expr::ForeignFunc(Arc::new(json_read_string)));
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::{
        api::eval_string,
        context::Context,
        expr::{format_value, Expr},
    };

    #[test]
    fn json_read_string_usage() {
        // #todo how to avoide `\"` escaping?
        // #todo consider separate namespace for module names and paths? then we could have json-codec -> json

        let input = r#"
            (use /codec/json-codec)
            (let json "{ \"name\": \"George\", \"balance\": 1022.33 }")
            (json-codec/read json)
        "#;
        let mut context = Context::new();
        let expr = eval_string(input, &mut context).unwrap();

        assert_matches!(expr.unpack(), Expr::Dict(..));

        let Expr::Dict(dict) = expr else {
            panic!("expected Expr::Dict");
        };

        assert_eq!(format_value(&dict.borrow()["name"]), "George");
        assert_eq!(format_value(&dict.borrow()["balance"]), "1022.33");
    }
}
