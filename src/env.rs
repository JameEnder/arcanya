use hashbrown::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::expression::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    pub parent: Option<Rc<RefCell<Env>>>,
    pub local: HashMap<String, Expression>,
}

impl Env {
    pub fn new(parent: Option<Rc<RefCell<Env>>>) -> Env {
        Env {
            parent,
            local: HashMap::from([
                ("__EXPORTED".to_string(), Expression::Table(HashMap::new())),
                ("__IMPORTED".to_string(), Expression::Table(HashMap::new())),
            ]),
        }
    }

    pub fn get(&self, symbol: &str) -> Option<Expression> {
        self.local
            .get(symbol)
            .cloned()
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|parent| parent.borrow().get(symbol))
            })
            .or_else(|| {
                self.get("__IMPORTED")
                    .unwrap()
                    .as_table()
                    .unwrap()
                    .get(symbol)
                    .cloned()
            })
    }

    pub fn get_mut_local(&mut self, symbol: String) -> Option<&mut Expression> {
        self.local.get_mut(&symbol)
    }

    pub fn set_local(&mut self, symbol: String, value: Expression) {
        self.local.insert(symbol, value);
    }

    pub fn set_parent(&mut self, symbol: String, value: Expression) {
        if let Some(parent) = &self.parent {
            parent.as_ref().borrow_mut().set_local(symbol, value);
        } else {
            self.set_local(symbol, value);
        }
    }

    pub fn set_global(&mut self, symbol: String, value: Expression) {
        if let Some(parent) = &self.parent {
            parent.as_ref().borrow_mut().set_parent(symbol, value);
        } else {
            self.set_local(symbol, value);
        }
    }

    pub fn extend(&mut self, other: Env) {
        self.local.extend(other.local);
    }
}
