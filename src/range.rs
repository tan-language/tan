use std::fmt::{Debug, Display};

// #TODO add methods to find positions (line, col) from span index!

/// Represents a range in the source code.
pub type Range = std::ops::Range<usize>;

// #Insight Range is a valid name: https://www.wordhippo.com/what-is/the-adjective-for/range.html

// #Insight Ranged works better as a tuple, it simplifies the code at use site.

#[derive(Debug, Clone)]
pub struct Ranged<T>(pub T, pub Range);

impl<T> Display for Ranged<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// impl<T> Debug for Ranged<T>
// where
//     T: Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.value.fmt(f)
//     }
// }

impl<T> AsRef<T> for Ranged<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
