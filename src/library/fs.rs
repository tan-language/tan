use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::error::ErrorVariant;
use crate::expr::annotate_type;
use crate::util::expect_lock_write;
use crate::util::module_util::require_module;
use crate::{context::Context, error::Error, expr::Expr};

// #todo consider system/fs, host/fs, os/fs.

// #todo do FFI functions really need an env?
// #todo differentiate pure functions that do not change the env!
// #todo add `symbolic-link?` or `sym-link?` or even `link?` (symbolic is confusing)
// #todo how does windows name symbolic links?

// #todo add a function for file-metadata
// #todo add a function for sym-link target

// #todo consider relationship with a `shell` package.

// #todo extract File functions to separate file?

// File < Resource
// #todo extract file-system-related functionality to `fs` or even the more general `rs` == resource space.
// #todo consider mapping `:` to `__` and use #[allow(snake_case)]

pub fn fs_create(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments("requires a `path` argument", None));
    };

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments(
            "path=`{path}` is not a String",
            path.range(),
        ));
    };

    let file = std::fs::File::create(path)?;

    let expr = Expr::ForeignStructMut(Arc::new(RwLock::new(file)));

    Ok(annotate_type(expr, "File"))
}

// #todo implement fs/close -> should be the same as drop$$File

pub fn file_write_string(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [file, string] = args else {
        return Err(Error::invalid_arguments(
            "requires `file` and `string` arguments",
            None,
        ));
    };

    let Expr::ForeignStructMut(s) = file.unpack() else {
        return Err(Error::invalid_arguments("invalid File", None));
    };

    let s = expect_lock_write(s);

    let Some(mut file) = s.downcast_ref::<std::fs::File>() else {
        return Err(Error::invalid_arguments("invalid File", None));
    };

    let Some(string) = string.as_string() else {
        return Err(Error::invalid_arguments(
            &format!("string=`{string}` is not a String"),
            string.range(),
        ));
    };

    file.write_all(string.as_bytes())?;

    Ok(Expr::None)
}

/// Reads the contents of a text file as a string.
/// ```tan
/// (let content (read-file-to-string "index.html"))
/// ```
pub fn read_file_to_string(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments(
            "`read_as_string` requires a `path` argument",
            None,
        ));
    };

    let path_range = path.range();

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a String",
            path.range(),
        ));
    };

    // #todo investigate if there is an error crate for annotating errors.

    match fs::read_to_string(path) {
        Ok(contents) => Ok(Expr::String(contents)),
        Err(io_error) => {
            let mut error = Error::new(ErrorVariant::Io(io_error));
            error.push_note(&format!("while reading `{path}`"), path_range);
            Err(error)
        }
    }
}

// #todo decide on the parameters order.
// (fs/write-string-to-file "path/to/file.text" "Hello world")
pub fn write_string_to_file(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [path, content] = args else {
        return Err(Error::invalid_arguments(
            "`write-string-to-file` requires `path` and `content` arguments",
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

    Ok(Expr::None)
}

// #todo use walkdir crate instead!
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
fn walk_dir(dir_path: &Path) -> Result<Vec<Expr>, std::io::Error> {
    let mut tree: Vec<Expr> = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry_path = entry?.path();
        // #todo investigate this unwrap.
        let entry_path_str = entry_path.to_str().unwrap().to_string();

        if entry_path.is_dir() {
            tree.push(Expr::String(format!("{entry_path_str}/")));
            tree.append(&mut walk_dir(&entry_path)?);
        } else {
            tree.push(Expr::String(entry_path_str));
        }
    }

    Ok(tree)
}

// #todo
pub fn list(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments(
            "`list` requires a `path` argument",
            None,
        ));
    };

    // #todo should be Stringable
    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a String",
            path.range(),
        ));
    };

    let mut list: Vec<Expr> = Vec::new();

    // #todo ugh remove all unwraps!
    for entry in fs::read_dir(path).unwrap() {
        let entry_path = entry.unwrap().path();

        // #todo should this also include dirs?
        if !entry_path.is_dir() {
            // #todo annotate with `File-Path`
            list.push(Expr::String(entry_path.to_str().unwrap().to_string()));
        }

        // #todo #fix this skips the directories, also add dirs!
        // #todo add trailing slash to dirs and push them to the list?
        // else {
        //     // #todo annotate with `Dir-Path``
        //     let dir_name = entry_path.to_str().unwrap().to_string();
        //     list.push(Expr::String(format!("{dir_name}/")));
        // }
    }

    Ok(Expr::array(list))
}

// #todo can be SLOW for large directories!
// #todo try to optimize!
// #todo should return nested or flat structure?
// #todo find a better name: walk-as-tree, build-tree
// #todo implement as generator/iterator, or (and?) with callback.
// (let tree (fs/list-as-tree "./source/"))
pub fn list_as_tree(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
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

    match walk_dir(Path::new(path)) {
        // #todo should return Array?
        Ok(entries) => Ok(Expr::List(entries)),
        Err(io_error) => {
            // #todo
            // in the future it should return a Tan 'Result' for the caller to handle.
            // for the moment we just panic.
            // Err(Error::io(io_error, &format!("path: {path}"), None))
            Err(Error::panic_with_context(
                &format!("while walking `{path}`: {}", io_error),
                context,
            ))
        }
    }
}

/// Checks if a path exists.
pub fn fs_exists(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments(
            "`exists?` requires a `path` argument",
            None,
        ));
    };

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a String",
            path.range(),
        ));
    };

    let exists = std::fs::metadata(path).is_ok();

    Ok(Expr::Bool(exists))
}

// #todo move
// #todo delete (or remove?)

// #todo support paths
pub fn fs_copy(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [source, target] = args else {
        return Err(Error::invalid_arguments(
            "`copy` requires `source` and `target` arguments",
            None,
        ));
    };

    let Some(source) = source.as_string() else {
        return Err(Error::invalid_arguments(
            "`source` argument should be a String",
            source.range(),
        ));
    };

    let Some(target) = target.as_string() else {
        return Err(Error::invalid_arguments(
            "`target` argument should be a String",
            target.range(),
        ));
    };

    let bytes_count = fs::copy(source, target)?;

    // #todo what to return?
    Ok(Expr::Int(bytes_count as i64))
}

// #ai
// #todo find a good name that denotes that this is foreign.
// #extract to a utility?
fn copy_dir(source: impl AsRef<Path>, target: impl AsRef<Path>) -> std::io::Result<()> {
    let source = source.as_ref();
    let target = target.as_ref();

    // #todo consider actually NOT using create_dir_all here, just create_dir.
    // #insight create_dir_all is required in the recursive call? no.
    // fs::create_dir_all(target)?;

    // #insight
    // we don't use create_dir_all here to have consistent semantics
    // with copy-file. Use glue/fs-util helper (that leverages ensure-directory)
    // to provide a more resilient version of this function.
    fs::create_dir(target)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_dir(&entry.path(), &target.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), target.join(entry.file_name()))?;
        }
    }
    Ok(())
}

// #insight this
pub fn fs_copy_directory(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [source, target] = args else {
        return Err(Error::invalid_arguments(
            "`copy_directory` requires `source` and `target` arguments",
            None,
        ));
    };

    let Some(source) = source.as_string() else {
        return Err(Error::invalid_arguments(
            "`source` argument should be a String",
            source.range(),
        ));
    };

    let Some(target) = target.as_string() else {
        return Err(Error::invalid_arguments(
            "`target` argument should be a String",
            target.range(),
        ));
    };

    copy_dir(source, target)?;

    // #todo what is a good return value?
    Ok(Expr::None)
}

// #todo consider `make-directory`? (make in process, create in system)
pub fn create_directory(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments(
            "`create-directory` requires a `path` argument",
            None,
        ));
    };

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a String",
            path.range(),
        ));
    };

    // #todo create_dir vs create_dir_all

    fs::create_dir_all(path)?;

    // #todo what to return?
    Ok(Expr::None)
}

// #todo add some kind of unit test for this.
// #todo find a better name, maybe canonicalize-path or normalize-path.
pub fn fs_canonicalize(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [path] = args else {
        return Err(Error::invalid_arguments(
            "`canonicalize` requires a `path` argument",
            None,
        ));
    };

    let Some(path) = path.as_string() else {
        return Err(Error::invalid_arguments(
            "`path` argument should be a String",
            path.range(),
        ));
    };

    let path = fs::canonicalize(path)?;

    // #todo should return a `Path` value.

    Ok(Expr::string(path.to_string_lossy()))
}

// #todo use Rc/Arc consistently
// #todo some helpers are needed here, to streamline the code.

pub fn setup_lib_fs(context: &mut Context) {
    let module = require_module("fs", context);

    module.insert("create", Expr::ForeignFunc(Arc::new(fs_create)));

    // #todo should not be required.
    module.insert(
        "write-string",
        Expr::ForeignFunc(Arc::new(file_write_string)),
    );
    module.insert(
        "write-string$$File$$String",
        Expr::ForeignFunc(Arc::new(file_write_string)),
    );

    module.insert(
        "read-file-to-string",
        Expr::ForeignFunc(Arc::new(read_file_to_string)),
    );
    module.insert(
        "read-file-to-string$$String",
        Expr::ForeignFunc(Arc::new(read_file_to_string)),
    );
    // #todo consider just `write`.
    // #todo alternatives: "std:fs:write_string", "std:url:write_string", "str.url.write-string"
    module.insert(
        "write-string-to-file",
        Expr::ForeignFunc(Arc::new(write_string_to_file)),
    );
    module.insert(
        "write-string-to-file$$String",
        Expr::ForeignFunc(Arc::new(write_string_to_file)),
    );

    // #todo find better name.
    module.insert("list", Expr::ForeignFunc(Arc::new(list)));
    module.insert("list$$String", Expr::ForeignFunc(Arc::new(list)));

    // #todo find better name.
    module.insert("list-as-tree", Expr::ForeignFunc(Arc::new(list_as_tree)));
    module.insert(
        "list-as-tree$$String",
        Expr::ForeignFunc(Arc::new(list_as_tree)),
    );

    module.insert("exists?", Expr::ForeignFunc(Arc::new(fs_exists)));
    module.insert("exists?$$String", Expr::ForeignFunc(Arc::new(fs_exists)));

    module.insert("copy", Expr::ForeignFunc(Arc::new(fs_copy)));

    module.insert(
        "copy-directory",
        Expr::ForeignFunc(Arc::new(fs_copy_directory)),
    );

    module.insert(
        "create-directory",
        Expr::ForeignFunc(Arc::new(create_directory)),
    );

    module.insert("canonicalize", Expr::ForeignFunc(Arc::new(fs_canonicalize)));
}

// #todo add unit tests.
