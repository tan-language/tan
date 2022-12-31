use std::{collections::HashMap, fmt, rc::Rc};

use crate::{
    ann::Ann,
    eval::{env::Env, error::EvalError},
};

// #TODO separate variant for list and apply/call (can this be defined statically?)
// #TODO List, MaybeList, Call

// #Insight
// AST = Expr = Value = Object

// #Insight
// The use of Vec in the Expr enum, keeps the nested expressions in the heap.

// #TODO consider parsing to 'simple' Expr, only List and Symbols
// #TODO optimize 'simple' Expr to 'execution' Expr
// #TODO introduce ForeignValue?
// #TODO ExprFn should get a single Expr? -> nah, it's foreign.

// A function that accepts a list of Exprs and returns an Expr.
pub type ExprFn = dyn Fn(&[Expr], &Env) -> Result<Expr, EvalError>;

#[derive(Clone)]
/// A symbolic expression. This is the 'universal' data type in the language,
/// all values are expressions (and expressions are values). Evaluation is expression
/// rewriting to a fixed point.
pub enum Expr {
    // --- Low-level ---
    One,        // Unit == List(Vec::new())
    Zero,       // Never, Nothing
    Bool(bool), // #TODO remove?
    Int(i64),
    Float(f64),
    Symbol(String),
    String(String),
    // #TODO better name for 'generic' List
    // #TODO add 'quoted' List
    List(Vec<Ann<Expr>>),
    // #TODO different name?
    // #TODO support Expr as keys?
    Dict(HashMap<String, Expr>),
    Func(Vec<Ann<Expr>>, Box<Ann<Expr>>), // #TODO is there a need to use Rc instead of Box? YES! fast clones? INVESTIGATE!
    ForeignFunc(Rc<ExprFn>),              // #TODO for some reason, Box is not working here!
    // --- High-level ---
    // #TODO do should contain the expressions also, pre-parsed!
    Do,
    // #TODO let should contain the expressions also, pre-parsed!
    Let,
    // #TODO maybe this 'compound' if prohibits homoiconicity?
    If(Box<Ann<Expr>>, Box<Ann<Expr>>, Option<Box<Ann<Expr>>>),
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Expr::One => "()".to_owned(),
            Expr::Zero => "!".to_owned(), // #TODO a different symbol, maybe Unicode!
            Expr::Bool(b) => format!("Bool({b})"),
            Expr::Symbol(s) => format!("Symbol({s})"),
            Expr::String(s) => format!("String({s})"),
            Expr::Int(num) => format!("Int({num})"),
            Expr::Float(num) => format!("Float({num})"),
            Expr::Do => "do".to_owned(),
            Expr::List(terms) => {
                format!(
                    "List({})",
                    terms
                        .iter()
                        .map(|term| format!("{:?}", term))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Expr::Dict(hm) => format!("Dict({hm:?})"),
            Expr::Func(..) => "#<fn>".to_owned(),
            Expr::ForeignFunc(..) => "#<foreign_func>".to_owned(),
            Expr::Let => "let".to_owned(),
            // #TODO properly format do, let, if, etc.
            Expr::If(_, _, _) => "if".to_owned(),
        };

        write!(f, "{}", text)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #TODO optimize this!
        f.write_str(
            (match self {
                Expr::One => "()".to_owned(),
                Expr::Zero => "!".to_owned(), // #TODO a different symbol, maybe Unicode!
                Expr::Bool(b) => b.to_string(),
                Expr::Int(n) => n.to_string(),
                Expr::Float(n) => n.to_string(),
                Expr::Symbol(s) => s.clone(),
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
                Expr::Dict(hm) => format!("{hm:?}"),
                Expr::Func(..) => "#<func>".to_owned(),
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

/// Formats the expression as a value.
pub fn format_value(expr: impl AsRef<Expr>) -> String {
    let expr = expr.as_ref();
    match expr {
        Expr::String(s) => s.to_string(),
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
