pub struct PutBackIterator<'a, T> {
    items: std::slice::Iter<'a, T>,
    buffer: Vec<&'a T>,
}

impl<'a, T> Iterator for PutBackIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // First try to exhaust the items that where 'peeked' and the 'put back'
        // to the buffer.
        if let Some(item) = self.buffer.pop() {
            return Some(item);
        }

        if let Some(item) = self.items.next() {
            Some(item)
        } else {
            None
        }
    }
}

// #todo explain what this does.
impl<'a, T> PutBackIterator<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        Self {
            items: items.iter(),
            buffer: Vec::new(),
        }
    }

    // #todo find a better name.
    pub fn put_back(&mut self, item: &'a T) {
        self.buffer.push(item);
    }
}

// #todo add unit tests.
