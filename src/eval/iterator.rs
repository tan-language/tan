// #todo what is the proper place for this?
// #todo conflict with expr/expr_iter.rs
// #todo reuse Rust's iterator trait?
// #todo consider renaming `next` -> `resume` like coroutines

// #todo what about negative iteration, negative step?

use std::{cell::RefCell, rc::Rc, sync::RwLockReadGuard};

use crate::{
    expr::{expr_clone, Expr},
    util::expect_lock_read,
};

pub trait ExprIterator {
    fn next(&mut self) -> Option<Expr>;
}

// #todo hmm, not really needed, can reuse Rust's range/iterator/for?
// #todo somehow unify RangeITerators

pub struct IntRangeIterator {
    current: i64,
    pub end: i64,
    pub step: i64,
}

impl IntRangeIterator {
    // #todo find a better name.
    #[inline]
    fn is_exhausted(&self) -> bool {
        if self.step > 0 {
            self.current >= self.end
        } else {
            self.current <= self.end
        }
    }
}

impl ExprIterator for IntRangeIterator {
    fn next(&mut self) -> Option<Expr> {
        if self.is_exhausted() {
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
    pub end: f64,
    pub step: f64,
}

impl FloatRangeIterator {
    // #todo find a better name.
    #[inline]
    fn is_exhausted(&self) -> bool {
        if self.step > 0.0 {
            self.current >= self.end
        } else {
            self.current <= self.end
        }
    }
}

impl ExprIterator for FloatRangeIterator {
    fn next(&mut self) -> Option<Expr> {
        if self.is_exhausted() {
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
    items: RwLockReadGuard<'a, Vec<Expr>>,
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

pub struct MapIterator {
    current: usize,
    items: Vec<Expr>,
    pub step: usize,
}

impl ExprIterator for MapIterator {
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

// #todo move iterator impls to the corresponding type impl?

pub struct SetIterator {
    current: usize,
    items: Vec<Expr>,
    pub step: usize,
}

impl ExprIterator for SetIterator {
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
            end: *n,
            step: 1,
        }))),
        Expr::IntRange(start, end, step) => Some(Rc::new(RefCell::new(IntRangeIterator {
            // #todo start is not really needed, could use just current!
            current: *start,
            end: *end,
            step: *step,
        }))),
        Expr::Float(n) => Some(Rc::new(RefCell::new(FloatRangeIterator {
            current: 0.0,
            end: *n,
            step: 1.0,
        }))),
        Expr::FloatRange(start, end, step) => Some(Rc::new(RefCell::new(FloatRangeIterator {
            // #todo start is really not needed!
            current: *start,
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
            // let Ok(items) = items.read() else {
            //     // #todo maybe panic here?
            //     return None;
            // };
            let items = expect_lock_read(items);
            Some(Rc::new(RefCell::new(ArrayIteratorRc {
                current: 0,
                items,
                step: 1,
            })))
        }
        Expr::Map(_) => {
            // example usage:
            // (let user {:name "George" :age :gender :male})
            // (for [f user] (writeln "*** ${(f 0)} = ${(f 1)}"))
            // #todo somehow reuse map_get_entries
            let Some(items) = expr.as_map_mut() else {
                panic!("invalid state in for-map");
            };

            // #todo why does map return k as String?
            // #todo wow, this is incredibly inefficient.
            // #todo #hack temp fix we add the a `:` prefix to generate keys
            let items: Vec<_> = items
                .iter()
                .map(|(k, v)| Expr::array(vec![Expr::KeySymbol(k.clone()), expr_clone(v)]))
                .collect();

            Some(Rc::new(RefCell::new(MapIterator {
                current: 0,
                items,
                step: 1,
            })))
        }
        Expr::Set(_) => {
            // example usage: #todo
            // #todo somehow reuse map_get_entries
            let Some(items) = expr.as_set_mut() else {
                panic!("invalid state in for-set");
            };

            // #todo wow, this is incredibly inefficient.
            // #todo #hack temp fix we add the a `:` prefix to generate keys
            // #todo argh, cloned!
            let items: Vec<_> = items.iter().cloned().collect();

            Some(Rc::new(RefCell::new(SetIterator {
                current: 0,
                items,
                step: 1,
            })))
        }
        _ => None,
    }
}
