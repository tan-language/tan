pub mod expr_iter;
pub mod expr_transform;

use std::{collections::HashMap, fmt, rc::Rc};

use crate::{ann::Ann, error::Error, eval::env::Env, range::Ranged};

// #TODO separate variant for list and apply/call (can this be defined statically?)
// #TODO List, MaybeList, Call
// #TODO Expr::Range()

// #Insight
// AST = Expr = Value = Object

// #Insight
// The use of Vec in the Expr enum, keeps the nested expressions in the heap.

// #Insight
// No need for a Zero/Never/Nothing Expr variant?

// #TODO consider parsing to 'simple' Expr, only List and Symbols
// #TODO optimize 'simple' Expr to 'execution' Expr
// #TODO introduce ForeignValue?
// #TODO ExprFn should get a single Expr? -> nah, it's foreign.

// #TODO not all Expr variants really need Ann, maybe the annotation should be internal to Expr?

// A function that accepts a list of Exprs and returns an Expr.
pub type ExprFn = dyn Fn(&[Ann<Expr>], &Env) -> Result<Ann<Expr>, Ranged<Error>>;

// #TODO use normal structs instead of tuple-structs?

#[derive(Clone)]
/// A symbolic expression. This is the 'universal' data type in the language,
/// all values are expressions (and expressions are values). Evaluation is expression
/// rewriting to a fixed point.
pub enum Expr {
    // --- Low-level ---
    One,                // Unit == List(Vec::new())
    Comment(String),    // #TODO consider renaming to Remark (REM)
    Annotation(String), // Only really useful for the formatter.
    Bool(bool),         // #TODO remove?
    Int(i64),
    Float(f64),
    Symbol(String),
    KeySymbol(String),
    Char(char),
    String(String),
    // #TODO better name for 'generic' List, how about `Cons` or `ConsList` or `Cell`?
    // #TODO add 'quoted' List -> Array!
    List(Vec<Ann<Expr>>),
    // #TODO should Array contain Ann<Expr>?
    Array(Vec<Expr>),
    // #TODO different name?
    // #TODO support Expr as keys?
    // #TODO should Dict contain Ann<Expr>?
    Dict(HashMap<String, Expr>),
    // Range(Box<Ann<Expr>>, Box<Ann<Expr>>, Option<Box<Ann<Expr>>>),
    Func(Vec<Ann<Expr>>, Box<Ann<Expr>>), // #TODO is there a need to use Rc instead of Box? YES! fast clones? INVESTIGATE!
    Macro(Vec<Ann<Expr>>, Box<Ann<Expr>>),
    ForeignFunc(Rc<ExprFn>), // #TODO for some reason, Box is not working here!
    // --- High-level ---
    // #TODO do should contain the expressions also, pre-parsed!
    Do,
    // #TODO let should contain the expressions also, pre-parsed!
    Let,
    // #TODO maybe this 'compound' if prohibits homoiconicity?
    If(Box<Ann<Expr>>, Box<Ann<Expr>>, Option<Box<Ann<Expr>>>),
}

// #TODO what is the Expr default? One (Unit/Any) or Zero (Noting/Never)

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Expr::One => "()".to_owned(),
            Expr::Comment(s) => format!("Comment({s})"),
            Expr::Annotation(s) => format!("Annotation({s})"),
            Expr::Bool(b) => format!("Bool({b})"),
            Expr::Symbol(s) => format!("Symbol({s})"),
            Expr::KeySymbol(s) => format!("KeySymbol({s})"),
            Expr::Char(c) => format!("Char({c})"),
            Expr::String(s) => format!("String(\"{s}\")"),
            Expr::Int(num) => format!("Int({num})"),
            Expr::Float(num) => format!("Float({num})"),
            Expr::Do => "do".to_owned(),
            Expr::List(terms) => {
                format!(
                    "List({})",
                    terms
                        .iter()
                        .map(|term| format!("{term:?}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Expr::Array(v) => format!("Array({v:?})"),
            Expr::Dict(d) => format!("Dict({d:?})"),
            Expr::Func(..) => "#<func>".to_owned(),
            Expr::Macro(..) => "#<macro>".to_owned(),
            Expr::ForeignFunc(..) => "#<foreign_func>".to_owned(),
            Expr::Let => "let".to_owned(),
            // #TODO properly format do, let, if, etc.
            Expr::If(_, _, _) => "if".to_owned(),
        };

        write!(f, "{text}")
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #TODO optimize this!
        f.write_str(
            (match self {
                Expr::One => "()".to_owned(),
                Expr::Comment(s) => format!(r#"(rem "{s}")"#), // #TODO what would be a good representation?
                Expr::Annotation(s) => format!("#{s}"),
                Expr::Bool(b) => b.to_string(),
                Expr::Int(n) => n.to_string(),
                Expr::Float(n) => n.to_string(),
                Expr::Symbol(s) => s.clone(),
                Expr::KeySymbol(s) => format!(":{s}"),
                Expr::Char(c) => format!(r#"(Char "{c}")"#), // #TODO no char literal?
                Expr::String(s) => format!("\"{s}\""),
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
                Expr::Array(exprs) => {
                    let exprs = exprs
                        .iter()
                        .map(|expr| expr.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("[{exprs}]")
                }
                Expr::Dict(dict) => {
                    // #TODO Dict should support arbitrary exprs (or at lease `(Into String)` exprs)
                    let exprs = dict
                        .iter()
                        .map(|(k, v)| format!("\"{k}\" {v}"))
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("{{{exprs}}}")
                }
                Expr::Func(..) => "#<func>".to_owned(),
                Expr::Macro(..) => "#<func>".to_owned(),
                Expr::ForeignFunc(..) => "#<foreign_func>".to_owned(),
            })
            .as_str(),
        )
    }
}

impl AsRef<Expr> for Expr {
    fn as_ref(&self) -> &Expr {
        self
    }
}

impl Expr {
    pub fn symbol(s: impl Into<String>) -> Self {
        Expr::Symbol(s.into())
    }

    pub fn string(s: impl Into<String>) -> Self {
        Expr::String(s.into())
    }
}

// #TODO think where this function is used. (it is used for Dict keys, hmm...)
// #TODO this is a confusing name!
/// Formats the expression as a value
pub fn format_value(expr: impl AsRef<Expr>) -> String {
    let expr = expr.as_ref();
    match expr {
        Expr::String(s) => s.to_string(),
        Expr::KeySymbol(s) => s.to_string(),
        _ => expr.to_string(),
    }
}

// #TODO use `.into()` to convert Expr to Annotated<Expr>.

#[cfg(test)]
mod tests {
    use crate::expr::Expr;

    #[test]
    fn expr_string_display() {
        let expr = Expr::string("hello");
        assert_eq!("\"hello\"", format!("{expr}"));
    }
}
