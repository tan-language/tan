pub struct LookaheadIterator<T, I>
where
    I: IntoIterator<Item = T>,
{
    items: I::IntoIter,
    lookahead: Vec<T>,
}

impl<T, I> LookaheadIterator<T, I>
where
    I: IntoIterator<Item = T>,
{
    pub fn new(items: I::IntoIter) -> Self {
        let items = items.into_iter();
        Self {
            items: items.into_iter(),
            lookahead: Vec::new(),
        }
    }

    // #TODO unit test
    // #TODO refactor
    pub fn next(&mut self) -> Option<T> {
        if let Some(item) = self.lookahead.pop() {
            return Some(item);
        }

        if let Some(token) = self.items.next() {
            Some(token)
        } else {
            None
        }
    }

    pub fn put_back(&mut self, item: T) {
        self.lookahead.push(item);
    }
}
