mod common;

use crate::common::resolve_file;

#[test]
fn resolver_handles_function_definition() {
    let result = resolve_file("function-definition-let.tan");
    assert!(result.is_ok());
}
