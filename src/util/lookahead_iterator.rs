pub struct LookaheadIterator<'a, T> {
    items: std::slice::Iter<'a, T>,
    lookahead: Vec<&'a T>, // #TODO find a better name.
}

// #TODO explain what this does.
impl<'a, T> LookaheadIterator<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        Self {
            items: items.iter(),
            lookahead: Vec::new(),
        }
    }

    // #TODO unit test
    // #TODO refactor
    pub fn next(&mut self) -> Option<&'a T> {
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

    // #TODO find a better name.
    pub fn put_back(&mut self, item: &'a T) {
        self.lookahead.push(item);
    }
}
