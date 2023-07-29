// #todo what is the proper place for this?
// #todo conflict with expr/expr_iter.rs
// #todo reuse Rust's iterator trait?
// #todo consider renaming `next` -> `resume` like coroutines

// #warning this is not used yet.

use crate::expr::Expr;

pub trait ExprIterator {
    fn next(&mut self) -> Option<Expr>;
}

// #todo hmm, not really needed, can reuse Rust's range/iterator/for.

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

impl IntRangeIterator {
    pub fn new(start: i64, end: i64, step: i64) -> Self {
        Self {
            current: 0,
            start,
            end,
            step,
        }
    }

    pub fn from_int(end: i64) -> Self {
        Self {
            current: 0,
            start: 0,
            end,
            step: 1,
        }
    }

    pub fn try_from(expr: &Expr) -> Option<Self> {
        match expr.unpack() {
            Expr::Int(n) => Some(IntRangeIterator::from_int(*n)),
            Expr::IntRange(start, end, step) => Some(IntRangeIterator::new(*start, *end, *step)),
            _ => None,
        }
    }
}
