use std::fmt;

use crate::ann::Annotated;

// #TODO Expr/Value is the _central_ datastructure, should be moved to the top-level.
// #TODO should be part of eval?

// #Insight
// AST = Expr = Value = Object

#[derive(Debug, Clone)]
pub enum Expr {
    One,        // Unit == List(Vec::new())
    Bool(bool), // #TODO remove?
    Int(i64),
    Float(f64),
    Symbol(String),
    String(String),
    Do(Vec<Annotated<Expr>>),
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
                Expr::Do(terms) => {
                    format!(
                        "(do {})",
                        terms
                            .iter()
                            .map(|term| format!("{}", term.as_ref()))
                            .collect::<Vec<String>>()
                            .join(" ")
                    )
                }
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
