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

impl Position {
    // #warning
    // These helpers are NOT used yet! Need to decide how to handle index.
    pub fn move_to_next_col(&mut self) {
        self.index += 1;
        self.col += 1;
    }

    pub fn move_to_prev_col(&mut self) {
        self.index -= 1;
        self.col -= 1;
    }

    pub fn move_to_next_line(&mut self) {
        // #todo what about index?
        self.line += 1;
        self.col = 0;
    }
}
