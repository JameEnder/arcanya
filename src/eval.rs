use anyhow::{anyhow, Result};
use hashbrown::HashMap;
use std::{cell::RefCell, rc::Rc, sync::Mutex};

use crate::{Env, Expression};

pub static EVALUATION_COUNT: Mutex<i64> = Mutex::new(0);
#[allow(dead_code)]
pub static LAST_EVALUATION_COUNT: Mutex<i64> = Mutex::new(0);

pub fn eval_expression(env: &mut Rc<RefCell<Env>>, expr: Expression) -> Result<Expression> {
    match expr {
        Expression::Integer(_)
        | Expression::String(_)
        | Expression::Builtin(_)
        | Expression::Float(_)
        | Expression::Boolean(_)
        | Expression::Function {
            arguments: _,
            body: _,
        }
        | Expression::Named { name: _, value: _ }
        | Expression::Void => Ok(expr),

        Expression::Symbol(s) => {
            let mut value = env.borrow().get(s).unwrap_or(Expression::Void);

            while let Expression::Symbol(s) = value {
                value = env.borrow().get(s).unwrap();
            }

            if let Expression::Symbol(s) = value {
                env.borrow()
                    .get(s.clone())
                    .ok_or(anyhow!(format!("Symbol {s} not bound")))
            } else {
                Ok(value.clone())
            }
        }
        Expression::List(ref list) => {
            if (matches!(list[0], Expression::Symbol(_))
                && env.borrow().get(list[0].as_symbol_string()?).is_some())
                || matches!(
                    eval_expression(env, list[0].clone()),
                    Ok(Expression::Function {
                        arguments: _,
                        body: _
                    })
                )
            {
                eval_call(env, list)
            } else {
                Ok(expr)
            }
        }
    }
}

pub fn eval_call(env: &mut Rc<RefCell<Env>>, list: &[Expression]) -> Result<Expression> {
    let mut caller = eval_expression(env, list[0].clone())?;

    while let Expression::List(_) = caller {
        caller = eval_expression(env, caller)?;
    }

    *EVALUATION_COUNT.lock().unwrap() += 1;

    match caller {
        Expression::Function { arguments, body } => {
            let mut e = Rc::new(RefCell::new(Env {
                parent: Some(env.clone()),
                local: HashMap::new(),
            }));

            if arguments.len() != list.len() - 1 || list.contains(&Expression::Symbol("_".into())) {
                if let Expression::List(body_list) = *body {
                    let mut specified_arguments_map = HashMap::new();

                    for i in 1..list.len() {
                        if list[i] != Expression::Symbol("_".into()) {
                            specified_arguments_map
                                .insert(arguments[i - 1].as_symbol_string()?, list[i].clone());
                        }
                    }

                    let new_body = body_list
                        .iter()
                        .map(|x| {
                            x.as_symbol_string()
                                .ok()
                                .and_then(|s| specified_arguments_map.get(&s))
                                .unwrap_or(x)
                        })
                        .cloned()
                        .collect();

                    let new_arguments = arguments
                        .iter()
                        .filter(|arg| {
                            !arg.as_symbol_string()
                                .is_ok_and(|s| specified_arguments_map.contains_key(&s))
                        })
                        .cloned()
                        .collect();

                    Ok(Expression::Function {
                        arguments: new_arguments,
                        body: Box::new(Expression::List(new_body)),
                    })
                } else {
                    Ok(Expression::Void)
                }
            } else {
                for i in 0..arguments.len() {
                    let mut value = list[i + 1].clone();

                    value = eval_expression(env, value)?;

                    e.as_ref()
                        .borrow_mut()
                        .set_local(arguments[i].clone().as_symbol_string()?, value);
                }

                eval_expression(&mut e, *body)
            }
        }
        // TODO: Partial application on Builtins
        Expression::Builtin(f) => f(env, &list[1..]),
        Expression::List(l) => eval_call(env, &l),
        _ => Ok(caller),
    }
}
