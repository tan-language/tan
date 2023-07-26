// #todo what is the proper place for this?
// #todo conflict with expr/expr_iter.rs
// #todo reuse Rust's iterator trait?
// #todo consider renaming `next` -> `resume` like coroutines

// #warning this is not used yet.

pub trait ExprIterator<T> {
    fn next(&mut self) -> Option<T>;
}

pub struct IntIterator {
    current: i64,
    end: i64,
    // step: i64, // #todo
}

impl ExprIterator<i64> for IntIterator {
    fn next(&mut self) -> Option<i64> {
        if self.current >= self.end {
            None
        } else {
            let value = self.current;
            self.current += 1;
            Some(value)
        }
    }
}

impl IntIterator {
    pub fn new(end: i64) -> Self {
        Self {
            current: 0,
            end,
            // step: 1,
        }
    }
}
