// #Insight
// AST = Expr = Value = Object

// #TODO Introduce annotated value

use crate::ann::Annotated;

#[derive(Debug)]
pub enum Expr {
    One,        // Unit == List(Vec::new())
    Bool(bool), // #TODO remove?
    Int(i64),
    Float(f64),
    Symbol(String),
    String(String),
    List(Vec<Annotated<Expr>>),
    Func(Vec<Annotated<Expr>>, Box<Annotated<Expr>>), // #TODO is there a need to use Rc instead of Box?
}
