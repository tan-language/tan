use std::fs;
use std::path::Path;
use std::{rc::Rc, sync::Arc};

use crate::module::Module;
use crate::{context::Context, error::Error, expr::Expr};

// #todo do FFI functions really need an env?
// #todo differentiate pure functions that do not change the env!

// File < Resource
// #todo extract file-system-related functionality to `fs` or even the more general `rs` == resource space.
// #todo consider mapping `:` to `__` and use #[allow(snake_case)]

/// Reads the contents of a text file as a string.
pub fn read_file_to_string(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments(
            "`read_as_string` requires a `path` argument",
            None,
        ));
    };

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a String",
            path.range(),
        ));
    };

    let contents = fs::read_to_string(path)?;

    Ok(Expr::String(contents))
}

// #todo decide on the parameters order.
pub fn write_string_to_file(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [path, content] = args else {
        return Err(Error::invalid_arguments(
            "`read_as_string` requires `path` and `content` arguments",
            None,
        ));
    };

    let Expr::String(path) = path.unpack() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a String",
            path.range(),
        ));
    };

    let Expr::String(content) = content.unpack() else {
        return Err(Error::invalid_arguments(
            "`content` argument should be a String",
            content.range(),
        ));
    };

    fs::write(path, content)?;

    Ok(Expr::One)
}

// #todo improve
// #todo follow symlinks
// #todo include dot-files
// #ai-generated
// fn walk_dir_nested(dir_path: &Path) -> Vec<Expr> {
//     let mut tree: Vec<Expr> = Vec::new();

//     // #todo ugh remove all unwraps!
//     for entry in fs::read_dir(dir_path).unwrap() {
//         let entry_path = entry.unwrap().path();

//         if entry_path.is_dir() {
//             // #insight returns nested structure.
//             let mut dir_tree: Vec<Expr> = Vec::new();
//             dir_tree.push(Expr::String(
//                 entry_path
//                     .file_name()
//                     .unwrap()
//                     .to_str()
//                     .unwrap()
//                     .to_string(),
//             ));
//             dir_tree.append(&mut walk_dir_nested(&entry_path));
//             tree.push(Expr::List(dir_tree));
//         } else {
//             tree.push(Expr::String(entry_path.to_str().unwrap().to_string()));
//         }
//     }

//     tree
// }

/// Returns flat structure.
fn walk_dir(dir_path: &Path) -> Vec<Expr> {
    let mut tree: Vec<Expr> = Vec::new();

    // #todo ugh remove all unwraps!
    for entry in fs::read_dir(dir_path).unwrap() {
        let entry_path = entry.unwrap().path();

        if entry_path.is_dir() {
            let dir_name = entry_path.to_str().unwrap().to_string();
            tree.push(Expr::String(format!("{dir_name}/")));
            tree.append(&mut walk_dir(&entry_path));
        } else {
            tree.push(Expr::String(entry_path.to_str().unwrap().to_string()));
        }
    }

    tree
}

// #todo should return nested or flat structure?
// #todo find a better name: walk-as-tree, build-tree
// #todo implement as generator/iterator, or (and?) with callback.
// (let tree (fs/list-as-tree "./source/"))
pub fn list_as_tree(args: &[Expr], _context: &Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments(
            "`list_as_tree` requires a `path` argument",
            None,
        ));
    };

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a String",
            path.range(),
        ));
    };

    let tree = Expr::List(walk_dir(Path::new(path)));

    Ok(tree)
}

// #todo use Rc/Arc consistently
// #todo some helpers are needed here, to streamline the code.

pub fn setup_std_fs(context: &mut Context) {
    let module = Module::new("fs", context.top_scope.clone());

    let scope = &module.scope;

    scope.insert(
        "read-file-to-string",
        Expr::ForeignFunc(Arc::new(read_file_to_string)),
    );
    scope.insert(
        "read-file-to-string$$String",
        Expr::ForeignFunc(Arc::new(read_file_to_string)),
    );

    // #todo consider just `write`.
    scope.insert(
        // #todo alternatives: "std:fs:write_string", "std:url:write_string", "str.url.write-string"
        "write-string-to-file",
        Expr::ForeignFunc(Arc::new(write_string_to_file)),
    );

    scope.insert(
        "write-string-to-file$$String",
        Expr::ForeignFunc(Arc::new(write_string_to_file)),
    );

    scope.insert(
        // #todo alternatives: "std:fs:write_string", "std:url:write_string", "str.url.write-string"
        "write-string-to-file",
        Expr::ForeignFunc(Arc::new(write_string_to_file)),
    );

    // #todo find better name.
    scope.insert("list-as-tree", Expr::ForeignFunc(Arc::new(list_as_tree)));

    // #todo this is a hack.
    let module_path = format!("{}/std/fs", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}
