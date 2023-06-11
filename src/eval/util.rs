use std::{fs, path::Path};

use crate::{
    ann::Ann,
    api::{eval_string, has_tan_extension, resolve_string},
    error::Error,
    expr::Expr,
    range::Ranged,
};

use super::{env::Env, eval};

// #TODO this needs _serious_ cleanup.

// #TODO error handling!
fn eval_file(path: impl AsRef<Path>) -> Result<Ann<Expr>, Vec<Ranged<Error>>> {
    let input = std::fs::read_to_string(path).expect("cannot read input");

    let mut env = Env::prelude();

    eval_string(&input, &mut env)
}

pub fn eval_module(path: impl AsRef<Path>) -> Result<Ann<Expr>, Vec<Ranged<Error>>> {
    // let mut errors: Vec<Ranged<Error>> = Vec::new();

    let path = path.as_ref();

    // #TODO also try to automatically add the .tan or emoji extension.

    if !path.exists() {
        // #TODO emit Error for this
        // #TODO how to handle errors not attached to source code?
        eprintln!("Path `{}` does not exist.", path.display());
    } else if has_tan_extension(&path) {
        eval_file(&path)?;
    } else if path.is_dir() {
        // #TODO report error if it's not a directory but a file with unsupported extension.
        // #TODO not working correctly yet, need to passes, first definitions, then eval.
        let file_paths = fs::read_dir(path);
        let Ok(file_paths) = file_paths else {
            return Err(vec![file_paths.unwrap_err().into()]);
        };

        let mut resolved_exprs: Vec<Ann<Expr>> = Vec::new();

        let mut env = Env::prelude();

        for file_path in file_paths {
            let Ok(file_path) = file_path else {
                return Err(vec![file_path.unwrap_err().into()]);
            };

            let path = file_path.path();

            if !has_tan_extension(&path) {
                continue;
            }

            // #TODO use eval_file here!
            // #TODO handle the range of the error.
            let input = std::fs::read_to_string(path);
            let Ok(input) = input else {
                return Err(vec![input.unwrap_err().into()]);
            };

            let result = resolve_string(input, &mut env);

            let Ok(mut exprs) = result else {
                let err = result.unwrap_err();
                // #TODO better error handling here!
                dbg!(&err);
                // #TODO maybe continue parsing/resolving to find more errors?
                // #TODO better error here!
                return Err(vec![crate::error::Error::FailedUse.into()]);
            };

            resolved_exprs.append(&mut exprs);
        }

        for expr in resolved_exprs {
            if let Err(err) = eval(&expr, &mut env) {
                // #TODO better error handling here!
                dbg!(&err);
                // #TODO better error here!
                return Err(vec![crate::error::Error::FailedUse.into()]);
            }
        }
    } else {
        // #TODO emit Error for this
        eprintln!(
            "Path `{}` is not a valid module, unrecognized extension.",
            path.display()
        );
    }

    // #TODO what should we return here?
    Ok(Expr::One.into())
}
