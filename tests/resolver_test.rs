mod common;

use crate::common::resolve_file;

#[test]
fn resolver_handles_function_definition() {
    let result = resolve_file("function-definition-let.tan");
    // dbg!(&result);
    assert!(result.is_ok());
}
