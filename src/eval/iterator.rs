// #todo what is the proper place for this?
// #todo conflict with expr/expr_iter.rs
// #todo reuse Rust's iterator trait?
// #todo consider renaming `next` -> `resume` like coroutines

use std::{cell::RefCell, rc::Rc};

use crate::expr::{expr_clone, Expr};

pub trait ExprIterator {
    fn next(&mut self) -> Option<Expr>;
}

// #todo hmm, not really needed, can reuse Rust's range/iterator/for?
// #todo somehow unify RangeITerators

pub struct IntRangeIterator {
    current: i64,
    pub start: i64,
    pub end: i64,
    pub step: i64,
}

impl ExprIterator for IntRangeIterator {
    fn next(&mut self) -> Option<Expr> {
        if self.current >= self.end {
            None
        } else {
            let value = self.current;
            self.current += self.step;
            Some(Expr::Int(value))
        }
    }
}

pub struct FloatRangeIterator {
    current: f64,
    pub start: f64,
    pub end: f64,
    pub step: f64,
}

impl ExprIterator for FloatRangeIterator {
    fn next(&mut self) -> Option<Expr> {
        if self.current >= self.end {
            None
        } else {
            let value = self.current;
            self.current += self.step;
            Some(Expr::Float(value))
        }
    }
}

// #todo what about reverse?
// #todo consolidate List/Array

// #insight this is used to iterate List.
pub struct ArrayIterator<'a> {
    current: usize,
    items: &'a [Expr],
    pub step: usize,
}

impl<'a> ExprIterator for ArrayIterator<'a> {
    fn next(&mut self) -> Option<Expr> {
        if self.current < self.items.len() {
            let value = self.items[self.current].clone(); // #todo argh, avoid this. should array have Rcs?
            self.current += self.step;
            Some(value)
        } else {
            None
        }
    }
}

// #insight this is used to iterate Array.
// #todo ArrayIterator2 should replace ArrayIterator.
pub struct ArrayIteratorRc<'a> {
    current: usize,
    items: std::cell::Ref<'a, Vec<Expr>>,
    pub step: usize,
}

impl<'a> ExprIterator for ArrayIteratorRc<'a> {
    fn next(&mut self) -> Option<Expr> {
        if self.current < self.items.len() {
            let value = self.items[self.current].clone(); // #todo argh, avoid this. should array have Rcs? SOS!!!
            self.current += self.step;
            Some(value)
        } else {
            None
        }
    }
}

// #todo under construction.
pub struct DictIterator {
    current: usize,
    items: Vec<Expr>,
    pub step: usize,
}

impl ExprIterator for DictIterator {
    // #todo keep rust iterator instead.
    fn next(&mut self) -> Option<Expr> {
        if self.current < self.items.len() {
            let value = self.items[self.current].clone(); // #todo argh, avoid this. should array have Rcs? SOS!!!
            self.current += self.step;
            Some(value)
        } else {
            None
        }
    }
}

// #todo find better name.
// #todo consider using Box<dyn ExprIterator> instead, at least have a custom helper that returns Box.
pub fn try_iterator_from<'a>(expr: &'a Expr) -> Option<Rc<RefCell<dyn ExprIterator + 'a>>> {
    match expr.unpack() {
        Expr::Int(n) => Some(Rc::new(RefCell::new(IntRangeIterator {
            current: 0,
            start: 0,
            end: *n,
            step: 1,
        }))),
        Expr::IntRange(start, end, step) => Some(Rc::new(RefCell::new(IntRangeIterator {
            current: 0,
            start: *start,
            end: *end,
            step: *step,
        }))),
        Expr::Float(n) => Some(Rc::new(RefCell::new(FloatRangeIterator {
            current: 0.0,
            start: 0.0,
            end: *n,
            step: 1.0,
        }))),
        Expr::FloatRange(start, end, step) => Some(Rc::new(RefCell::new(FloatRangeIterator {
            current: 0.0,
            start: *start,
            end: *end,
            step: *step,
        }))),
        // #todo consolidate handling of List and Array.
        Expr::List(items) => Some(Rc::new(RefCell::new(ArrayIterator {
            current: 0,
            items,
            step: 1,
        }))),
        Expr::Array(items) => {
            let items = items.borrow();
            Some(Rc::new(RefCell::new(ArrayIteratorRc {
                current: 0,
                items,
                step: 1,
            })))
        }
        Expr::Dict(_) => {
            // example usage:
            // (let user {:name "George" :age :gender :male})
            // (for [f user] (writeln "*** ${(f 0)} = ${(f 1)}"))
            // #todo somehow reuse dict_get_entries
            let Some(items) = expr.as_dict_mut() else {
                panic!("invalid state in for-dict");
            };

            // #todo why does map return k as String?
            // #todo wow, this is incredibly inefficient.
            // #todo #hack temp fix we add the a `:` prefix to generate keys
            let items: Vec<_> = items
                .iter()
                .map(|(k, v)| Expr::array(vec![Expr::KeySymbol(k.clone()), expr_clone(v)]))
                .collect();

            Some(Rc::new(RefCell::new(DictIterator {
                current: 0,
                items,
                step: 1,
            })))
        }
        _ => None,
    }
}
