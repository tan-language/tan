pub mod expr_iter;
pub mod expr_transform;

use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fmt,
    hash::{Hash, Hasher},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[cfg(feature = "dec")]
use rust_decimal::Decimal;

use crate::{
    context::Context,
    error::Error,
    lexer::comment::CommentKind,
    module::Module,
    range::{Position, Range},
    scope::Scope,
    util::{expect_lock_read, expect_lock_write, fmt::format_float},
};

// #todo #important optimization, implement clone_from!

// #todo make some Expr variants non annotatable (e.g. U8)

// #todo introduce Expr::Ref() with an Rc reference to avoid excessive cloning!

// #insight
// Annotations are only for named bindings, static-time pseudo-annotations for
// literals are resolved before dynamic-time, Expr::Ann() is useful for that! But,
// we can probably skip most expr.unpack()s.

// #insight use structs for enum values, is more ..structured, readable and can have methods.

// #insight Rc/Arc is used instead of Box to support Clone.

// #todo separate variant for list and apply/call (can this be defined statically?)
// #todo List, MaybeList, Call
// #todo Expr::Range()

// #insight
// AST = Expr = Value = Object

// #insight
// The use of Vec in the Expr enum, keeps the nested expressions in the heap.

// #insight
// No need for a Zero/Never/Nothing Expr variant?

// #todo what would be the 'default'? -> the 'Unit'/'One' type, Nil!
// #todo consider parsing to 'simple' Expr, only List and Symbols
// #todo optimize 'simple' Expr to 'execution' Expr
// #todo introduce ForeignValue?
// #todo ExprFn should get a single Expr? -> nah, it's foreign.

// #todo not all Expr variants really need Ann, maybe the annotation should be internal to Expr?

// #todo consider Visitor pattern instead of enum?

// #todo for ForeignFn
// #todo consider &mut and & Context, different types!
// #todo consider version with no Context
// #todo find a better name for the type-alias
// #todo add an option that gets a 'self' expression to allow for e.g. assert! implementation (uses self annotations)
// #todo maybe should pass 'self' to all foreign functions?

// #insight the `+ Send + Sync + 'static` suffix allows Expr to be Sync.

// #todo considere renaming to ExprContextMutFn and also provide an ExprContextFn.
/// A function that accepts a list of Exprs and a mut Context, returns maybe an Expr.
pub type ExprContextFn =
    dyn Fn(&[Expr], &mut Context) -> Result<Expr, Error> + Send + Sync + 'static;

// #todo not used yet.
/// A function that accepts a list of Exprs, returns maybe an Expr.
pub type ExprFn = dyn Fn(&[Expr]) -> Result<Expr, Error> + Send + Sync + 'static;

// #todo use normal structs instead of tuple-structs?

// #todo add Expr::Date
// #todo add Expr::Panic (catched by the runtime, should support unwind)

// #insight Maybe.None == Nil == Unit
// #insight (Maybe T) = (Or T Nil)

// #todo probably the Any/Never (i.e. Top/Bottom) types should not be encoded in Expr.

/// A symbolic expression. This is the 'universal' data type in the language,
/// all values are expressions (and expressions are values). Evaluation is expression
/// rewriting to a fixed point.
#[derive(Default, Clone)]
pub enum Expr {
    // --- Low-level ---
    // #todo Any <> Nothing or even Anything <> Nothing, better keep the shorter Any
    // Any is the Top type.
    // Any,
    // `Never` is the Bottom type (Zero). It's the empty type, a type without instances.
    // #insight Never is the 'zero' in algebraic sense (x+0 = x, x*0 = 0)
    // #insight In the Curry–Howard correspondence, an empty type corresponds to falsity.
    // #insight the Bottom type is the dual to the Top type (Any)
    Never,
    // `None` is the Unit type (One). It's a type with a single instance, and thus carries no information.
    // The single instance of `None` is `()` (none).
    // #insight Python also uses the term `None`.
    // #insight `()` is used to avoid reserving `nil`.
    // #insight Nil/Unit/One is the 'one' in algebraic sense (x+1 != 0, x*1 = x)
    // #insight Unit == One, and it _is_ 'one' in the algebraic sense
    // #insight None = (N)one
    // #insight preferred None over nil to play well with Maybe{Some,None}
    // #insight None is the default Expr value.
    #[default]
    None,
    Comment(String, CommentKind), // #todo consider renaming to Remark (REM)
    TextSeparator,                // for the formatter.
    Bool(bool),                   // #todo remove?
    // #todo consider `Byte`, `UInt8`?
    U8(u8),
    Int(i64),
    Float(f64),
    #[cfg(feature = "dec")]
    Dec(Decimal),
    Symbol(String),    // #todo consider renaming to Expr::Sym
    KeySymbol(String), // #todo consider renaming to Expr::Key
    Char(char),
    String(String),
    // #todo currently a special String for types.
    // #todo consider Typ
    Type(String),
    // #todo better name for 'generic' List, how about `Cons` or `ConsList` or `Cell`?
    // #todo add 'quoted' List -> Array!
    // #todo do we really need Vec here? Maybe Arc<[Expr]> is enough?
    List(Vec<Expr>),
    Array(Arc<RwLock<Vec<Expr>>>),       // #insight 'reference' type
    Buffer(usize, Arc<RwLock<Vec<u8>>>), // #insight 'reference' type
    // #todo different name?
    // #todo support Expr as keys?
    Map(Arc<RwLock<HashMap<String, Expr>>>),
    Set(Arc<RwLock<HashSet<Expr>>>),
    // #todo support `start..` and `..end` ranges.
    // #todo open-ended range with step can look like this: `start../2`
    // #todo have type render as (Range Int)
    IntRange(i64, i64, i64), // start, end, step #todo use a struct here,
    // #todo have type render as (Range Float)
    FloatRange(f64, f64, f64), // start, end, step #todo use a struct here,
    // Range(...),
    // #todo the Func should probably store the Module environment.
    // #todo maybe should have explicit do block?
    /// Func(params, body, func_scope, filename)
    Func(Vec<Expr>, Vec<Expr>, Arc<Scope>, String),
    // #todo add file_path to Macro
    // #todo maybe should have explicit do block?
    /// Macro(params, body)
    Macro(Vec<Expr>, Vec<Expr>),
    // #todo add file_path to ForeignFunc
    // #todo the ForeignFunc should probably store the Module environment.
    // #todo introduce a ForeignFuncMut for mutating scope? what would be a better name?
    // #todo #optimization: I could use symbol table for foreing funcs and just put an integer index here!
    ForeignFunc(Arc<ExprContextFn>), // #todo for some reason, Box is not working here!
    // #todo consider renaming to just `Foreign`,
    // #todo consider adding type-name field?
    // #todo to optimize consider using an index into a table of type-names.
    // #todo support both mutable and immutable foreignStructs
    ForeignStruct(Arc<dyn Any + Send + Sync + 'static>),
    ForeignStructMut(Arc<RwLock<dyn Any + Send + Sync + 'static>>),
    Error(String),
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
    Module(Arc<Module>),
}

impl Eq for Expr {}

// #todo think some more about this.
impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Comment(l0, l1), Self::Comment(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Int(l0), Self::Int(r0)) => l0 == r0,
            (Self::Float(l0), Self::Float(r0)) => l0 == r0,
            (Self::Dec(l0), Self::Dec(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::KeySymbol(l0), Self::KeySymbol(r0)) => l0 == r0,
            (Self::Type(l0), Self::Type(r0)) => l0 == r0,
            (Self::Char(l0), Self::Char(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            // #todo maybe should leave for the default discriminant case?
            // #todo equality not supported for Array, due to RwLock.
            // (Self::Array(..), Self::Array(..)) => false,
            // (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            // #todo equality not supported for Map, due to RwLock.
            // (Self::Map(l0), Self::Map(r0)) => l0 == r0,
            (Self::IntRange(l0, l1, l2), Self::IntRange(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            (Self::FloatRange(l0, l1, l2), Self::FloatRange(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            (Self::Func(..), Self::Func(..)) => false,
            (Self::Macro(l0, l1), Self::Macro(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::ForeignFunc(..), Self::ForeignFunc(..)) => false,
            (Self::ForeignStruct(..), Self::ForeignStruct(..)) => false,
            (Self::If(l0, l1, l2), Self::If(r0, r1, r2)) => l0 == r0 && l1 == r1 && l2 == r2,
            // #todo #think should unpack and ignore annotations?
            (Self::Annotated(l0, _l1), Self::Annotated(r0, _r1)) => l0.eq(r0),
            (Self::Module(..), Self::Module(..)) => false,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Int(n) => {
                0.hash(state);
                n.hash(state);
            }
            Self::String(s) => {
                0.hash(state);
                s.hash(state);
            }
            Self::Annotated(inner, _) => inner.hash(state),
            // Expr::Zero => todo!(),
            // Expr::One => todo!(),
            // Expr::Comment(_, _) => todo!(),
            // Expr::TextSeparator => todo!(),
            // Expr::Bool(_) => todo!(),
            // Expr::Int(_) => todo!(),
            // Expr::Float(_) => todo!(),
            // Expr::Dec(_) => todo!(),
            // Expr::Symbol(_) => todo!(),
            // Expr::KeySymbol(_) => todo!(),
            // Expr::Char(_) => todo!(),
            // Expr::String(_) => todo!(),
            // Expr::Type(_) => todo!(),
            // Expr::List(_) => todo!(),
            // Expr::Array(_) => todo!(),
            // Expr::Map(_) => todo!(),
            // Expr::Set(_) => todo!(),
            // Expr::IntRange(_, _, _) => todo!(),
            // Expr::FloatRange(_, _, _) => todo!(),
            // Expr::Func(_, _, _, _) => todo!(),
            // Expr::Macro(_, _) => todo!(),
            // Expr::ForeignFunc(_) => todo!(),
            // Expr::ForeignStruct(_) => todo!(),
            // Expr::Do => todo!(),
            // Expr::Let => todo!(),
            // Expr::If(_, _, _) => todo!(),
            // Expr::Annotated(_, _) => todo!(),
            // Expr::Module(_) => todo!(),
            _ => {
                println!("******** no hash computation: {self}");
            }
        }
    }
}

// #todo what is the Expr default? One (Unit/Any) or Zero (Noting/Never)
// #todo
// use Sexp notation here. actually not really, maybe it's good as it is,
// it's more a view into the Rust/Foreign wold.
impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Expr::Never => "⊥".to_owned(), // #todo maybe use an ASCII representation, e.g. `!` or `!!`
            // #insight `None` is more readable than `()` in the debug context.
            Expr::None => "None".to_owned(),
            Expr::Comment(s, _) => format!("Comment({s})"),
            Expr::TextSeparator => "<TEXT-SEPARATOR>".to_owned(),
            Expr::Bool(b) => format!("Bool({b})"),
            Expr::Symbol(s) => format!("Symbol({s})"),
            Expr::KeySymbol(s) => format!("KeySymbol({s})"),
            Expr::Type(s) => format!("Type({s})"),
            Expr::Char(c) => format!("Char({c})"),
            Expr::String(s) => format!("String(\"{s}\")"),
            Expr::U8(num) => format!("U8({num})"),
            Expr::Int(num) => format!("Int({num})"),
            Expr::Float(num) => format!("Float({num})"),
            #[cfg(feature = "dec")]
            Expr::Dec(num) => format!("Dec({num})"),
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
            Expr::Buffer(size, v) => format!("Buffer({size}, {v:?})"),
            Expr::Array(v) => format!("Array({v:?})"),
            Expr::Map(d) => format!("Map({d:?})"),
            Expr::Set(d) => format!("Set({d:?})"),
            Expr::IntRange(start, end, step) => format!("IntRange({start},{end},{step})"),
            Expr::FloatRange(start, end, step) => format!("FloatRange({start},{end},{step})"),
            Expr::Func(..) => "<FUNC>".to_owned(),
            Expr::Macro(..) => "<MACRO>".to_owned(),
            Expr::ForeignFunc(..) => "<FOREIGN-FUNC>".to_owned(),
            Expr::ForeignStruct(..) => "<FOREIGN-STRUCT>".to_owned(),
            Expr::ForeignStructMut(..) => "<FOREIGN-STRUCT-MUT>".to_owned(),
            // #todo find a better name than `reason`.
            // #todo add support for wrapping upstream errors
            // #todo add support for wrapping foreign (rust) errors
            Expr::Error(reason) => format!("Error({reason})"),
            Expr::Let => "let".to_owned(),
            // #todo properly format do, let, if, etc.
            Expr::If(_, _, _) => "if".to_owned(),
            // #todo uncomment only for debugging purposes!
            // Expr::Annotated(expr, ann) => format!("ANN({expr:?}, {ann:?})"),
            // #insight intentionally ignore annotations in formatting the formatting.
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
                Expr::Never => "⊥".to_owned(),
                Expr::None => "()".to_owned(),
                Expr::Comment(s, _) => format!(r#"(rem "{s}")"#), // #todo what would be a good representation?
                Expr::TextSeparator => "<TS>".to_owned(),
                Expr::Bool(b) => b.to_string(),
                Expr::U8(n) => n.to_string(),
                Expr::Int(n) => n.to_string(),
                Expr::Float(n) => n.to_string(),
                #[cfg(feature = "dec")]
                Expr::Dec(n) => format!("(Dec {n})"), // #todo 'literal', e.f. 1.23d or #Dec 1.23
                Expr::Symbol(s) => s.clone(),
                Expr::KeySymbol(s) => format!(":{s}"),
                Expr::Type(s) => s.clone(),
                Expr::Char(c) => format!(r#"(Char "{c}")"#), // #todo no char literal?
                Expr::String(s) => format!("\"{s}\""),
                Expr::Error(reason) => format!(r#"(Error "{reason}")"#),
                Expr::Do => "do".to_owned(),
                Expr::Let => "let".to_owned(),
                // #todo properly format if!
                Expr::If(..) => "if".to_owned(),
                Expr::List(terms) => {
                    format!(
                        "({})",
                        terms
                            .iter()
                            .map(|term| format!("{}", term))
                            .collect::<Vec<String>>()
                            .join(" ")
                    )
                }
                Expr::Buffer(_size, bytes) => {
                    let exprs = expect_lock_read(bytes)
                        .iter()
                        .map(|expr| expr.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("[{exprs}]")
                }
                Expr::Array(exprs) => {
                    let exprs = expect_lock_read(exprs)
                        .iter()
                        .map(|expr| expr.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("[{exprs}]")
                }
                Expr::Map(map) => {
                    // #todo Map should support arbitrary exprs (or at lease `(Into String)` exprs)
                    // #todo currently we convert keys to symbol, make this more subtle.
                    let exprs = expect_lock_read(map)
                        .iter()
                        .map(|(k, v)| format!(":{k} {v}"))
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("{{{exprs}}}")
                }
                Expr::Set(set) => {
                    // #todo Map should support arbitrary exprs (or at lease `(Into String)` exprs)
                    // #todo currently we convert keys to symbol, make this more subtle.
                    let exprs = expect_lock_read(set)
                        .iter()
                        .map(|v| format!("{v}"))
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("[{exprs}]")
                }
                Expr::IntRange(start, end, step) => {
                    if *step == 1 {
                        format!("{start}..{end}")
                    } else {
                        // #insight cannot use `/`
                        // #todo consider using `:` or `,` instead of `|`?
                        format!("{start}..{end}|{step}")
                    }
                }
                Expr::FloatRange(start, end, step) => {
                    if *step == 1.0 {
                        format!("{start}..{end}")
                    } else {
                        // #insight cannot use `/`
                        // #todo consider using `:` or `,` instead of `|`?
                        format!("{start}..{end}|{step}")
                    }
                }
                Expr::Func(..) => "#<func>".to_owned(),
                Expr::Macro(..) => "#<func>".to_owned(),
                Expr::ForeignFunc(..) => "#<foreign-func>".to_owned(),
                Expr::ForeignStruct(..) => "#<foreign-struct>".to_owned(),
                Expr::ForeignStructMut(..) => "#<foreign-struct-mut>".to_owned(),
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

    pub fn typ(s: impl Into<String>) -> Self {
        Expr::Type(s.into())
    }

    pub fn array(a: impl Into<Vec<Expr>>) -> Self {
        Expr::Array(Arc::new(RwLock::new(a.into())))
    }

    pub fn map(m: impl Into<HashMap<String, Expr>>) -> Self {
        Expr::Map(Arc::new(RwLock::new(m.into())))
    }

    pub fn set(s: impl Into<HashSet<Expr>>) -> Self {
        Expr::Set(Arc::new(RwLock::new(s.into())))
    }

    // pub fn foreign_func(f: &ExprFn) -> Self {
    //     Expr::ForeignFunc(Arc::new(*f))
    // }

    pub fn annotated(expr: Expr, annotations: &HashMap<String, Expr>) -> Self {
        // #insight don't override existing annotations.
        let mut expr = expr;
        if let Some(current_annotations) = expr.annotations_mut() {
            for (k, v) in annotations {
                if !current_annotations.contains_key(k) {
                    current_annotations.insert(k.clone(), v.clone());
                }
            }
            expr
        } else {
            // #todo do something about this clone!!
            Expr::Annotated(Box::new(expr), annotations.clone())
        }
    }

    pub fn maybe_annotated(expr: Expr, annotations: Option<&HashMap<String, Expr>>) -> Self {
        if let Some(annotations) = annotations {
            Self::annotated(expr, annotations)
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

    pub fn annotations_mut(&mut self) -> Option<&mut HashMap<String, Expr>> {
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
    /// Removes the annotation from an expression.
    #[inline]
    pub fn unpack(&self) -> &Self {
        match self {
            Expr::Annotated(expr, _) => expr,
            _ => self,
        }
    }

    #[inline]
    pub fn unpack_consuming(self) -> Self {
        match self {
            Expr::Annotated(expr, _) => *expr,
            _ => self,
        }
    }

    pub fn annotation(&self, name: impl Into<String>) -> Option<&Expr> {
        match self {
            Expr::Annotated(_, ann) => ann.get(&name.into()),
            _ => None,
        }
    }

    // #todo do we _really_ want Expr::Nil as value/variant?
    // #todo rename to is_none
    // #todo is_one/is_unit
    pub fn is_none(&self) -> bool {
        matches!(self.unpack(), Expr::None)
    }

    // #todo do we really need this? or we should always use `is_invocable`?
    pub fn is_func(&self) -> bool {
        matches!(self.unpack(), Expr::Func(..))
    }

    pub fn is_invocable(&self) -> bool {
        matches!(self.unpack(), Expr::Func(..) | Expr::ForeignFunc(..))
    }

    // #todo remove TextSeparator concept.
    // #todo find a better name.
    // Returns true if the expresion is 'transient'/'inept' i.e. it will
    // be stripped before evaluation. Transient helpers are currently used
    // for analysis, not evaluation.
    pub fn is_transient(&self) -> bool {
        matches!(self.unpack(), Expr::Comment(..) | Expr::TextSeparator)
    }

    // #insight
    // We provide is_false() instead of is_true() as in the future we _may_
    // consider all non-false values as true.
    pub fn is_false(&self) -> bool {
        matches!(self.unpack(), Expr::Bool(false))
    }

    // #todo consider #[inline]
    pub fn as_int(&self) -> Option<i64> {
        let Expr::Int(n) = self.unpack() else {
            return None;
        };
        Some(*n)
    }

    pub fn as_u8(&self) -> Option<u8> {
        let Expr::U8(n) = self.unpack() else {
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

    #[cfg(feature = "dec")]
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

    // #todo
    // pub fn as_string_mut(&self) -> Option<RefMut<'_, &String>> {
    //     // #todo how to implement this?
    //     todo!()
    // }

    // #insight https://en.wiktionary.org/wiki/stringable
    pub fn as_stringable(&self) -> Option<&str> {
        // #todo try to optimize away the unpacks.
        let expr = self.unpack();

        match expr {
            Expr::Symbol(s) => Some(s),
            Expr::KeySymbol(s) => Some(s),
            Expr::String(s) => Some(s),
            Expr::Type(s) => Some(s),
            _ => None,
        }
    }

    // #insight useful for optimizations.
    pub fn as_stringable_consuming(self) -> Option<String> {
        // #todo try to optimize away the unpacks.
        let expr = self.unpack_consuming();

        match expr {
            Expr::Symbol(s) => Some(s),
            Expr::KeySymbol(s) => Some(s),
            Expr::String(s) => Some(s),
            Expr::Type(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_symbol(&self) -> Option<&str> {
        let Expr::Symbol(s) = self.unpack() else {
            return None;
        };
        Some(s)
    }

    pub fn as_key_symbol(&self) -> Option<&str> {
        let Expr::KeySymbol(s) = self.unpack() else {
            return None;
        };
        Some(s)
    }

    // #todo can just make this the as_symbol impl.
    /// Tries to extract Symbol or KeySymbol.
    pub fn as_symbolic(&self) -> Option<&str> {
        // #todo try to optimize away the unpacks.
        let expr = self.unpack();

        match expr {
            Expr::Symbol(s) => Some(s),
            Expr::KeySymbol(s) => Some(s),
            Expr::Type(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_type(&self) -> Option<&str> {
        let Expr::Type(s) = self.unpack() else {
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

    pub fn as_array(&self) -> Option<RwLockReadGuard<'_, Vec<Expr>>> {
        let Expr::Array(v) = self.unpack() else {
            return None;
        };
        // #todo what would be a good message?
        // #todo extract as variable.
        Some(expect_lock_read(v))
    }

    pub fn as_array_consuming(self) -> Option<Arc<RwLock<Vec<Expr>>>> {
        let Expr::Array(v) = self.unpack_consuming() else {
            return None;
        };
        Some(v)
    }

    // // #todo try to find a better name.
    // pub fn as_seq(&self) -> Option<&Vec<Expr>> {
    //     // #todo try to optimize away the unpacks.
    //     let expr = self.unpack();

    //     match expr {
    //         Expr::List(v) => Some(v),
    //         Expr::Array(v) => Some(???), // #todo how to implement this?
    //         _ => None,
    //     }
    // }

    pub fn as_array_mut(&self) -> Option<RwLockWriteGuard<'_, Vec<Expr>>> {
        let Expr::Array(v) = self.unpack() else {
            return None;
        };
        Some(expect_lock_write(v))
    }

    pub fn as_buffer(&self) -> Option<RwLockReadGuard<'_, Vec<u8>>> {
        // #todo what to do with size/length?
        let Expr::Buffer(_, v) = self.unpack() else {
            return None;
        };
        // #todo what would be a good message?
        // #todo extract as variable.
        Some(expect_lock_read(v))
    }

    // #insight you _always_ need the length/size when mutating a buffer.
    pub fn as_buffer_mut(&self) -> Option<(usize, RwLockWriteGuard<'_, Vec<u8>>)> {
        // #todo what to do with size/length?
        let Expr::Buffer(length, v) = self.unpack() else {
            return None;
        };
        Some((*length, expect_lock_write(v)))
    }

    pub fn as_map(&self) -> Option<RwLockReadGuard<'_, HashMap<String, Expr>>> {
        let Expr::Map(map) = self.unpack() else {
            return None;
        };
        Some(expect_lock_read(map))
    }

    pub fn as_map_mut(&self) -> Option<RwLockWriteGuard<'_, HashMap<String, Expr>>> {
        let Expr::Map(map) = self.unpack() else {
            return None;
        };
        Some(expect_lock_write(map))
    }

    pub fn as_set(&self) -> Option<RwLockReadGuard<'_, HashSet<Expr>>> {
        let Expr::Set(set) = self.unpack() else {
            return None;
        };
        Some(expect_lock_read(set))
    }

    pub fn as_set_mut(&self) -> Option<RwLockWriteGuard<'_, HashSet<Expr>>> {
        let Expr::Set(set) = self.unpack() else {
            return None;
        };
        Some(expect_lock_write(set))
    }

    // // #todo consider #[inline]
    // pub fn as_func(&self) -> Option<i64> {
    //     let Expr::Func(params, body, scope, filename) = self.unpack() else {
    //         return None;
    //     };
    //     Some(...)
    // }

    // // static vs dyn type.
    // pub fn static_type(&self) -> Expr {
    //     match self {
    //         Expr::Int(_) => return Expr::symbol("Int"),
    //         Expr::Float(_) => return Expr::symbol("Float"),
    //         _ => return Expr::symbol("Unknown"),
    //     }
    // }

    pub fn static_type(&self) -> &Expr {
        self.annotation("type").unwrap_or(&Expr::None)
    }

    // #todo we need a version that returns just a string.

    // #todo introduce a version that does not look-through Symbol, no Context required.
    // #todo how about return &Expr to avoid clones?
    // #todo alternatively consider key-symbol instead of String
    // #insight use string for the type to support parameterized types, e.g (Map String Any)
    // Returns the dynamic (eval-time) type of the expression.
    pub fn dyn_type(&self, context: &Context) -> Expr {
        // #todo make constant out of "type".
        if let Some(typ) = self.annotation("type") {
            // #todo why is the unpack needed?
            return typ.unpack().clone();
        }

        match self.unpack() {
            Expr::Never => Expr::typ("Never"), // Never, Zero
            Expr::None => Expr::typ("None"),   // Unit, One, Nil
            Expr::Bool(_) => Expr::typ("Bool"),
            Expr::U8(_) => Expr::typ("U8"),
            Expr::Int(_) => Expr::typ("Int"),
            Expr::Float(_) => Expr::typ("Float"),
            Expr::Dec(_) => Expr::typ("Dec"),
            Expr::String(_) => Expr::typ("String"),
            Expr::Type(_) => Expr::typ("Type"),
            Expr::List(_) => Expr::typ("List"), // #todo return parameterized type
            Expr::Array(_) => Expr::typ("Array"), // #todo return parameterized type
            Expr::Buffer(..) => Expr::typ("Buffer"), // #todo return parameterized type
            Expr::Map(_) => Expr::typ("Map"),   // #todo return parameterized type
            Expr::Set(_) => Expr::typ("Set"),   // #todo return parameterized type
            // #todo what about quoted Symbol?
            Expr::Symbol(name) => {
                // #todo it's weird that we look through symbols.
                if let Some(value) = context.scope.get(name) {
                    value.dyn_type(context)
                } else {
                    // #todo could use symbol here!
                    Expr::typ("Unknown")
                }
            }
            Expr::KeySymbol(..) => Expr::typ("KeySymbol"),
            // #todo keep the Range type parameter as a ...parameter
            Expr::IntRange(..) => Expr::typ("(Range Int)"),
            Expr::FloatRange(..) => Expr::typ("(Range Float)"),
            Expr::Func(..) => Expr::typ("Func"),
            // #todo consider returning Func?
            Expr::ForeignFunc(..) => Expr::typ("ForeignFunc"),
            // #todo add more here!
            // #todo the wildcard is very error-prone, cover all cases!
            _ => {
                eprintln!("WARNING dyn-type unknown ---> {self:?}");
                Expr::typ("Unknown")
            }
        }
    }

    pub fn range(&self) -> Option<Range> {
        self.annotation("range").map(expr_to_range)
    }
}

impl From<i64> for Expr {
    fn from(item: i64) -> Self {
        Expr::Int(item)
    }
}

impl From<f64> for Expr {
    fn from(item: f64) -> Self {
        Expr::Float(item)
    }
}

// #todo impl TryFrom<Expr> for f64, i64, etc.

#[must_use]
pub fn annotate(mut expr: Expr, name: impl Into<String>, ann_expr: Expr) -> Expr {
    let name = name.into();
    match expr {
        Expr::Annotated(_, ref mut ann) => {
            ann.insert(name, ann_expr);
            expr
        }
        expr => {
            let mut ann = HashMap::new();
            ann.insert(name, ann_expr);
            Expr::Annotated(Box::new(expr), ann)
        }
    }
}

// #todo use special sigil for implicit/system annotations.

#[must_use]
pub fn annotate_type(expr: Expr, type_name: impl Into<String>) -> Expr {
    // #todo String is not good, we need a symbol/key-symbol that supports spaces.
    annotate(expr, "type", Expr::Type(type_name.into()))
}

// #insight it checks exclusively for annotation, maybe too error-prone.
pub fn has_type_annotation(expr: &Expr, type_name: &str) -> bool {
    // #todo should also check, dyn_type.
    if let Some(typ) = expr.annotation("type") {
        if let Some(name) = typ.as_stringable() {
            name == type_name
        } else {
            false
        }
    } else {
        false
    }
}

// #todo we need a version without Context, duh!
pub fn has_dyn_type(expr: &Expr, type_name: &str, context: &Context) -> bool {
    let typ = expr.dyn_type(context);

    if let Some(name) = typ.as_stringable() {
        name == type_name
    } else {
        false
    }
}

#[must_use]
pub fn annotate_range(expr: Expr, range: Range) -> Expr {
    annotate(expr, "range", range_to_expr(&range))
}

// #todo move elsewhere, e.g. api.
// #todo think where this function is used. (it is used for Map keys, hmm...)
// #todo this is a confusing name!
/// Formats the expression as a value.
/// For example strings are formatted without the quotes and keys without
/// the `:` prefix.
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

// #todo consider using Arc<Expr> everywhere?
// #todo proper name
// #todo proper value/reference handling for all types.
/// Clones expressions in optimized way, handles ref types.
pub fn expr_clone(expr: &Expr) -> Expr {
    match expr {
        // #insight treat Array and Map as a 'reference' types, Arc.clone is efficient.
        Expr::Array(items) => Expr::Array(items.clone()),
        Expr::Map(items) => Expr::Map(items.clone()),
        _ => expr.clone(),
    }
}

// ---

// #todo implement Defer into Expr!

// #todo convert to the Expr::Range variant.
// #todo convert position to Map Expr.

pub fn position_to_expr(position: &Position) -> Expr {
    let mut map: HashMap<String, Expr> = HashMap::new();
    map.insert("index".to_owned(), Expr::Int(position.index as i64));
    map.insert("line".to_owned(), Expr::Int(position.line as i64));
    map.insert("col".to_owned(), Expr::Int(position.col as i64));
    Expr::map(map)
}

pub fn expr_to_position(expr: &Expr) -> Position {
    if let Some(map) = expr.as_map() {
        let Some(Expr::Int(index)) = map.get("index") else {
            // #todo fix me!
            return Position::default();
        };

        let Some(Expr::Int(line)) = map.get("line") else {
            // #todo fix me!
            return Position::default();
        };

        let Some(Expr::Int(col)) = map.get("col") else {
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
    Position::default()
}

pub fn range_to_expr(range: &Range) -> Expr {
    let start = position_to_expr(&range.start);
    let end = position_to_expr(&range.end);

    Expr::array(vec![start, end])
}

// #todo nasty code.
pub fn expr_to_range(expr: &Expr) -> Range {
    // #todo error checking?
    // let Expr::Array(terms) = expr else {
    //     // #todo hmm...
    //     return Range::default();
    // };

    let Some(terms) = expr.as_array() else {
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

    #[test]
    fn expr_is_false() {
        assert!(Expr::Bool(false).is_false());
        assert!(!Expr::Bool(true).is_false());
    }
}
