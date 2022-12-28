use std::fmt;

use crate::ann::Annotated;

// #TODO separate variant for list and apply/call (can this be defined statically?)
// #TODO List, MaybeList, Call

// #Insight
// Quote/`'` is like a an annotation, especially the #List annotation, but it is RECURSIVE!

// #TODO support Quote _and_ QuasiQuote.
// #TODO (recursive) quoting is not trivial.
// `(koko zonk lala)
// #quot (koko zonk lala)
// #quot koko
// (quot koko zonk lala)
// (List koko zonk lala)
// [koko zonk lala]

// #Insight
// AST = Expr = Value = Object

// #Insight
// The use of Vec in the Expr enum, keeps the nested expressions in the heap.

// #TODO consider parsing to 'simple' Expr, only List and Symbols
// #TODO optimize 'simple' Expr to 'execution' Expr

#[derive(Debug, Clone)]
/// A symbolic expression. This is the 'universal' data type in the language,
/// all values are expressions (and expressions are values). Evaluation is expression
/// rewriting to a fixed point.
pub enum Expr {
    One,        // Unit == List(Vec::new())
    Bool(bool), // #TODO remove?
    Int(i64),
    Float(f64),
    Symbol(String),
    String(String),
    // #TODO do should contain the expressions also, pre-parsed!
    Do,
    // #TODO let should contain the expressions also, pre-parsed!
    Let,
    // #TODO maybe this 'compound' if prohibits homoiconicity?
    If(
        Box<Annotated<Expr>>,
        Box<Annotated<Expr>>,
        Option<Box<Annotated<Expr>>>,
    ),
    List(Vec<Annotated<Expr>>),
    Func(Vec<Annotated<Expr>>, Box<Annotated<Expr>>), // #TODO is there a need to use Rc instead of Box?
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #TODO optimize this!
        f.write_str(
            (match self {
                Expr::One => "()".to_owned(),
                Expr::Bool(b) => b.to_string(),
                Expr::Int(n) => n.to_string(),
                Expr::Float(n) => n.to_string(),
                Expr::Symbol(s) => s.clone(),
                Expr::String(s) => s.clone(),
                Expr::Do => "do".to_owned(),
                Expr::Let => "let".to_owned(),
                // #TODO properly format if!
                Expr::If(..) => "if".to_owned(),
                Expr::List(terms) => {
                    format!(
                        "({})",
                        terms
                            .iter()
                            .map(|term| format!("{}", term.as_ref()))
                            .collect::<Vec<String>>()
                            .join(" ")
                    )
                }
                Expr::Func(..) => "#<func>".to_owned(),
            })
            .as_str(),
        )
    }
}
