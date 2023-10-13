#![feature(test)]

use anyhow::Result;
use std::io::{BufRead, Write};
use std::{cell::RefCell, rc::Rc};

pub mod builtin;
pub mod env;
pub mod eval;
pub mod expression;
pub mod parse;
#[cfg(test)]
mod tests;

use env::Env;
use eval::{eval_expression, EVALUATION_COUNT, LAST_EVALUATION_COUNT};
use expression::Expression;
use parse::parse_expression;

fn main() -> Result<()> {
    let mut global = Rc::new(RefCell::new(Env::new(None)));

    global.borrow_mut().extend(builtin::std_lib());

    let mut buffer = String::new();

    loop {
        buffer.clear();

        let mut lock = std::io::stdout().lock();
        write!(lock, "> ")?;
        std::io::stdout().flush()?;

        let stdin = std::io::stdin();
        let mut handle = stdin.lock();

        handle.read_line(&mut buffer)?;

        let returned = run(&mut global, &buffer);

        match returned {
            Ok(value) => println!("=> {}", value),
            Err(err) => println!("{:?}", err),
        }
    }
}

pub fn run(env: &mut Rc<RefCell<Env>>, input: &str) -> Result<Expression> {
    parse_expression(input)
        .map(|(_, expr)| eval_expression(env, expr))
        .map_err(|e| e.to_owned())?
}

#[allow(dead_code)]
fn run_log(env: &mut Rc<RefCell<Env>>, input: &str) -> Result<Expression> {
    let value = parse_expression(input)
        .map(|(_, expr)| eval_expression(env, expr))
        .map_err(|e| e.to_owned())?;

    println!(
        "Evaluation count: {}",
        *EVALUATION_COUNT.lock().unwrap() - *LAST_EVALUATION_COUNT.lock().unwrap()
    );

    *LAST_EVALUATION_COUNT.lock().unwrap() = *EVALUATION_COUNT.lock().unwrap();

    value
}
