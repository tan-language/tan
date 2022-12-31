use std::{fmt, rc::Rc};

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

#[derive(Clone)]
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
    If(Box<Ann<Expr>>, Box<Ann<Expr>>, Option<Box<Ann<Expr>>>),
    List(Vec<Ann<Expr>>),
    Func(Vec<Ann<Expr>>, Box<Ann<Expr>>), // #TODO is there a need to use Rc instead of Box? YES! fast clones? INVESTIGATE!
    ForeignFunc(Rc<dyn Fn(&[Expr], &Env) -> Result<Expr, EvalError>>), // #TODO for some reason, Box is not working here!
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Expr::One => "()".to_owned(),
            Expr::Bool(b) => format!("Bool({})", b),
            Expr::Symbol(s) => format!("Symbol({})", s),
            Expr::String(s) => format!("String({})", s),
            Expr::Int(num) => format!("Int({})", num),
            Expr::Float(num) => format!("Float({})", num),
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
