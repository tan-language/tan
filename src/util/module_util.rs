use std::sync::Arc;

use crate::{context::Context, eval::util::canonicalize_path, module::Module};

// #insight the module `name` is the last segment of the module `path`.

// #todo move to context, as method
// #todo add unit test.
// #todo find better name?
/// Returns a module from the registry. If the module does not exist this function
/// creates it.
pub fn require_module<'a>(path: &str, context: &'a mut Context) -> &'a mut Arc<Module> {
    // #todo Something like this is in `canonicalize_module_path`, use one method.
    // #todo support leading `/`.
    // #todo extract the url generation.
    // #todo this is a hack.
    // #todo #temp very hackish, non-general.
    let url = if path.starts_with('@') {
        format!("{}/{path}", context.root_path)
    } else {
        format!("{}/@std/{path}", context.root_path)
    };

    // #insight weird sym-linking can fuckup this cannonicalization
    // #todo need more robust canonicalization soultion.
    // #todo rethink about this canonicalization.
    let url = canonicalize_path(url);

    if !context.module_registry.contains_key(&url) {
        // #todo better error handling here.
        let name = path.split('/').last().expect("invalid module path");
        let module = Module::new(name, context.top_scope.clone());
        context
            .module_registry
            .insert(url.clone(), Arc::new(module));
    }

    context.module_registry.get_mut(&url).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::context::Context;

    use super::require_module;

    #[test]
    fn require_module_usage() {
        let mut context = Context::new();
        let module = require_module("system/fs", &mut context);
        assert_eq!(module.stem, "fs");

        let module = require_module("math", &mut context);
        assert_eq!(module.stem, "math");
    }
}
