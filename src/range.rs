use std::fmt;

// #TODO use Annotated instead of Ranged.

/// Represents a range in the source code.
pub type Range = std::ops::Range<usize>;

// #Insight Ranged is a valid name: https://www.wordhippo.com/what-is/the-adjective-for/ranged.html

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

/// A position within a text document.
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    // #TODO seems this conversion is needed too often, maybe should keep line,col info in range?
    // #TODO add unit test.
    pub fn from(index: usize, input: &str) -> Self {
        let chars = input.chars();

        let mut i: usize = 0;
        let mut line = 0;
        let mut line_start: usize = 0;

        for c in chars {
            i += 1;

            if c == '\n' {
                if i > index {
                    break;
                }

                line += 1;
                line_start = i;
            }
        }

        let col = index - line_start;

        Self { line, col }
    }
}
