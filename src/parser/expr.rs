// #Insight
// AST = Expr = Value = Object

// #TODO Introduce annotated value

#[derive(Debug)]
pub enum Expr {
    One,        // Unit == List(Vec::new())
    Bool(bool), // #TODO remove?
    Int(i64),
    Symbol(String),
    String(String),
    List(Vec<Expr>),
    Func(Vec<Expr>, Box<Expr>), // #TODO is there a need to use Rc instead of Box?
}
