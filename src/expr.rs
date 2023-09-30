pub mod expr_iter;
pub mod expr_transform;

use std::{collections::HashMap, fmt, rc::Rc, sync::Arc};

use rust_decimal::Decimal;

use crate::{
    context::Context,
    error::Error,
    lexer::comment::CommentKind,
    module::Module,
    range::{Position, Range},
    scope::Scope,
    util::fmt::format_float,
};

// #todo introduce Expr::Ref() with an Rc reference to avoid excessive cloning!

// #insight
// Annotations are only for named bindings, static-time pseudo-annotations for
// literals are resolved before dynamic-time, Expr::Ann() is useful for that! But,
// we can probably skip most expr.unpack()s.

// #insight use structs for enum values, is more ..structured, readable and can have methods.

// #todo separate variant for list and apply/call (can this be defined statically?)
// #todo List, MaybeList, Call
// #todo Expr::Range()

// #Insight
// AST = Expr = Value = Object

// #Insight
// The use of Vec in the Expr enum, keeps the nested expressions in the heap.

// #Insight
// No need for a Zero/Never/Nothing Expr variant?

// #todo (do ...) blocks should also have lexical scope.

// #todo what would be the 'default'?
// #todo consider parsing to 'simple' Expr, only List and Symbols
// #todo optimize 'simple' Expr to 'execution' Expr
// #todo introduce ForeignValue?
// #todo ExprFn should get a single Expr? -> nah, it's foreign.

// #todo not all Expr variants really need Ann, maybe the annotation should be internal to Expr?

// #todo consider Visitor pattern instead of enum?

// #todo consider &mut Context <--
// #todo consider &mut and & Context, different types!
// #todo find a better name for the type-alias
// #insight the `+ Send + Sync + 'static` suffix allows Expr to be Sync.
/// A function that accepts a list of Exprs and returns an Expr.
pub type ExprFn = dyn Fn(&[Expr], &Context) -> Result<Expr, Error> + Send + Sync + 'static;

// #todo use normal structs instead of tuple-structs?

#[derive(Clone)]
/// A symbolic expression. This is the 'universal' data type in the language,
/// all values are expressions (and expressions are values). Evaluation is expression
/// rewriting to a fixed point.
pub enum Expr {
    // --- Low-level ---
    One,                          // Unit == List(Vec::new())
    Comment(String, CommentKind), // #todo consider renaming to Remark (REM)
    TextSeparator,                // for the formatter.
    Bool(bool),                   // #todo remove?
    Int(i64),
    Float(f64),
    Dec(Decimal),
    Symbol(String),
    KeySymbol(String),
    Char(char),
    String(String),
    // #todo better name for 'generic' List, how about `Cons` or `ConsList` or `Cell`?
    // #todo add 'quoted' List -> Array!
    // #todo do we really need Vec here? Maybe Arc<[Expr]> is enough?
    List(Vec<Expr>),
    Array(Vec<Expr>),
    // #todo different name?
    // #todo support Expr as keys?
    Dict(HashMap<String, Expr>),
    IntRange(i64, i64, i64),   // start, end, step #todo use a struct here,
    FloatRange(f64, f64, f64), // start, end, step #todo use a struct here,
    // Range(...),
    // #todo, the Func should probably store the Module environment.
    Func(Vec<Expr>, Vec<Expr>, Rc<Scope>), // #todo maybe should have explicit do block?
    Macro(Vec<Expr>, Vec<Expr>),           // #todo maybe should have explicit do block?
    // #todo, the ForeignFunc should probably store the Module environment.
    ForeignFunc(Arc<ExprFn>), // #todo for some reason, Box is not working here!
    // --- High-level ---
    // #todo do should contain the expressions also, pre-parsed!
    Do,
    // #todo let should contain the expressions also, pre-parsed!
    Let,
    // #todo maybe this 'compound' if prohibits homoiconicity?
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Annotated(Box<Expr>, HashMap<String, Expr>),
    // #todo maybe use annotation in Expr for public/exported? no Vec<String> for exported?
    // #todo convert free-expression into pseudo-function?
    // Module(HashMap<String, Expr>, Vec<String>, Vec<Expr>), // bindings, public/exported, free-expressions.
    Module(Rc<Module>),
}

// #todo what is the Expr default? One (Unit/Any) or Zero (Noting/Never)
// #todo use Sexp notation here, duh.
impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Expr::One => "()".to_owned(),
            Expr::Comment(s, _) => format!("Comment({s})"),
            Expr::TextSeparator => "<TEXT-SEPARATOR>".to_owned(),
            Expr::Bool(b) => format!("Bool({b})"),
            Expr::Symbol(s) => format!("Symbol({s})"),
            Expr::KeySymbol(s) => format!("KeySymbol({s})"),
            Expr::Char(c) => format!("Char({c})"),
            Expr::String(s) => format!("String(\"{s}\")"),
            Expr::Int(num) => format!("Int({num})"),
            Expr::Float(num) => format!("Float({num})"),
            Expr::Dec(num) => format!("(Dec {num})"),
            Expr::Do => "do".to_owned(),
            Expr::List(terms) => {
                format!(
                    "List(\n{})",
                    terms
                        .iter()
                        .map(|term| format!("{term:?}"))
                        .collect::<Vec<String>>()
                        .join(",\n")
                )
            }
            Expr::Array(v) => format!("Array({v:?})"),
            Expr::Dict(d) => format!("Dict({d:?})"),
            Expr::IntRange(start, end, step) => format!("(Range Int {start} {end} {step})"),
            Expr::FloatRange(start, end, step) => format!("(Range Int {start} {end} {step})"),
            Expr::Func(..) => "#<func>".to_owned(),
            Expr::Macro(..) => "#<macro>".to_owned(),
            Expr::ForeignFunc(..) => "#<foreign_func>".to_owned(),
            Expr::Let => "let".to_owned(),
            // #todo properly format do, let, if, etc.
            Expr::If(_, _, _) => "if".to_owned(),
            // #insight intentionally pass through the formatting.
            // Expr::Annotated(expr, ann) => format!("ANN({expr:?}, {ann:?})"),
            Expr::Annotated(expr, _ann) => format!("Ann({expr:?})"), // #skip annotations.
            Expr::Module(module) => format!("Module({})", module.stem),
        };

        write!(f, "{text}")
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #todo optimize this!
        f.write_str(
            (match self {
                Expr::One => "()".to_owned(),
                Expr::Comment(s, _) => format!(r#"(rem "{s}")"#), // #todo what would be a good representation?
                Expr::TextSeparator => "<TS>".to_owned(),
                Expr::Bool(b) => b.to_string(),
                Expr::Int(n) => n.to_string(),
                Expr::Float(n) => n.to_string(),
                Expr::Dec(n) => format!("(Dec {n})"), // #todo 'literal', e.f. 1.23d or #Dec 1.23
                Expr::Symbol(s) => s.clone(),
                Expr::KeySymbol(s) => format!(":{s}"),
                Expr::Char(c) => format!(r#"(Char "{c}")"#), // #todo no char literal?
                Expr::String(s) => format!("\"{s}\""),
                Expr::Do => "do".to_owned(),
                Expr::Let => "let".to_owned(),
                // #todo properly format if!
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
                    // #todo Dict should support arbitrary exprs (or at lease `(Into String)` exprs)
                    // #todo currently we convert keys to symbol, make this more subtle.
                    let exprs = dict
                        .iter()
                        .map(|(k, v)| format!(":{k} {v}"))
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("{{{exprs}}}")
                }
                Expr::IntRange(start, end, step) => format!("{start}..{end}/{step}"),
                Expr::FloatRange(start, end, step) => format!("{start}..{end}/{step}"), // #todo consider using `:` or `,` instead of `/`?
                Expr::Func(..) => "#<func>".to_owned(),
                Expr::Macro(..) => "#<func>".to_owned(),
                Expr::ForeignFunc(..) => "#<foreign_func>".to_owned(),
                // #insight intentionally pass through the formatting.
                Expr::Annotated(expr, _) => format!("{expr}"),
                Expr::Module(module) => format!("Module({})", module.stem),
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

    // pub fn foreign_func(f: &ExprFn) -> Self {
    //     Expr::ForeignFunc(Arc::new(*f))
    // }

    pub fn annotated(expr: Expr) -> Self {
        Expr::Annotated(Box::new(expr), HashMap::new())
    }

    pub fn maybe_annotated(expr: Expr, annotations: Option<&HashMap<String, Expr>>) -> Self {
        if let Some(annotations) = annotations {
            // #todo do something about this clone!!
            Expr::Annotated(Box::new(expr), annotations.clone())
        } else {
            expr
        }
    }
}

impl Expr {
    pub fn annotations(&self) -> Option<&HashMap<String, Expr>> {
        match self {
            Expr::Annotated(_, ann) => Some(ann),
            _ => None,
        }
    }

    // #todo name unpack? / project?
    pub fn extract(&self) -> (&Expr, Option<&HashMap<String, Expr>>) {
        match self {
            Expr::Annotated(expr, ann) => (expr, Some(ann)),
            _ => (self, None),
        }
    }

    // #todo name unwrap?
    // #todo unpack is very dangerous, we need to encode in the typesystem that the expr is unpacked.
    // #todo unwrap into tuple (expr, ann)
    // #todo find better name?
    #[inline]
    pub fn unpack(&self) -> &Self {
        match self {
            Expr::Annotated(expr, _) => expr,
            _ => self,
        }
    }

    pub fn annotation(&self, name: impl Into<String>) -> Option<&Expr> {
        // #todo dangerous method, emit a warning if it's called on a non-annotated Expr !!!
        match self {
            Expr::Annotated(_, ann) => ann.get(&name.into()),
            _ => None,
        }
    }

    // #todo consider #[inline]
    pub fn as_int(&self) -> Option<i64> {
        let Expr::Int(n) = self.unpack() else {
            return None;
        };
        Some(*n)
    }

    pub fn as_float(&self) -> Option<f64> {
        let Expr::Float(n) = self.unpack() else {
            return None;
        };
        Some(*n)
    }

    pub fn as_decimal(&self) -> Option<Decimal> {
        let Expr::Dec(n) = self.unpack() else {
            return None;
        };
        Some(*n)
    }

    pub fn as_bool(&self) -> Option<bool> {
        let Expr::Bool(b) = self.unpack() else {
            return None;
        };
        Some(*b)
    }

    pub fn as_string(&self) -> Option<&str> {
        let Expr::String(s) = self.unpack() else {
            return None;
        };
        Some(s)
    }

    pub fn as_symbol(&self) -> Option<&str> {
        let Expr::Symbol(s) = self.unpack() else {
            return None;
        };
        Some(s)
    }

    // #todo add an extra function to extract all string-

    pub fn as_char(&self) -> Option<char> {
        let Expr::Char(c) = self.unpack() else {
            return None;
        };
        Some(*c)
    }

    pub fn as_list(&self) -> Option<&Vec<Expr>> {
        let Expr::List(v) = self.unpack() else {
            return None;
        };
        Some(v)
    }

    pub fn as_array(&self) -> Option<&Vec<Expr>> {
        let Expr::Array(v) = self.unpack() else {
            return None;
        };
        Some(v)
    }

    // // static vs dyn type.
    // pub fn static_type(&self) -> Expr {
    //     match self {
    //         Expr::Int(_) => return Expr::symbol("Int"),
    //         Expr::Float(_) => return Expr::symbol("Float"),
    //         _ => return Expr::symbol("Unknown"),
    //     }
    // }

    pub fn static_type(&self) -> &Expr {
        self.annotation("type").unwrap_or(&Expr::One)
    }

    pub fn dyn_type(&self, context: &Context) -> Expr {
        if let Some(typ) = self.annotation("type") {
            return typ.clone();
        }

        match self.unpack() {
            Expr::Int(_) => Expr::symbol("Int"),
            Expr::Float(_) => Expr::symbol("Float"),
            Expr::Symbol(name) => {
                if let Some(value) = context.scope.get(name) {
                    value.dyn_type(context)
                } else {
                    Expr::symbol("Unknown")
                }
            }
            _ => Expr::symbol("Unknown"),
        }
    }

    pub fn range(&self) -> Option<Range> {
        self.annotation("range").map(expr_to_range)
    }
}

#[must_use]
pub fn annotate(mut expr: Expr, name: impl Into<String>, ann_expr: Expr) -> Expr {
    match expr {
        Expr::Annotated(_, ref mut ann) => {
            ann.insert(name.into(), ann_expr);
            expr
        }
        expr => {
            let mut ann = HashMap::new();
            ann.insert(name.into(), ann_expr);
            Expr::Annotated(Box::new(expr), ann)
        }
    }
}

// #todo use special sigil for implicit/system annotations.

#[must_use]
pub fn annotate_type(expr: Expr, type_name: impl Into<String>) -> Expr {
    annotate(expr, "type", Expr::Symbol(type_name.into()))
}

#[must_use]
pub fn annotate_range(expr: Expr, range: Range) -> Expr {
    annotate(expr, "range", range_to_expr(&range))
}

// #todo move elsewhere, e.g. api.
// #todo think where this function is used. (it is used for Dict keys, hmm...)
// #todo this is a confusing name!
/// Formats the expression as a value
pub fn format_value(expr: impl AsRef<Expr>) -> String {
    let expr = expr.as_ref();
    match expr {
        Expr::Float(n) => format_float(*n),
        Expr::Annotated(expr, _) => format_value(expr),
        Expr::String(s) => s.to_string(),
        Expr::KeySymbol(s) => s.to_string(),
        _ => expr.to_string(),
    }
}

// ---

// #todo implement Defer into Expr!

// #todo convert to the Expr::Range variant.
// #todo convert position to Dict Expr.

pub fn position_to_expr(position: &Position) -> Expr {
    let mut map: HashMap<String, Expr> = HashMap::new();
    map.insert("index".to_owned(), Expr::Int(position.index as i64));
    map.insert("line".to_owned(), Expr::Int(position.line as i64));
    map.insert("col".to_owned(), Expr::Int(position.line as i64));
    Expr::Dict(map)
}

pub fn expr_to_position(expr: &Expr) -> Position {
    if let Expr::Dict(dict) = expr {
        let Some(Expr::Int(index)) = dict.get("index") else {
            // #todo fix me!
            return Position::default();
        };

        let Some(Expr::Int(line)) = dict.get("line") else {
            // #todo fix me!
            return Position::default();
        };

        let Some(Expr::Int(col)) = dict.get("col") else {
            // #todo fix me!
            return Position::default();
        };

        return Position {
            index: *index as usize,
            line: *line as usize,
            col: *col as usize,
        };
    }

    // #todo fix me!
    return Position::default();
}

pub fn range_to_expr(range: &Range) -> Expr {
    let start = position_to_expr(&range.start);
    let end = position_to_expr(&range.end);

    Expr::Array(vec![start, end])
}

// #todo nasty code.
pub fn expr_to_range(expr: &Expr) -> Range {
    // #todo error checking?
    let Expr::Array(terms) = expr else {
        // #todo hmm...
        return Range::default();
    };

    Range {
        start: expr_to_position(&terms[0]),
        end: expr_to_position(&terms[1]),
    }
}

// #todo use `.into()` to convert Expr to Annotated<Expr>.

#[cfg(test)]
mod tests {
    use crate::expr::Expr;

    #[test]
    fn expr_string_display() {
        let expr = Expr::string("hello");
        assert_eq!("\"hello\"", format!("{expr}"));
    }
}
