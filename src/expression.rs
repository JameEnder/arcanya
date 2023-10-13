use anyhow::Result;
use std::{cell::RefCell, rc::Rc};

use crate::env::Env;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    String(String),
    Symbol(String),
    List(Vec<Expression>),
    Function {
        arguments: Vec<Expression>,
        body: Box<Expression>,
    },
    Named {
        name: String,
        value: Box<Expression>,
    },
    Builtin(fn(&mut Rc<RefCell<Env>>, &[Expression]) -> Result<Expression>),
    Boolean(bool),
    Void,
}

impl Expression {
    pub fn as_i64(&self) -> Result<i64> {
        if let Expression::Integer(i) = self {
            Ok(*i)
        } else {
            Err(anyhow::anyhow!(format!("Not an integer {:?}", self)))
        }
    }

    pub fn as_f64(&self) -> Result<f64> {
        if let Expression::Float(f) = self {
            Ok(*f)
        } else {
            Err(anyhow::anyhow!(format!("Not a float {:?}", self)))
        }
    }

    pub fn as_string(&self) -> Result<String> {
        if let Expression::String(s) = self {
            Ok(s.clone())
        } else {
            Err(anyhow::anyhow!(format!("Not a string {:?}", self)))
        }
    }

    pub fn as_symbol_string(&self) -> Result<String> {
        if let Expression::Symbol(s) = self {
            Ok(s.clone())
        } else {
            Err(anyhow::anyhow!(format!("Not a symbol {:?}", self)))
        }
    }

    pub fn as_boolean(&self) -> Result<bool> {
        if let Expression::Boolean(b) = self {
            Ok(*b)
        } else {
            Err(anyhow::anyhow!(format!("Not a bool {:?}", self)))
        }
    }

    pub fn as_list(&self) -> Result<Vec<Expression>> {
        if let Expression::List(l) = self {
            Ok(l.clone())
        } else {
            Err(anyhow::anyhow!(format!("Not a list {:?}", self)))
        }
    }

    pub fn as_type_string(&self) -> String {
        match self {
            Expression::Builtin(_) => "builtin".to_string(),
            Expression::Function {
                arguments: _,
                body: _,
            } => "function".to_string(),
            Expression::List(_) => "list".to_string(),
            Expression::Integer(_) => "integer".to_string(),
            Expression::String(_) => "string".to_string(),
            Expression::Symbol(_) => "symbol".to_string(),
            Expression::Named { name: _, value: _ } => "named".to_string(),
            Expression::Void => "void".to_string(),
            Expression::Float(_) => "float".to_string(),
            Expression::Boolean(_) => "boolean".to_string(),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Expression::Builtin(_) => "builtin".to_string(),
                Expression::Function { arguments, body } => {
                    format!(
                        "function : ({}) => {}",
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
                Expression::Integer(i) => i.to_string(),
                Expression::String(s) => s.to_string(),
                Expression::Symbol(s) => s.to_string(),
                Expression::Named { name, value } => format!(":{} {}", name, value),
                Expression::Void => "void".to_string(),
                Expression::Float(f) => format!("{:?}", f),
                Expression::Boolean(b) => b.to_string(),
            }
        )
    }
}
