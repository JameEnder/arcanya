use color_eyre::{eyre::eyre, Result};
use colored::Colorize;
// use core::hash::Hasher;
use hashbrown::HashMap;
// use std::hash::Hash;
use lazy_static::lazy_static;
use std::{cell::RefCell, rc::Rc};

use crate::env::Env;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    String(String),
    Symbol(String),
    List(Vec<Expression>),
    Table(HashMap<String, Expression>),
    Function {
        arguments: Vec<Expression>,
        body: Box<Expression>,
    },
    Builtin {
        name: &'static str,
        function: fn(&mut Rc<RefCell<Env>>, &[Expression]) -> Result<Expression>,
    },
    Nil,
}

lazy_static! {
    pub static ref NIL: Expression = Expression::Nil;
    pub static ref TRUE: Expression = Expression::Symbol(String::from("t"));
}

impl Expression {
    pub fn as_i64(&self) -> Result<i64> {
        if let Expression::Integer(i) = self {
            Ok(*i)
        } else {
            Err(eyre!("Not an integer: {}", self))
        }
    }

    pub fn as_f64(&self) -> Result<f64> {
        if let Expression::Float(f) = self {
            Ok(*f)
        } else {
            Err(eyre!("Not a float: {}", self))
        }
    }

    pub fn as_string(&self) -> Result<String> {
        if let Expression::String(s) = self {
            Ok(s.clone())
        } else {
            Err(eyre!("Not a string: {}", self))
        }
    }

    pub fn as_symbol_string(&self) -> Result<String> {
        if let Expression::Symbol(s) = self {
            Ok(s.clone())
        } else {
            Err(eyre!("Not a symbol: {}", self))
        }
    }

    pub fn as_boolean(&self) -> Result<bool> {
        Ok(!matches!(self, Expression::Nil))
    }

    pub fn as_list(&self) -> Result<Vec<Expression>> {
        if let Expression::List(l) = self {
            Ok(l.clone())
        } else {
            Err(eyre!("Not a list: {}", self))
        }
    }

    pub fn as_table(&self) -> Result<HashMap<String, Expression>> {
        if let Expression::Table(t) = self {
            Ok(t.clone())
        } else {
            Err(eyre!("Not a list: {}", self))
        }
    }

    pub fn as_type_string(&self) -> String {
        match self {
            Expression::Builtin {
                name: _,
                function: _,
            } => "builtin".to_string(),
            Expression::Function {
                arguments: _,
                body: _,
            } => "function".to_string(),
            Expression::List(_) => "list".to_string(),
            Expression::Integer(_) => "integer".to_string(),
            Expression::String(_) => "string".to_string(),
            Expression::Symbol(_) => "symbol".to_string(),
            Expression::Nil => "nil".to_string(),
            Expression::Float(_) => "float".to_string(),
            Expression::Table(_) => "table".to_string(),
        }
    }

    pub fn as_debug_string(&self) -> String {
        match self {
            Expression::Builtin { name, function: _ } => name.to_string(),
            Expression::Function { arguments: _, body } => body.as_debug_string(),
            Expression::List(list) => format!(
                "({})",
                list.iter()
                    .map(|item| item.as_debug_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Expression::Integer(i) => i.to_string(),
            Expression::Float(f) => f.to_string(),
            Expression::String(s) => format!("\"{s}\""),
            Expression::Symbol(s) => s.to_string(),
            Expression::Nil => "nil".to_string(),
            Expression::Table(table) => {
                if table.is_empty() {
                    "{}".to_string()
                } else {
                    format!(
                        "{{ {} }}",
                        table
                            .iter()
                            .map(|(key, value)| format!("{key}: {value}"))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
            }
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Expression::Builtin { name, function: _ } => format!("{}", name.yellow()),
                Expression::Function { arguments, body } => {
                    format!(
                        "{} : ({}) => {}",
                        "function".blue(),
                        arguments
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<String>>()
                            .join(" "),
                        body
                    )
                }
                Expression::List(list) => format!(
                    "({})",
                    list.iter()
                        .map(|l| l.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                ),
                Expression::Table(table) =>
                    if table.is_empty() {
                        "{}".to_string()
                    } else {
                        format!(
                            "{{ {} }}",
                            table
                                .iter()
                                .map(|(key, value)| format!("{key}: {value}"))
                                .collect::<Vec<String>>()
                                .join(", ")
                        )
                    },
                Expression::Integer(i) => i.to_string().yellow().to_string(),
                Expression::String(s) => format!("\"{}\"", s).green().to_string(),
                Expression::Symbol(s) => s.to_string(),
                Expression::Nil => "nil".to_string().purple().to_string(),
                Expression::Float(f) => format!("{:?}", f).yellow().to_string(),
            }
        )
    }
}

impl From<bool> for Expression {
    fn from(b: bool) -> Self {
        if b {
            TRUE.clone()
        } else {
            NIL.clone()
        }
    }
}
