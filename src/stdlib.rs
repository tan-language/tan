pub mod fs;
pub mod io;
pub mod prelude;
pub mod process;

use std::{rc::Rc, sync::Arc};

use crate::{
    context::Context,
    expr::Expr,
    module::Module,
    stdlib::fs::{file_read_as_string, file_write_string},
};
use crate::{
    expr::annotate_type,
    ops::{
        arithmetic::{
            add_float, add_int, cos_float, div_float, mul_float, mul_int, powi_float, sin_float,
            sub_float, sub_int,
        },
        eq::{eq, gt, lt},
        seq::array_join,
        string::{char_uppercased, format, string_chars, string_constructor_from_chars},
    },
};

use self::{
    io::{write, writeln},
    process::exit,
};

// #todo consider removing the `std` prefix from module paths, like haskell.
// #todo find a better prefix than setup_
// #todo use Rc/Arc consistently
// #todo some helpers are needed here, to streamline the code.

pub fn setup_std_fs(context: &mut Context) {
    let module = Module::new("fs", context.top_scope.clone());

    let scope = &module.scope;

    scope.insert(
        "read-file-to-string",
        Expr::ForeignFunc(Arc::new(file_read_as_string)),
    );
    scope.insert(
        "read-file-to-string$$String",
        Expr::ForeignFunc(Arc::new(file_read_as_string)),
    );

    // #TODO consider just `write`.
    scope.insert(
        // #TODO alternatives: "std:fs:write_string", "std:url:write_string", "str.url.write-string"
        "write-string-to-file",
        Expr::ForeignFunc(Arc::new(file_write_string)),
    );

    scope.insert(
        "write-string-to-file$$String",
        Expr::ForeignFunc(Arc::new(file_write_string)),
    );

    // #todo this is a hack.
    let module_path = format!("{}/std/fs", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}

pub fn setup_std_process(context: &mut Context) {
    let module = Module::new("process", context.top_scope.clone());

    let scope = &module.scope;

    scope.insert("exit", Expr::ForeignFunc(Arc::new(exit)));
    scope.insert("exit$$", Expr::ForeignFunc(Arc::new(exit)));

    // #todo this is a hack.
    let module_path = format!("{}/std/process", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}

pub fn setup_std_prelude(context: &mut Context) {
    let module = Module::new("prelude", context.top_scope.clone());

    let scope = &module.scope;

    // num

    // #TODO forget the mangling, implement with a dispatcher function, multi-function.
    scope.insert(
        "+",
        annotate_type(Expr::ForeignFunc(Arc::new(add_int)), "Int"),
    );
    scope.insert(
        "+$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(add_int)), "Int"),
    );
    scope.insert(
        "+$$Float$$Float",
        // #TODO add the proper type: (Func Float Float Float)
        // #TODO even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(add_float)), "Float"),
    );
    scope.insert("-", Expr::ForeignFunc(Arc::new(sub_int)));
    scope.insert(
        "-$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(sub_int)), "Int"),
    );
    scope.insert(
        "-$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(sub_float)), "Float"),
    );
    scope.insert("*", Expr::ForeignFunc(Arc::new(mul_int)));
    scope.insert(
        "*$$Int$$Int",
        annotate_type(Expr::ForeignFunc(Arc::new(mul_int)), "Int"),
    );
    scope.insert(
        "*$$Float$$Float",
        // #TODO add the proper type: (Func Float Float Float)
        // #TODO even better: (Func (Many Float) Float)
        annotate_type(Expr::ForeignFunc(Arc::new(mul_float)), "Float"),
    );
    scope.insert(
        "/",
        annotate_type(Expr::ForeignFunc(Arc::new(div_float)), "Float"),
    );
    // #TODO ultra-hack
    scope.insert(
        "/$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(div_float)), "Float"),
    );
    // #TODO ultra-hack
    scope.insert(
        "/$$Float$$Float$$Float",
        annotate_type(Expr::ForeignFunc(Arc::new(div_float)), "Float"),
    );
    scope.insert(
        "sin",
        annotate_type(Expr::ForeignFunc(Arc::new(sin_float)), "Float"),
    );
    scope.insert(
        "cos",
        annotate_type(Expr::ForeignFunc(Arc::new(cos_float)), "Float"),
    );
    scope.insert(
        "**",
        annotate_type(Expr::ForeignFunc(Arc::new(powi_float)), "Float"),
    );

    // eq

    scope.insert("=", Expr::ForeignFunc(Arc::new(eq)));
    scope.insert(">", Expr::ForeignFunc(Arc::new(gt)));
    scope.insert("<", Expr::ForeignFunc(Arc::new(lt)));

    // io

    // #todo grab those from /std/io module

    scope.insert("write", Expr::ForeignFunc(Arc::new(write)));
    scope.insert("write$$String", Expr::ForeignFunc(Arc::new(write)));

    scope.insert("writeln", Expr::ForeignFunc(Arc::new(writeln)));
    scope.insert("writeln$$String", Expr::ForeignFunc(Arc::new(writeln)));

    // seq

    scope.insert("join", Expr::ForeignFunc(Arc::new(array_join)));

    // string

    scope.insert(
        "String",
        Expr::ForeignFunc(Arc::new(string_constructor_from_chars)),
    );
    // env.insert("String$$Array", Expr::ForeignFunc(Arc::new(string_constructor_from_chars)));

    scope.insert("chars", Expr::ForeignFunc(Arc::new(string_chars)));
    scope.insert("chars$$String", Expr::ForeignFunc(Arc::new(string_chars)));

    scope.insert("uppercased", Expr::ForeignFunc(Arc::new(char_uppercased)));
    scope.insert(
        "uppercases$$Char",
        Expr::ForeignFunc(Arc::new(char_uppercased)),
    );

    scope.insert("format", Expr::ForeignFunc(Arc::new(format)));

    // #todo this is a hack.
    let module_path = format!("{}/std/prelude", context.root_path);
    // #todo introduce a helper for this.
    context.module_registry.insert(module_path, Rc::new(module));
}

pub fn setup_std(context: &mut Context) {
    setup_std_fs(context);
    setup_std_process(context);
    setup_std_prelude(context);
}
