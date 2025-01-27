use color_eyre::{Result, Section};
use hashbrown::HashMap;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{env::Env, expression::Expression};

pub const DEBUG_MODE: bool = false;
pub static EVALUATION_COUNT: AtomicUsize = AtomicUsize::new(0);
#[allow(dead_code)]
pub static LAST_EVALUATION_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn eval_expression(env: &mut Rc<RefCell<Env>>, expr: &Expression) -> Result<Expression> {
    match expr {
        Expression::Integer(_)
        | Expression::String(_)
        | Expression::Builtin {
            name: _,
            function: _,
        }
        | Expression::Float(_)
        | Expression::Function {
            arguments: _,
            body: _,
        }
        | Expression::Table(_)
        | Expression::Nil => Ok(expr.clone()),
        Expression::Symbol(s) => Ok(env.borrow().get(s).unwrap_or(Expression::Nil)),
        Expression::List(l) => eval_list(env, l),
    }
}

pub fn eval_list(env: &mut Rc<RefCell<Env>>, list: &[Expression]) -> Result<Expression> {
    let mut caller = eval_expression(env, &list[0])?;

    while let Expression::List(_) = caller {
        caller = eval_expression(env, &caller)?;
    }

    EVALUATION_COUNT.fetch_add(1, Ordering::SeqCst);

    if DEBUG_MODE {
        println!("{}", Expression::List(list.to_vec()).as_debug_string());
    }

    match caller {
        Expression::Function {
            ref arguments,
            ref body,
        } => {
            if arguments.len() != list.len() - 1 || list.contains(&Expression::Symbol("_".into())) {
                if list.len() == 1 {
                    Ok(caller)
                } else if let Expression::List(_) = *(body.clone()) {
                    let current_arguments = &list[1..];

                    let new_arguments = &arguments[current_arguments.len()..];
                    let mut new_body = vec![caller.clone()];
                    new_body.extend(current_arguments.iter().cloned());

                    Ok(Expression::Function {
                        arguments: new_arguments.to_vec(),
                        body: Box::new(Expression::List(new_body)),
                    })
                } else {
                    Ok(Expression::Nil)
                }
            } else {
                let mut e = Rc::new(RefCell::new(Env {
                    parent: Some(env.clone()),
                    local: HashMap::new(),
                }));

                let current_arguments = &list[1..];

                for i in 0..arguments.len() {
                    e.as_ref().borrow_mut().set_local(
                        arguments[i].as_symbol_string()?,
                        eval_expression(env, &current_arguments[i])?,
                    );
                }

                eval_expression(&mut e, &body)
            }
        }
        // TODO: Partial application on Builtins
        Expression::Builtin { name: _, function } => function(env, &list[1..]).map_err(|e| {
            e.note(format!(
                "Evaluating: ({})",
                list.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ))
        }),
        Expression::List(l) => eval_list(env, &l),
        _ => Ok(caller),
    }
}
