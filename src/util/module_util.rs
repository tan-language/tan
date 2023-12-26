use std::rc::Rc;

use crate::{context::Context, module::Module};

// #insight the module `name` is the last segment of the module `path`.

// #todo add unit test.
// #todo find better name?
/// Returns a module from the registry. If the module does not exist this function
/// creates it.
pub fn require_module<'a>(path: &str, context: &'a mut Context) -> &'a mut Rc<Module> {
    // #todo better error handling here.
    let name = path.split("/").last().expect("invalid module path");

    let module = Module::new(name, context.top_scope.clone());

    // #todo this is a hack.
    let url = format!("{}/@std/{}", context.root_path, path);

    // #todo introduce a helper for this.
    context.module_registry.insert(url.clone(), Rc::new(module)); // #todo use Arc everywhere!
    context.module_registry.get_mut(&url).unwrap()
}
