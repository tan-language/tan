// #TODO implement the Iterator interface!

pub struct LookaheadIterator<'a, T> {
    items: std::slice::Iter<'a, T>,
    lookahead: Vec<&'a T>, // #TODO find a better name.
}

impl<'a, T> Iterator for LookaheadIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // First try to exhaust the lookahead buffer, containing items that
        // where 'peeked' and the 'put back' to the buffer.
        if let Some(item) = self.lookahead.pop() {
            return Some(item);
        }

        if let Some(item) = self.items.next() {
            Some(item)
        } else {
            None
        }
    }
}

// #TODO explain what this does.
impl<'a, T> LookaheadIterator<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        Self {
            items: items.iter(),
            lookahead: Vec::new(),
        }
    }

    // #TODO find a better name.
    pub fn put_back(&mut self, item: &'a T) {
        self.lookahead.push(item);
    }
}

// #TODO add unit tests.
