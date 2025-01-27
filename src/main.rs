use color_eyre::Result;
use std::io::{BufRead, Write};
use std::sync::atomic::Ordering;
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
    color_eyre::install()?;

    let mut global = Rc::new(RefCell::new(builtin::std_lib()));

    let file_path = std::env::args().nth(1);

    if let Some(file_path) = file_path {
        let content = std::fs::read_to_string(file_path)?;
        let content = content.trim();

        let returned = run(&mut global, &content);

        match returned {
            Ok(value) => println!("=> {}", value),
            Err(err) => println!("{:?}", err),
        }

        Ok(())
    } else {
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
}

pub fn run(env: &mut Rc<RefCell<Env>>, input: &str) -> Result<Expression> {
    match parse_expression(input) {
        Ok((rest, expr)) => {
            let result = eval_expression(env, &expr);
            let rest = rest.trim();

            if !rest.is_empty() {
                run(env, rest)
            } else {
                result
            }
        }
        Err(e) => Err(e.to_owned())?,
    }
}

#[allow(dead_code)]
fn run_log(env: &mut Rc<RefCell<Env>>, input: &str) -> Result<Expression> {
    let value = parse_expression(input)
        .map(|(_, expr)| eval_expression(env, &expr))
        .map_err(|e| e.to_owned())?;

    println!(
        "Evaluation count: {}",
        EVALUATION_COUNT.load(Ordering::SeqCst) - LAST_EVALUATION_COUNT.load(Ordering::SeqCst)
    );

    LAST_EVALUATION_COUNT.store(EVALUATION_COUNT.load(Ordering::SeqCst), Ordering::SeqCst);

    value
}
