// #TODO this is a temp solution.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommentKind {
    Line,
    Inline,
    Multiline,
}
