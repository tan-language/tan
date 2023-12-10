pub mod iterator;
pub mod util;

use std::{collections::HashMap, rc::Rc};

use crate::{
    context::Context,
    error::Error,
    expr::{annotate, expr_clone, format_value, Expr},
    resolver::compute_dyn_signature,
    scope::Scope,
    util::is_reserved_symbol,
};

use self::{iterator::try_iterator_from, util::eval_module};

// #Insight
// _Not_ a pure evaluator, performs side-effects.

// #Insight
// I don't like the name `interpreter`.

// #todo move excessive error-checking/linting to the resolve/typecheck pass.
// #todo encode effects in the type-system.
// #todo alternative names: Processor, Runner, Interpreter
// #todo split eval_special, eval_func -> not needed if we put everything uniformly in prelude.
// #todo Stack-trace is needed!
// #todo https://clojure.org/reference/evaluation

// #todo try to remove non-needed .into()s <--

// #todo give more 'general' name.
// #todo what about if a required argument is not passed to a function? currently we report undefined symbol.
fn eval_args(args: &[Expr], context: &mut Context) -> Result<Vec<Expr>, Error> {
    args.iter()
        .map(|x| eval(x, context))
        .collect::<Result<Vec<_>, _>>()
}

// fn eval_quote(expr: Expr, context: &mut Context) -> Expr {
//     // #insight unpack is OK, no extract needed.
//     println!("fquot --> {}", expr.unpack());
//     match expr.unpack() {
//         Expr::List(terms) => {
//             if terms.is_empty() {
//                 expr
//             } else {
//                 if let Some(sym) = terms[0].unpack().as_symbol() {
//                     if sym == "unquot" {
//                         debug_assert!(terms.len() == 2);
//                         // #todo remove the unwrap!
//                         println!("evan unquot ::: {}", &terms[1].unpack());
//                         eval(&terms[1], context).unwrap()
//                     } else {
//                         expr
//                     }
//                 } else {
//                     expr
//                 }
//             }
//         }
//         _ => expr,
//     }
// }

// #todo needs better conversion to Expr::Annotated

/// Evaluates via expression rewriting. The expression `expr` evaluates to
/// a fixed point. In essence this is a 'tree-walk' interpreter.
pub fn eval(expr: &Expr, context: &mut Context) -> Result<Expr, Error> {
    match expr.unpack() {
        // #todo are you sure?
        // Expr::Annotated(..) => eval(expr.unpack(), env),
        Expr::Symbol(symbol) => {
            // #todo differentiate between evaluating symbol in 'op' position.

            if is_reserved_symbol(symbol) {
                return Ok(expr.clone());
            }

            // #todo handle 'PathSymbol'

            // #todo try to populate "method"/"signature" annotations during
            // #todo this is missing now that we don't have the resolve stage.
            // #todo maybe resolve or optimize should already have placed the method in the AST?

            let value = if let Some(Expr::Symbol(method)) = expr.annotation("method") {
                // If the symbol is annotated with a `method`, it's in 'operator' position.
                // `method` is just one of the variants of a multi-method-function.
                // println!("--> {method}");
                if let Some(value) = context.scope.get(method) {
                    value
                } else {
                    // #todo leave this trace on in some kind of debug mode.
                    // println!("--> method-fallback");
                    // #todo ultra-hack, if the method is not found, try to lookup the function symbol, fall-through.
                    // #todo should do proper type analysis here.

                    context
                        .scope
                        .get(symbol)
                        .ok_or::<Error>(Error::undefined_function(
                            symbol,
                            method,
                            &format!("undefined function `{symbol}` with signature `{method}"),
                            expr.range(),
                        ))?
                }
            } else {
                context
                    .scope
                    .get(symbol)
                    .ok_or::<Error>(Error::undefined_symbol(
                        &symbol,
                        &format!("symbol not defined: `{symbol}`"),
                        expr.range(),
                    ))?
            };

            // #todo hm, can we somehow work with references?
            // #hint this could help: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.unwrap_or_clone

            Ok(expr_clone(&value))
        }
        Expr::KeySymbol(..) => {
            // #todo handle 'PathSymbol'
            // #todo strip annotation?

            // #todo lint '::' etc.
            // #todo check that if there is a leading ':' there is only one ':', make this a lint warning!
            // #todo consider renaming KeywordSymbol to KeySymbol.

            // A `Symbol` that starts with `:` is a so-called `KeywordSymbol`. Keyword
            // symbols evaluate to themselves, and are convenient to use as Map keys,
            // named (keyed) function parameter, enum variants, etc.
            Ok(expr.clone())
        }
        // #todo if is unquotable!!
        Expr::If(predicate, true_clause, false_clause) => {
            let predicate = eval(predicate, context)?;

            let Some(predicate) = predicate.as_bool() else {
                return Err(Error::invalid_arguments(
                    "the if predicate is not a boolean value",
                    predicate.range(),
                ));
            };

            if predicate {
                eval(true_clause, context)
            } else if let Some(false_clause) = false_clause {
                eval(false_clause, context)
            } else {
                // #todo what should we return if there is no false-clause? Zero/Never?
                Ok(Expr::One.into())
            }
        }
        Expr::List(list) => {
            // #todo no need for dynamic invocable, can use (apply f ...) / (invoke f ...) instead.
            // #todo replace head/tail with first/rest

            if list.is_empty() {
                // () == One (Unit)
                // This is handled statically, in the parser, but an extra, dynamic
                // check is needed in the evaluator to handle the case where the
                // expression is constructed programmatically (e.g. self-modifying code,
                // dynamically constructed expression, homoiconicity, etc).
                return Ok(Expr::One.into());
            }

            // The unwrap here is safe.
            let head = list.first().unwrap();
            let tail = &list[1..];

            // #todo could check special forms before the eval

            // #todo this is an ULTRA-HACK! SUPER NASTY/UGLY CODE, refactor!

            // Evaluate the head, try to find dynamic signature
            let head = if let Some(name) = head.as_symbol() {
                if !is_reserved_symbol(name) {
                    // #todo super nasty hack!!!!
                    let args = eval_args(tail, context)?;

                    if let Some(value) = context.scope.get(name) {
                        if let Expr::Func(params, ..) = value.unpack() {
                            // #todo ultra-hack to kill shared ref to `env`.
                            let params = params.clone();

                            let prev_scope = context.scope.clone();
                            context.scope = Rc::new(Scope::new(prev_scope.clone()));

                            for (param, arg) in params.iter().zip(&args) {
                                let Some(param) = param.as_symbol() else {
                                    return Err(Error::invalid_arguments(
                                        "parameter is not a symbol",
                                        param.range(),
                                    ));
                                };

                                context.scope.insert(param, arg.clone());
                            }

                            let signature = compute_dyn_signature(&args, context);
                            let head = annotate(
                                head.clone(),
                                "method",
                                Expr::Symbol(format!("{name}$${signature}")),
                            );
                            let head = eval(&head, context)?;

                            context.scope = prev_scope;

                            head
                        } else if let Expr::ForeignFunc(_) = value.unpack() {
                            let signature = compute_dyn_signature(&args, &context);
                            let head = annotate(
                                head.clone(),
                                "method",
                                Expr::Symbol(format!("{name}$${signature}")),
                            );
                            let head = eval(&head, context)?;

                            head
                        } else {
                            eval(head, context)?
                        }
                    } else {
                        eval(head, context)?
                    }
                } else {
                    eval(head, context)?
                }
            } else {
                eval(head, context)?
            };

            // #todo move special forms to prelude, as Expr::Macro or Expr::Special

            match head.unpack() {
                Expr::Func(params, body, func_scope) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, context)?;

                    // #todo ultra-hack to kill shared ref to `env`.
                    let params = params.clone();

                    // Dynamic scoping, #todo convert to lexical.

                    let prev_scope = context.scope.clone();
                    context.scope = Rc::new(Scope::new(func_scope.clone())); // #insight notice we use func_scope here!

                    for (param, arg) in params.iter().zip(args) {
                        let Some(param) = param.as_symbol() else {
                            return Err(Error::invalid_arguments(
                                "parameter is not a symbol",
                                param.range(),
                            ));
                        };

                        context.scope.insert(param, arg);
                    }

                    // #todo this code is the same as in the (do ..) block, extract.

                    // #todo do should be 'monadic', propagate Eff (effect) wrapper.
                    let mut value = Expr::One;

                    for expr in body {
                        value = eval(expr, context)?;
                    }

                    context.scope = prev_scope;

                    Ok(value)
                }
                Expr::ForeignFunc(foreign_function) => {
                    // #todo do NOT pre-evaluate args for ForeignFunc, allow to implement 'macros'.
                    // Foreign Functions do NOT change the environment, hmm...
                    // #todo use RefCell / interior mutability instead, to allow for changing the environment (with Mutation Effect)

                    // println!("--> {:?}", context.scope);

                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, context)?;

                    let result = foreign_function(&args, context);

                    // If the error has no range, try to apply the range of the invocation.
                    if let Err(mut error) = result {
                        if let Some(note) = error.notes.first_mut() {
                            if note.range.is_none() {
                                note.range = expr.range()
                            }
                        };

                        return Err(error);
                    };

                    result
                }
                Expr::Array(arr) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, context)?;

                    // #todo optimize this!
                    // #todo error checking, one arg, etc.
                    let index = &args[0];
                    // #todo we need UInt, USize, Nat type
                    let Some(index) = index.as_int() else {
                        return Err(Error::invalid_arguments(
                            "invalid array index, expecting Int",
                            index.range(),
                        ));
                    };
                    let index = index as usize;
                    if let Some(value) = arr.borrow().get(index) {
                        // #todo replace the clone with the custom expr::copy/ref
                        Ok(value.clone().into())
                    } else {
                        // #todo introduce Maybe { Some, None }
                        Ok(Expr::One.into())
                    }
                }
                Expr::Dict(dict) => {
                    // Evaluate the arguments before calling the function.
                    let args = eval_args(tail, context)?;

                    // #todo optimize this!
                    // #todo error checking, one arg, stringable, etc.

                    // #insight no need to unpack, format_value sees-through.
                    let key = format_value(&args[0]);
                    if let Some(value) = dict.borrow().get(&key) {
                        Ok(value.clone().into())
                    } else {
                        // #todo introduce Maybe { Some, None }
                        Ok(Expr::One.into())
                    }
                }
                // #todo add handling of 'high-level', compound expressions here.
                // #todo Expr::If
                // #todo Expr::Let
                // #todo Expr::Do
                // #todo Expr::..
                Expr::Symbol(s) => {
                    match s.as_str() {
                        // special term
                        // #todo the low-level handling of special forms should use the above high-level cases.
                        // #todo use the `optimize`/`raise` function, here to prepare high-level expression for evaluation, to avoid duplication.
                        "do" => {
                            // #todo do should be 'monadic', propagate Eff (effect) wrapper.
                            let mut value = Expr::One.into();

                            // #todo extract this.

                            let prev_scope = context.scope.clone();
                            context.scope = Rc::new(Scope::new(prev_scope.clone()));

                            for expr in tail {
                                value = eval(expr, context)?;
                            }

                            context.scope = prev_scope;

                            Ok(value)
                        }
                        "ann" => {
                            // #Insight implemented as special-form because it applies to Ann<Expr>.
                            // #todo try to implement as ForeignFn

                            if tail.len() != 1 {
                                return Err(Error::invalid_arguments(
                                    "`ann` requires one argument",
                                    expr.range(),
                                ));
                            }

                            // #todo support multiple arguments.

                            let expr = tail.first().unwrap();

                            if let Some(ann) = expr.annotations() {
                                Ok(Expr::dict(ann.clone()))
                            } else {
                                Ok(Expr::dict(HashMap::new()))
                            }
                        }
                        "eval" => {
                            let [expr] = tail else {
                                return Err(Error::invalid_arguments(
                                    "missing expression to be evaluated",
                                    expr.range(),
                                ));
                            };

                            // #todo consider naming this `form`?
                            let expr = eval(expr, context)?;

                            eval(&expr, context)
                        }
                        "quot" => {
                            // #insight not obvious how to move to static/comptime phase.
                            // #todo doesn't quote all exprs, e.g. the if expression.
                            // #todo optimize with custom exprs, e.g Expr::Quot, Expr::QuasiQuot, etc.

                            let [value] = tail else {
                                return Err(Error::invalid_arguments(
                                    "missing quote target",
                                    expr.range(),
                                ));
                            };

                            // #todo transform_mut is not the correct traversal, it's depth first it should be breadth first.
                            // #todo expr.quote() is a temp hack.
                            Ok(value.clone().quot(context))
                        }
                        // #todo check racket.
                        // #todo implement for->list, for->map, for->fold, etc.
                        "for" => {
                            // #insight
                            // `while` is a generalization of `if`
                            // `for` is a generalization of `let`
                            // `for` is related with `do`
                            // `for` is monadic

                            // (for (x 10) (writeln x))

                            // #todo reuse code from let
                            // #todo the resolver should handle this.

                            if tail.len() < 2 {
                                // #todo add more structural checks.
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "missing for arguments",
                                    expr.range(),
                                ));
                            }

                            let binding = tail.first().unwrap();
                            let body = &tail[1..];

                            // #todo should check both for list and array.
                            let Some(binding_parts) = binding.as_list() else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "invalid for binding",
                                    binding.range(),
                                ));
                            };

                            let [var, value] = &binding_parts[..] else {
                                return Err(Error::invalid_arguments(
                                    "invalid for binding",
                                    binding.range(),
                                ));
                            };

                            let Some(var) = var.as_symbol() else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "invalid for binding, malformed variable",
                                    var.range(),
                                ));
                            };

                            // #insight for the ListIterator
                            let value = eval(value, context)?;

                            // #todo also handle (Range start end step)
                            // #todo maybe step should be external to Range, or use SteppedRange, or (Step-By (Range T))
                            let Some(iterator) = try_iterator_from(&value) else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "invalid for binding, the value is not iterable",
                                    value.range(),
                                ));
                            };

                            let prev_scope = context.scope.clone();
                            context.scope = Rc::new(Scope::new(prev_scope.clone()));

                            let mut iterator = iterator.borrow_mut();

                            while let Some(value) = iterator.next() {
                                context.scope.insert(var, value);
                                for expr in body {
                                    // #insight plain `for` is useful only for the side-effects.
                                    let _ = eval(expr, context)?;
                                }
                            }

                            context.scope = prev_scope;

                            Ok(Expr::One)
                        }
                        // #todo consider the name `for*` or something similar?
                        "for->list" => {
                            // #insight
                            // `while` is a generalization of `if`
                            // `for` is a generalization of `let`
                            // `for` is related with `do`
                            // `for` is monadic

                            // (for (x 10) (writeln x))

                            // #todo solve duplication between for and for->list
                            // #todo reuse code from let
                            // #todo the resolver should handle this.

                            if tail.len() < 2 {
                                // #todo add more structural checks.
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "missing for arguments",
                                    expr.range(),
                                ));
                            }

                            let mut values = Vec::new();

                            let binding = tail.first().unwrap();
                            let body = &tail[1..];

                            // #todo should check both for list and array.
                            let Some(binding_parts) = binding.as_list() else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "invalid for binding",
                                    binding.range(),
                                ));
                            };

                            let [var, value] = &binding_parts[..] else {
                                return Err(Error::invalid_arguments(
                                    "invalid for binding",
                                    binding.range(),
                                ));
                            };

                            let Some(var) = var.as_symbol() else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "invalid for binding, malformed variable",
                                    var.range(),
                                ));
                            };

                            // #insight for the ListIterator
                            let value = eval(value, context)?;

                            // #todo also handle (Range start end step)
                            // #todo maybe step should be external to Range, or use SteppedRange, or (Step-By (Range T))
                            let Some(iterator) = try_iterator_from(&value) else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "invalid for binding, the value is not iterable",
                                    value.range(),
                                ));
                            };

                            let prev_scope = context.scope.clone();
                            context.scope = Rc::new(Scope::new(prev_scope.clone()));

                            let mut iterator = iterator.borrow_mut();

                            while let Some(value) = iterator.next() {
                                context.scope.insert(var, value);
                                for expr in body {
                                    values.push(eval(expr, context)?);
                                }
                            }

                            context.scope = prev_scope;

                            Ok(Expr::array(values))
                        }
                        "while" => {
                            // #insight
                            // `while` is a generalization of `if`
                            // `for` is a generalization of `let`
                            // `for` is related with `do`
                            // `for` is monadic

                            // #todo
                            // try to merge `while` with `for` (maybe `for` is implemented on top of `while`?)

                            let [predicate, body] = tail else {
                                // #todo proper error!
                                return Err(Error::invalid_arguments(
                                    "missing for arguments",
                                    expr.range(),
                                ));
                            };

                            let mut value = Expr::One.into();

                            loop {
                                let predicate = eval(predicate, context)?;

                                let Some(predicate) = predicate.as_bool() else {
                                    return Err(Error::invalid_arguments(
                                        "the `while` predicate is not a boolean value",
                                        predicate.range(),
                                    ));
                                };

                                if !predicate {
                                    break;
                                }

                                value = eval(body, context)?;
                            }

                            Ok(value)
                        }
                        "if" => {
                            // #todo this is a temp hack!
                            let Some(predicate) = tail.get(0) else {
                                return Err(Error::invalid_arguments(
                                    "malformed if predicate",
                                    expr.range(),
                                ));
                            };

                            let Some(true_clause) = tail.get(1) else {
                                return Err(Error::invalid_arguments(
                                    "malformed if true clause",
                                    expr.range(),
                                ));
                            };

                            // #todo don't get false_clause if not required?
                            let false_clause = tail.get(2);

                            let predicate = eval(predicate, context)?;

                            let Some(predicate) = predicate.as_bool() else {
                                return Err(Error::invalid_arguments(
                                    "the if predicate is not a boolean value",
                                    predicate.range(),
                                ));
                            };

                            if predicate {
                                eval(true_clause, context)
                            } else if let Some(false_clause) = false_clause {
                                eval(false_clause, context)
                            } else {
                                // #insight In the Curry–Howard correspondence, an empty type corresponds to falsity.
                                // #insight
                                // Zero / Nothing disallows this:
                                // (let flag (if predicate (+ 1 2))) ; compile error: cannot assign Nothing
                                Ok(Expr::Zero)
                            }
                        }
                        // #todo is this different enough from `if`?
                        // (cond
                        //   (> i 5) (...)
                        //   (> i 15) (...)
                        //   else (...)
                        // )
                        "cond" => {
                            let mut i = 0;

                            loop {
                                if i >= tail.len() {
                                    // #todo what should we return here? probably Never/Zero?
                                    break Ok(Expr::One);
                                }

                                let Some(predicate) = tail.get(i) else {
                                    return Err(Error::invalid_arguments(
                                        "malformed cond predicate",
                                        expr.range(),
                                    ));
                                };

                                let Some(clause) = tail.get(i + 1) else {
                                    return Err(Error::invalid_arguments(
                                        "malformed cond clause",
                                        expr.range(),
                                    ));
                                };

                                // #todo `else` should not be annotated.
                                // #todo should NOT annotate symbols and keysymbols!
                                // #todo introduce a helper to check for specific symbol.

                                if let Expr::Symbol(sym) = predicate.unpack() {
                                    if sym == "else" {
                                        break eval(clause, context);
                                    }
                                }

                                let predicate = eval(predicate, context)?;

                                let Some(predicate) = predicate.as_bool() else {
                                    return Err(Error::invalid_arguments(
                                        "the if predicate is not a boolean value",
                                        predicate.range(),
                                    ));
                                };

                                if predicate {
                                    break eval(clause, context);
                                }

                                i += 2;
                            }
                        }
                        // #todo for-each or overload for?
                        "for-each" => {
                            // #todo this is a temp hack!
                            let [seq, var, body] = tail else {
                                return Err(Error::invalid_arguments(
                                    "malformed `for-each`",
                                    expr.range(),
                                ));
                            };

                            let seq = eval(seq, context)?;

                            let Some(arr) = seq.as_array() else {
                                return Err(Error::invalid_arguments(
                                    "`for-each` requires a `Seq` as the first argument",
                                    seq.range(),
                                ));
                            };

                            let Some(sym) = var.as_symbol() else {
                                return Err(Error::invalid_arguments(
                                    "`for-each` requires a symbol as the second argument",
                                    var.range(),
                                ));
                            };

                            let prev_scope = context.scope.clone();
                            context.scope = Rc::new(Scope::new(prev_scope.clone()));

                            for x in arr.iter() {
                                // #todo array should have Ann<Expr> use Ann<Expr> everywhere, avoid the clones!
                                // #todo replace the clone with custom expr::ref/copy?
                                context.scope.insert(sym, x.clone());
                                eval(body, context)?;
                            }

                            context.scope = prev_scope;

                            // #todo intentionally don't return a value, reconsider this?
                            Ok(Expr::One.into())
                        }
                        // #todo extract
                        // #todo functions implemented here have dynamic dispatch!
                        // #todo show usage in comments
                        "map" => {
                            // #todo this is a temp hack!
                            let [seq, var, body] = tail else {
                                return Err(Error::invalid_arguments(
                                    "malformed `map`",
                                    expr.range(),
                                ));
                            };

                            let seq = eval(seq, context)?;

                            let Some(arr) = seq.as_array() else {
                                return Err(Error::invalid_arguments(
                                    "`map` requires a `Seq` as the first argument",
                                    seq.range(),
                                ));
                            };

                            let Some(sym) = var.as_symbol() else {
                                return Err(Error::invalid_arguments(
                                    "`map` requires a symbol as the second argument",
                                    var.range(),
                                ));
                            };

                            let prev_scope = context.scope.clone();
                            context.scope = Rc::new(Scope::new(prev_scope.clone()));

                            let mut results: Vec<Expr> = Vec::new();

                            for x in arr.iter() {
                                // #todo array should have Ann<Expr> use Ann<Expr> everywhere, avoid the clones!
                                context.scope.insert(sym, x.clone());
                                let result = eval(body, context)?;
                                // #todo replace the clone with custom expr::ref/copy?
                                results.push(result.unpack().clone());
                            }

                            context.scope = prev_scope.clone();

                            // #todo intentionally don't return a value, reconsider this?
                            Ok(Expr::array(results).into())
                        }
                        "set!" => {
                            // #todo find other name: poke, mut, mutate
                            // #todo this is a temp hack
                            // #todo write unit tests
                            // #todo support mutating multiple variables.

                            let [name, value] = tail else {
                                return Err(Error::invalid_arguments(
                                    "malformed `set!`",
                                    expr.range(),
                                ));
                            };

                            let Some(name) = name.try_string() else {
                                return Err(Error::invalid_arguments(
                                    "`set!` requires a symbol as the first argument",
                                    name.range(),
                                ));
                            };

                            let value = eval(value, context)?;

                            context.scope.update(name, value.clone());

                            // #todo what should this return? One/Unit (i.e. nothing useful) or the actual value?
                            Ok(Expr::One)
                        }
                        "scope-update" => {
                            // #todo this name conflicts with scope.update()
                            // #todo consider a function that nests a new scope.
                            // #todo maybe it should be some kind of let? e.g. (`let-all` ?? or `let*` or `let..`)
                            // #todo this is a temp hack.

                            // Updates the scope with bindings of the given dict.

                            let [dict] = tail else {
                                return Err(Error::invalid_arguments(
                                    "malformed `scope-update`",
                                    expr.range(),
                                ));
                            };

                            let dict = eval(dict, context)?;

                            let Some(dict) = dict.as_dict() else {
                                // #todo report what was found!
                                return Err(Error::invalid_arguments(
                                    "malformed `scope-update`, expects Dict argument",
                                    expr.range(),
                                ));
                            };

                            for (name, value) in dict.iter() {
                                // #todo remove clone.
                                context.scope.insert(name, expr_clone(value));
                            }

                            Ok(Expr::One)
                        }
                        "use" => {
                            // #insight modules are (currently) directories, _not_ files.

                            // #todo support single-file modules (xxx.mod.tan, xxx.module.tan)
                            // #todo extract as function
                            // #insight this code is temporarily(?) moved from `resolver` here.
                            // #todo also introduce a dynamic version of `use`.

                            // #todo temp hack!!!

                            // #todo properly handle this here, strip the use expression, remove from eval
                            // #todo move this to resolve? use should be stripped at dyn-time
                            // #todo also support path as symbol.

                            // Import a directory as a module.

                            let Some(term) = tail.get(0) else {
                                return Err(Error::invalid_arguments(
                                    "malformed use expression",
                                    expr.range(),
                                ));
                            };

                            // #todo the formatter should convert string paths to symbols.

                            // #insight support both Strings and Symbols as module paths.
                            // #insight, notice the `try_string`.
                            let Some(module_path) = term.try_string() else {
                                // let Some(module_path) = term.as_string() else {
                                return Err(Error::invalid_arguments(
                                    "malformed use expression",
                                    expr.range(),
                                ));
                            };

                            // #todo make sure paths are relative to the current file.
                            let result = eval_module(module_path, context);

                            if let Err(errors) = result {
                                // #todo precise formating is _required_ here!
                                // eprintln!("{}", format_errors(&errors));
                                // dbg!(errors);
                                // #todo add note with information here!
                                return Err(Error::failed_use(&module_path, errors));
                            };

                            let Ok(Expr::Module(module)) = result else {
                                // #todo could use a panic here, this should never happen.
                                return Err(Error::failed_use(&module_path, vec![]));
                            };

                            if let Some(arg) = tail.get(1) {
                                // (use /math [pi tau]) ; pi
                                // (use /math :only [pi tau]) ; math/pi
                                // (use /math :exclude [pi])
                                // (use /math :as "mathematics") ; mathematics/pi

                                if let Some(names) = arg.as_array() {
                                    // #todo consider (use /math pi tau) -> nah.
                                    // (use /math [pi tau]) ; pi, embed without namespace.
                                    for name in names.iter() {
                                        // #todo ONLY export public bindings
                                        // #todo assign as top-level bindings!
                                        let Some(name) = name.try_string() else {
                                            return Err(Error::invalid_arguments(
                                                "use explicit imports should be Stringables",
                                                expr.range(),
                                            ));
                                        };
                                        let Some(value) = module.scope.get(name) else {
                                            return Err(Error::invalid_arguments(
                                                &format!("undefined import `{name}`"),
                                                expr.range(),
                                            ));
                                        };
                                        context.scope.insert(name, value.clone());
                                    }
                                } else {
                                    return Err(Error::invalid_arguments(
                                        "malformed use expression",
                                        expr.range(),
                                    ));
                                }
                            } else {
                                // Import public names from module scope into the current scope.

                                // #todo support (use "/path/to/module" *) or (use "/path/to/module" :embed)

                                // #todo temp, needs cleanup!
                                let bindings = module.scope.bindings.borrow().clone();
                                for (name, value) in bindings {
                                    // #todo temp fix to not override the special var
                                    if name.starts_with("*") {
                                        continue;
                                    }

                                    // #todo ONLY export public bindings

                                    let name = format!("{}/{}", module.stem, name);

                                    // #todo assign as top-level bindings!
                                    context.scope.insert(name, value.clone());
                                }
                            }

                            // #todo allow for embedding explicit symbols, non-namespaced!

                            // #todo what could we return here? the Expr::Module?
                            // Ok(Expr::Module(module))
                            Ok(Expr::One)
                        }
                        "let" => {
                            // #todo this is already parsed statically by resolver, no need to duplicate the tests here?
                            // #todo also report some of these errors statically, maybe in a sema phase?
                            let mut args = tail.iter();

                            loop {
                                let Some(name) = args.next() else {
                                    break;
                                };

                                let Some(value) = args.next() else {
                                    // #todo error?
                                    break;
                                };

                                let Some(s) = name.as_symbol() else {
                                    return Err(Error::invalid_arguments(
                                        &format!("`{name}` is not a Symbol"),
                                        name.range(),
                                    ));
                                };

                                // #todo do we really want this? Maybe convert to a lint?
                                if is_reserved_symbol(s) {
                                    return Err(Error::invalid_arguments(
                                        &format!("let cannot shadow the reserved symbol `{s}`"),
                                        name.range(),
                                    ));
                                }

                                let value = eval(value, context)?;

                                // #todo notify about overrides? use `set`?
                                context.scope.insert(s, value);
                            }

                            // #todo return last value!
                            Ok(Expr::One.into())
                        }
                        "not" => {
                            // #todo make a function
                            // #todo consider binary/bitmask version.
                            // #todo consider operator `~` (_not_ `!`)

                            let [arg] = tail else {
                                return Err(Error::invalid_arguments(
                                    "`not` expects one argument",
                                    expr.range(),
                                ));
                            };

                            let value = eval(arg, context)?;

                            let Some(predicate) = value.as_bool() else {
                                return Err(Error::invalid_arguments(
                                    "`not` argument should be boolean",
                                    expr.range(),
                                ));
                            };

                            Ok(Expr::Bool(!predicate))
                        }
                        "and" => {
                            // #todo what about binary and?
                            // #todo consider operator form? `&&` or `*`
                            // #todo optimize case with 2 arguments.
                            // #insight `and` is not short-circuiting
                            // #todo make a function?
                            // #todo should these 'special forms' get added in scope/env?

                            for arg in tail {
                                let value = eval(arg, context)?;
                                let Some(predicate) = value.as_bool() else {
                                    return Err(Error::invalid_arguments(
                                        "`and` argument should be boolean",
                                        expr.range(),
                                    ));
                                };

                                if !predicate {
                                    return Ok(Expr::Bool(false));
                                }
                            }

                            Ok(Expr::Bool(true))
                        }
                        "or" => {
                            // #todo what about binary or?
                            // #todo consider operator form? `||` or `+`
                            // #insight `or` is short-circuiting so it cannot be implemented as a function

                            for arg in tail {
                                let value = eval(arg, context)?;
                                let Some(predicate) = value.as_bool() else {
                                    return Err(Error::invalid_arguments(
                                        "`or` argument should be boolean",
                                        expr.range(),
                                    ));
                                };

                                if predicate {
                                    return Ok(Expr::Bool(true));
                                }
                            }

                            Ok(Expr::Bool(false))
                        }
                        "Char" => {
                            // #todo report more than 1 arguments.

                            let Some(arg) = tail.get(0) else {
                                return Err(Error::invalid_arguments(
                                    "malformed Char constructor, missing argument",
                                    expr.range(),
                                ));
                            };

                            let Some(c) = arg.as_string() else {
                                return Err(Error::invalid_arguments(
                                    "malformed Char constructor, expected String argument",
                                    expr.range(),
                                ));
                            };

                            if c.len() != 1 {
                                // #todo better error message.
                                return Err(Error::invalid_arguments(
                                    "the Char constructor requires a single-char string",
                                    expr.range(),
                                ));
                            }

                            let c = c.chars().next().unwrap();

                            Ok(Expr::Char(c).into())
                        }
                        "List" => {
                            let args = eval_args(tail, context)?;
                            Ok(Expr::List(args).into())
                        }
                        "Func" => {
                            let Some(params) = tail.first() else {
                                // #todo seems the range is not reported correctly here!!!
                                return Err(Error::invalid_arguments(
                                    "malformed func definition, missing function parameters",
                                    expr.range(),
                                ));
                            };

                            let body = &tail[1..];

                            let Some(params) = params.as_list() else {
                                return Err(Error::invalid_arguments(
                                    "malformed func parameters definition",
                                    params.range(),
                                ));
                            };

                            // #insight captures the static (lexical scope)

                            // #todo optimize!
                            Ok(Expr::Func(
                                params.clone(),
                                body.into(),
                                context.scope.clone(),
                            ))
                        }
                        // #todo macros should be handled at a separate, comptime, macroexpand pass.
                        // #todo actually two passes, macro_def, macro_expand
                        // #todo probably macro handling should be removed from eval, there are no runtime/dynamic macro definitions!!
                        "Macro" => {
                            let Some(params) = tail.first() else {
                                // #todo seems the range is not reported correctly here!!!
                                return Err(Error::invalid_arguments(
                                    "malformed macro definition, missing function parameters",
                                    expr.range(),
                                ));
                            };

                            let body = &tail[1..];

                            let Some(params) = params.as_list() else {
                                return Err(Error::invalid_arguments(
                                    "malformed macro parameters definition",
                                    params.range(),
                                ));
                            };

                            // #todo optimize!
                            Ok(Expr::Macro(params.clone(), body.into()))
                        }
                        _ => {
                            return Err(Error::not_invocable(
                                &format!("symbol `{head}`"),
                                head.range(),
                            ));
                        }
                    }
                }
                _ => {
                    return Err(Error::not_invocable(
                        &format!("expression `{head}`"),
                        head.range(),
                    ));
                }
            }
        }
        Expr::Array(items) => {
            // #insight [...] => (Array ...) => it's like a function.
            // #todo can this get pre-evaluated statically in some cases?
            let mut evaled_items = Vec::new();
            for item in items.borrow().iter() {
                evaled_items.push(eval(item, context)?);
            }
            Ok(Expr::array(evaled_items))
        }
        Expr::Dict(dict) => {
            // #insight evaluates the values.
            // #insight [...] => (Dict ...) => it's like a function.
            // #todo nasty code, improve.
            // #todo can this get pre-evaluated statically in some cases?
            let mut evaled_dict = HashMap::new();
            for (k, v) in dict.borrow().iter() {
                evaled_dict.insert(k.clone(), eval(v, context)?);
            }
            Ok(Expr::dict(evaled_dict))
        }
        _ => {
            // #todo hm, maybe need to report an error here? or even select the desired behavior? -> NO ERROR
            // #todo can we avoid the clone?
            // Unhandled expression variants evaluate to themselves.
            Ok(expr.clone())
        }
    }
}
