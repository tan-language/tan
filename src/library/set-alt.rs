// #insight
// Alternative implementation of Set, for the future. Does not use a 'dedicated'
// Expr for Set.

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    collections::HashSet,
    rc::Rc,
    sync::Arc,
};

use crate::{
    context::Context,
    error::Error,
    eval::iterator::ArrayIterator,
    expr::{annotate_type, Expr},
    util::module_util::require_module,
};

// #warning under construction, not ready for use yet!

pub fn set_new(_args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    // #todo for the moment only strings are supported, we need to support Expr.
    let set: HashSet<String> = HashSet::new();
    let expr = Expr::ForeignStruct(Rc::new(RefCell::new(set)) as Rc<RefCell<dyn Any>>);

    Ok(annotate_type(expr, "Set"))
}

pub fn set_put(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this, value] = args else {
        return Err(Error::invalid_arguments(
            "requires `this` and `value` arguments",
            None,
        ));
    };

    let Expr::ForeignStruct(s) = this.unpack() else {
        return Err(Error::invalid_arguments("`this` should be a Set", None));
    };

    let Some(value) = value.as_stringable() else {
        return Err(Error::invalid_arguments(
            "`value` argument should be a Stringable",
            value.range(),
        ));
    };

    if let Some(s) = s.borrow_mut().downcast_mut::<HashSet<String>>() {
        s.insert(value.to_string());
    } else {
        return Err(Error::invalid_arguments("invalid Set", None));
    }

    // #todo what should we return here?
    Ok(Expr::One)
}

// #todo consider other names, e.g. items?
pub fn set_values(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    // #todo convert this to a macro?

    let Expr::ForeignStruct(s) = this.unpack() else {
        return Err(Error::invalid_arguments("`this` should be a Set", None));
    };

    let mut s: RefMut<dyn Any> = s.borrow_mut();

    let Some(s) = s.downcast_mut::<HashSet<String>>() else {
        return Err(Error::invalid_arguments("invalid Set", None));
    };

    let mut values = Vec::new();

    for value in s.iter() {
        values.push(Expr::string(value));
    }

    Ok(Expr::array(values))
}

// pub fn set_iterator(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
//     let [this] = args else {
//         return Err(Error::invalid_arguments("requires `this` argument", None));
//     };

//     // #todo convert this to a macro?

//     let Expr::ForeignStruct(s) = this.unpack() else {
//         return Err(Error::invalid_arguments("`this` should be a Set", None));
//     };

//     let mut s: Ref<dyn Any> = s.borrow();

//     let Some(items) = s.downcast_ref::<HashSet<String>>() else {
//         return Err(Error::invalid_arguments("invalid Set", None));
//     };

//     // #todo why does map return k as String?
//     // #todo wow, this is incredibly inefficient.
//     // #todo #hack temp fix we add the a `:` prefix to generate keys
//     let items: Vec<_> = items.iter().map(|v| Expr::string(v)).collect();

//     Some(Rc::new(RefCell::new(ArrayIterator {
//         current: 0,
//         items,
//         step: 1,
//     })))
// }

pub fn setup_lib_set(context: &mut Context) {
    // #tod op consider other paths, e.g. data/set, collections/set, collection/set, etc?
    let module = require_module("set", context);

    module.insert_invocable("Set", Expr::ForeignFunc(Arc::new(set_new)));

    // #todo #hack temp fix!
    // #todo really need to improve signature matching and e.g. support put$$Set$$Expr or put$$Set$$Any
    module.insert_invocable("put$$Set$$Int", Expr::ForeignFunc(Arc::new(set_put)));
    module.insert_invocable("put$$Set$$Float", Expr::ForeignFunc(Arc::new(set_put)));
    module.insert_invocable("put$$Set$$String", Expr::ForeignFunc(Arc::new(set_put)));
    // #todo investigate why this is needed!
    // #todo better solution: use Expr::Method or Expr::Multi for foreign functions and functions.
    module.insert_invocable("values-of", Expr::ForeignFunc(Arc::new(set_values)));
    module.insert_invocable("values-of$$Set", Expr::ForeignFunc(Arc::new(set_values)));
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use crate::{api::eval_string, context::Context, expr::Expr};

    #[test]
    fn set_push_usage() {
        let mut context = Context::new();
        let expr = eval_string(
            r#"
            (use [Set put values-of] set)
            (let s (Set))
            (put s "hello")
            (put s "hello")
            (put s "hello")
            (put s "world")
            (put s "world")
            (values-of s)
        "#,
            &mut context,
        )
        .unwrap();
        let values = expr.as_array().unwrap();
        let values = values.borrow();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&Expr::string("hello")));
        assert!(values.contains(&Expr::string("world")));
    }
}
