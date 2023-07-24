// #todo use Position{index, line, col}, Range<Position>, or, Span{index, line, col, len}?
// #todo make sure expressions are ranged correctly.

// #todo it's better to keep 'position' (line, col) like LSP, for easier reconstruction
// of line,col for error messages and/or LSP, and to allow to index the buffer by line
// (if we keep the buffer as array of lines)

// #todo use Annotated instead of Ranged.
// #todo add support for Set operations for ranges.
// #todo if/when we convert this to a range of positions, we should consider renaming to Span.

/// Represents a range in the source code.
pub type Range = std::ops::Range<Position>;

/// A position within a text document.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub index: usize,
    pub line: usize,
    pub col: usize,
}

// impl Position {
//     // #todo seems this conversion is needed too often, maybe should keep line,col info in range?
//     // #todo add unit test.
//     pub fn from_index(index: usize, input: &str) -> Self {
//         let chars = input.chars();

//         let mut i: usize = 0;
//         let mut line = 0;
//         let mut line_start: usize = 0;

//         for c in chars {
//             i += 1;

//             if c == '\n' {
//                 if i > index {
//                     break;
//                 }

//                 line += 1;
//                 line_start = i;
//             }
//         }

//         let col = index - line_start;

//         Self { line, col }
//     }

//     pub fn from_range(range: &Range, input: &str) -> Self {
//         Self::from_index(range.start, input)
//     }
// }
