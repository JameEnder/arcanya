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
        Expression::List(l) => eval_list(env, &l),
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
                    Ok(Expression::Nil)
                }
            } else {
                for i in 0..arguments.len() {
                    e.as_ref().borrow_mut().set_local(
                        arguments[i].as_symbol_string()?,
                        eval_expression(env, &list[i + 1])?,
                    );
                }

                eval_expression(&mut e, &*body)
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
