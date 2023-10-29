// #todo what is the proper place for this?
// #todo conflict with expr/expr_iter.rs
// #todo reuse Rust's iterator trait?
// #todo consider renaming `next` -> `resume` like coroutines

use std::{cell::RefCell, rc::Rc};

use crate::expr::Expr;

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

pub struct ArrayIterator2<'a> {
    current: usize,
    items: std::cell::Ref<'a, Vec<Expr>>,
    pub step: usize,
}

impl<'a> ExprIterator for ArrayIterator2<'a> {
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
            items: &items,
            step: 1,
        }))),
        Expr::Array(items) => {
            let items = items.borrow();
            Some(Rc::new(RefCell::new(ArrayIterator2 {
                current: 0,
                items: items,
                step: 1,
            })))
        }
        _ => None,
    }
}
