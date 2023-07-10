use core::fmt;
use std::collections::HashMap;

use crate::{
    expr::{format_value, Expr},
    range::{Position, Range},
};

// #insight
// We keep this file for the comments.

// #TODO somehow annotations should trigger macros.
// #TODO initially keep the range as a list

// #TODO
// - Uppercase -> (:type Uppercase)
// - lowercase -> (:lowercase true)
// - (:key value)

// #TODO consider {+/-}lowercase -> true/false

// #TODO consider `Ann`, `Ax`, `An`, `Av`
// #TODO define 'special' annotations, e.g. `type`, `range`, `method`, etc.
// #TODO maybe use ALLCAP for special annotations? e.g. TYPE, RANGE, METHOD.

// #Insight
// The Annotated struct will be used a lot, it makes sense to use
// Option to avoid unnecessary HashMap allocations.

// #Insight
// Annotations are 'culled' in the parser, so we can use them for 'shebang'.

// #TODO consider keeping annotations as Vec (to maintain order, and also, not many annotations, typically fast scanning)
// #TODO keep range separate?
// #TODO actually, we don't need insertion order but alphabetical order, a BTreeMap can work

// #[derive(Clone)]
// pub struct ANNO<T>(pub T, pub Option<HashMap<String, Expr>>);

// impl<T> ANNO<T> {
//     pub fn with_type(value: T, type_expr: Expr) -> Self {
//         let mut map = HashMap::new();
//         map.insert("type".to_owned(), type_expr);
//         Self(value, Some(map))
//     }

//     pub fn with_range(value: T, range: Range) -> Self {
//         let mut map = HashMap::new();
//         map.insert("range".to_owned(), range_to_expr(&range));
//         Self(value, Some(map))
//     }
// }

// impl<T> ANNO<T> {
//     pub fn set_annotation(&mut self, name: impl Into<String>, expr: Expr) {
//         self.1
//             .get_or_insert(HashMap::new())
//             .insert(name.into(), expr);
//     }

//     pub fn get_annotation(&self, name: impl Into<String>) -> Option<&Expr> {
//         let Some(ref ann ) = self.1 else {
//             return None;
//         };

//         ann.get(&name.into())
//     }

//     pub fn contains_annotation(&self, name: impl Into<String>) -> bool {
//         let Some(ref ann ) = self.1 else {
//             return false;
//         };

//         ann.contains_key(&name.into())
//     }

//     pub fn set_type(&mut self, type_expr: Expr) {
//         self.set_annotation("type", type_expr);
//     }

//     // #TODO consider specialization for `Ann<Expr>` that also checks the target?
//     // #TODO optimize get_type for literals, and even skip adding as annotation?
//     // #TODO introduce `Unknown` type? or just use `One`?
//     pub fn get_type(&self) -> &Expr {
//         self.get_annotation("type").unwrap_or(&Expr::One)
//     }

//     pub fn to_type_string(&self) -> String {
//         format_value(self.get_type())
//     }

//     /// Sets the range of the annotated expression.
//     pub fn set_range(&mut self, range: &Range) {
//         self.set_annotation("range", range_to_expr(range))
//     }

//     /// Returns the range of the annotated expression. Forces a default range
//     /// if the annotation is missing.
//     pub fn get_range(&self) -> Option<Range> {
//         self.get_annotation("range").map(expr_to_range)
//     }

//     // #TODO get_method (multiple-dispatch)
// }

// impl<T> fmt::Debug for ANNO<T>
// where
//     T: fmt::Debug,
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         if self.1.is_some() {
//             let annotations = self
//                 .1
//                 .clone()
//                 .unwrap()
//                 .iter()
//                 .map(|(k, v)| format!("{k}={v}"))
//                 .collect::<Vec<_>>()
//                 .join(",");
//             write!(f, "{:?}@[{annotations}]", self.0)
//         } else {
//             write!(f, "{:?}", self.0)
//         }
//     }
// }

// impl<T> fmt::Display for ANNO<T>
// where
//     T: fmt::Display,
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }

// impl<T> ANNO<T> {
//     pub fn new(value: T) -> Self {
//         Self(value, None)
//     }
// }

// impl<T> AsRef<T> for ANNO<T> {
//     fn as_ref(&self) -> &T {
//         &self.0
//     }
// }

// impl<T> AsRef<T> for Box<ANNO<T>> {
//     fn as_ref(&self) -> &T {
//         &self.0
//     }
// }

// impl<T> From<T> for ANNO<T> {
//     fn from(value: T) -> Self {
//         ANNO::new(value)
//     }
// }

// // #TODO implement Defer into Expr!

// // #TODO convert to the Expr::Range variant.
// // #TODO convert position to Dict Expr.

// pub fn position_to_expr(position: &Position) -> Expr {
//     let mut map: HashMap<String, Expr> = HashMap::new();
//     map.insert("index".to_owned(), Expr::Int(position.index as i64));
//     map.insert("line".to_owned(), Expr::Int(position.line as i64));
//     map.insert("col".to_owned(), Expr::Int(position.line as i64));
//     Expr::Dict(map)
// }

// pub fn expr_to_position(expr: &Expr) -> Position {
//     if let Expr::Dict(dict) = expr {
//         let Some(Expr::Int(index)) = dict.get("index") else {
//             // #TODO fix me!
//             return Position::default();
//         };

//         let Some(Expr::Int(line)) = dict.get("line") else {
//             // #TODO fix me!
//             return Position::default();
//         };

//         let Some(Expr::Int(col)) = dict.get("col") else {
//             // #TODO fix me!
//             return Position::default();
//         };

//         return Position {
//             index: *index as usize,
//             line: *line as usize,
//             col: *col as usize,
//         };
//     }

//     // #TODO fix me!
//     return Position::default();
// }

// pub fn range_to_expr(range: &Range) -> Expr {
//     let start = position_to_expr(&range.start);
//     let end = position_to_expr(&range.end);

//     Expr::Array(vec![start, end])
// }

// // #TODO nasty code.
// pub fn expr_to_range(expr: &Expr) -> Range {
//     // #TODO error checking?
//     let Expr::Array(terms) = expr else {
//         // #TODO hmm...
//         return Range::default();
//     };

//     Range {
//         start: expr_to_position(&terms[0]),
//         end: expr_to_position(&terms[1]),
//     }
// }
