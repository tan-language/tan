use std::fmt;

// #TODO add methods to find positions (line, col) from span index!
// #TODO use Annotated instead of Ranged.

/// Represents a range in the source code.
pub type Range = std::ops::Range<usize>;

// #Insight Range is a valid name: https://www.wordhippo.com/what-is/the-adjective-for/range.html

// #Insight Ranged works better as a tuple, it simplifies the code at use site.

#[derive(Debug, Clone)]
pub struct Ranged<T>(pub T, pub Range);

// #TODO is this good? it hides the wrapped data.
impl<T> fmt::Display for Ranged<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Ranged<T> {
    pub fn new(value: T) -> Self {
        // #TODO ultra-hack
        Ranged(value, 0..0)
    }
}

impl<T> AsRef<T> for Ranged<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
