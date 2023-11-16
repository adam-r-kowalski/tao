extern crate alloc;

use crate::effect::Effect;
use crate::numerics::Float;
use crate::NativeType;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{self, Display, Formatter};
use core::future::Future;
use core::hash::Hash;
use core::pin::Pin;
use im::{OrdMap, Vector};
use rug::{Integer, Rational};

type Expressions = Vector<Expression>;

pub type Environment = OrdMap<String, Expression>;

pub type Result = core::result::Result<(Environment, Expression), Effect>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pattern {
    pub parameters: Expressions,
    pub body: Vector<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Function {
    pub env: Environment,
    pub patterns: Vector<Pattern>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Call {
    pub function: Box<Expression>,
    pub arguments: Expressions,
}

type NativeFunction = fn(Environment, Expressions) -> Pin<Box<dyn Future<Output = Result> + Send>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Expression {
    Symbol(String),
    NamespacedSymbol(Vec<String>),
    Keyword(String),
    String(String),
    Integer(Integer),
    Float(Float),
    Ratio(Rational),
    Bool(bool),
    Nil,
    Array(Expressions),
    Map(OrdMap<Expression, Expression>),
    Call(Call),
    Function(Function),
    Quote(Box<Expression>),
    NativeFunction(NativeFunction),
    NativeType(NativeType),
    Module(Environment),
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Map(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", {} {}", k, v)?;
                    } else {
                        write!(f, "{} {}", k, v)?;
                    }
                }
                write!(f, "}}")
            }
            Expression::Array(arr) => {
                write!(f, "[")?;
                for (i, e) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", {}", e)?;
                    } else {
                        write!(f, "{}", e)?;
                    }
                }
                write!(f, "]")
            }
            Expression::Symbol(s) => write!(f, "{}", s),
            Expression::NamespacedSymbol(s) => write!(f, "{}", s.join("/")),
            Expression::Keyword(k) => write!(f, "{}", k),
            Expression::String(s) => {
                write!(f, "\"")?;
                for c in s.chars() {
                    match c {
                        '\n' => write!(f, "\\n")?,
                        '\t' => write!(f, "\\t")?,
                        '\r' => write!(f, "\\r")?,
                        '\\' => write!(f, "\\\\")?,
                        '"' => write!(f, "\\\"")?,
                        _ => write!(f, "{}", c)?,
                    }
                }
                write!(f, "\"")
            }
            Expression::Integer(i) => write!(f, "{}", i),
            Expression::Float(fl) => write!(f, "{}", fl),
            Expression::Ratio(r) => write!(f, "{}/{}", r.numer(), r.denom()),
            Expression::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Expression::Nil => write!(f, "nil"),
            Expression::Call(Call {
                function,
                arguments,
            }) => {
                let arg_strs: Vec<String> = arguments.iter().map(|e| format!("{}", e)).collect();
                write!(f, "({} {})", function, arg_strs.join(" "))
            }
            Expression::Function(function) => {
                if function.patterns.len() == 1 {
                    let Pattern { parameters, body } = &function.patterns[0];
                    let param_strs: Vec<String> =
                        parameters.iter().map(|e| format!("{}", e)).collect();
                    write!(f, "(fn [{}] ", param_strs.join(" "))?;
                    body.iter().try_for_each(|e| write!(f, "{} ", e))?;
                    write!(f, ")")
                } else {
                    write!(f, "(fn")?;
                    for Pattern { parameters, body } in &function.patterns {
                        let param_strs: Vec<String> =
                            parameters.iter().map(|e| format!("{}", e)).collect();
                        write!(f, "\n  ([{}] ", param_strs.join(" "))?;
                        body.iter().try_for_each(|e| write!(f, "{} ", e))?;
                        write!(f, ")")?;
                    }
                    write!(f, ")")
                }
            }
            Expression::NativeFunction(_) => write!(f, "#native_function"),
            Expression::NativeType(t) => write!(f, "{}", t),
            Expression::Quote(e) => write!(f, "'{}", e),
            Expression::Module(e) => write!(
                f,
                "#module({})",
                e.get("*name*").unwrap_or(&Expression::Nil)
            ),
        }
    }
}
