use std::fmt::{Debug, Display};

// #TODO keep the name `Range` instead of `Span`?
// #TODO add methods to find positions (line, col) from span index!

/// Represents a span in the source code.
pub type Span = std::ops::Range<usize>;

// #TODO weird name, rename to `Ranged`? or `WithRange`?
// #TODO https://www.wordhippo.com/what-is/the-adjective-for/range.html

// #Insight Spanned works better as a tuple, it simplifies the code at use site.
#[derive(Debug, Clone)]
pub struct Spanned<T>(pub T, pub Span);

impl<T> Display for Spanned<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// impl<T> Debug for Spanned<T>
// where
//     T: Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.value.fmt(f)
//     }
// }

impl<T> AsRef<T> for Spanned<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
