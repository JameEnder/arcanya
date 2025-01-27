use std::{cell::RefCell, rc::Rc, sync::atomic::Ordering};

use hashbrown::HashMap;

use crate::{env::Env, eval::*, expression::Expression, run};
use color_eyre::{eyre::eyre, Result};
fn has_float(list: &[Expression]) -> bool {
    list.iter().any(|x| matches!(x, Expression::Float(_)))
}

const PLUS: Expression = Expression::Builtin {
    name: "+",
    function: |env, list| {
        let evaluated = list
            .iter()
            .map(|e| eval_expression(env, e.clone()))
            .collect::<Result<Vec<Expression>>>()?;

        if has_float(&evaluated) {
            Ok(Expression::Float(
                evaluated
                    .iter()
                    .map(|x| x.as_f64())
                    .collect::<Result<Vec<f64>>>()?
                    .iter()
                    .sum(),
            ))
        } else {
            Ok(Expression::Integer(
                evaluated
                    .iter()
                    .map(|x| x.as_i64())
                    .collect::<Result<Vec<i64>>>()?
                    .iter()
                    .sum(),
            ))
        }
    },
};

const MINUS: Expression = Expression::Builtin {
    name: "-",
    function: |env, list| {
        let evaluated = list
            .iter()
            .flat_map(|e| eval_expression(env, e.clone()))
            .collect::<Vec<Expression>>();

        let first = evaluated[0].clone();

        if has_float(&evaluated) {
            Ok(Expression::Float(
                evaluated[1..]
                    .iter()
                    .flat_map(|x| x.as_f64())
                    .fold(first.as_f64()?, |acc, x| acc - x),
            ))
        } else {
            Ok(Expression::Integer(
                evaluated[1..]
                    .iter()
                    .flat_map(|x| x.as_i64())
                    .fold(first.as_i64()?, |acc, x| acc - x),
            ))
        }
    },
};

const MULTIPLY: Expression = Expression::Builtin {
    name: "*",
    function: |env, list| {
        let evaluated = list
            .iter()
            .flat_map(|e| eval_expression(env, e.clone()))
            .collect::<Vec<Expression>>();

        if has_float(&evaluated) {
            Ok(Expression::Float(
                evaluated
                    .iter()
                    .flat_map(|l| eval_expression(env, l.clone()).map(|v| v.as_f64()))
                    .filter_map(Result::ok)
                    .product(),
            ))
        } else {
            Ok(Expression::Integer(
                evaluated
                    .iter()
                    .flat_map(|l| eval_expression(env, l.clone()).map(|v| v.as_i64()))
                    .filter_map(Result::ok)
                    .product(),
            ))
        }
    },
};

const DIVIDE: Expression = Expression::Builtin {
    name: "/",
    function: |env, list| {
        let evaluated = list
            .iter()
            .flat_map(|e| eval_expression(env, e.clone()))
            .collect::<Vec<Expression>>();

        let first = evaluated[0].clone();

        if has_float(&evaluated) {
            Ok(Expression::Float(
                evaluated[1..]
                    .iter()
                    .flat_map(|l| eval_expression(env, l.clone()).map(|v| v.as_f64()))
                    .filter_map(Result::ok)
                    .fold(first.as_f64()?, |acc, x| acc / x),
            ))
        } else {
            Ok(Expression::Integer(
                evaluated[1..]
                    .iter()
                    .flat_map(|l| eval_expression(env, l.clone()).map(|v| v.as_i64()))
                    .filter_map(Result::ok)
                    .fold(first.as_i64()?, |acc, x| acc / x),
            ))
        }
    },
};

const MOD: Expression = Expression::Builtin {
    name: "%",
    function: |env, list| {
        let first = eval_expression(env, list[0].clone())?;
        let second = eval_expression(env, list[1].clone())?;

        Ok(Expression::Integer(first.as_i64()? % second.as_i64()?))
    },
};

const FUNCTION: Expression = Expression::Builtin {
    name: "function",
    function: |env, list| {
        let args = eval_expression(env, list[0].clone())?;
        let body = eval_expression(env, list[1].clone())?;

        Ok(Expression::Function {
            arguments: args.as_list()?,
            body: Box::new(body),
        })
    },
};

const IF: Expression = Expression::Builtin {
    name: "if",
    function: |env, list| {
        let condition = eval_expression(env, list[0].clone())?;

        let has_else = list.len() > 2;

        if condition.as_boolean()? {
            eval_expression(env, list[1].clone())
        } else if has_else {
            eval_expression(env, list[2].clone())
        } else {
            Ok(Expression::Nil)
        }
    },
};

const DEFINE: Expression = Expression::Builtin {
    name: "define",
    function: |env, list| {
        let name = list[0].clone();
        let value = list[1].clone();

        if let Expression::Symbol(_) = name {
            let evaluated = eval_expression(env, value)?;

            env.as_ref()
                .borrow_mut()
                .set_global(name.as_symbol_string()?, evaluated);
        }

        Ok(Expression::Nil)
    },
};

const LET: Expression = Expression::Builtin {
    name: "let",
    function: |env, list| {
        let name = list[0].clone();
        let value = list[1].clone();

        if let Expression::Symbol(_) = name {
            let evaluated = eval_expression(env, value)?;
            env.as_ref()
                .borrow_mut()
                .set_local(name.as_symbol_string()?, evaluated);
        }

        eval_expression(env, list[2].clone())
    },
};

const EQUAL: Expression = Expression::Builtin {
    name: "=",
    function: |env, list| {
        let evaluated = list
            .iter()
            .flat_map(|e| eval_expression(env, e.clone()))
            .collect::<Vec<Expression>>();

        Ok(evaluated[1..].iter().all(|x| evaluated[0] == *x).into())
    },
};

const GREATER: Expression = Expression::Builtin {
    name: ">",
    function: |env, list| {
        let evaluated = list
            .iter()
            .flat_map(|e| eval_expression(env, e.clone()))
            .collect::<Vec<Expression>>();

        Ok(if has_float(&evaluated) {
            evaluated[1..]
                .iter()
                .flat_map(|x| x.as_f64())
                .all(|x| evaluated[0].as_f64().unwrap() > x)
                .into()
        } else {
            evaluated[1..]
                .iter()
                .flat_map(|x| x.as_i64())
                .all(|x| evaluated[0].as_i64().unwrap() > x)
                .into()
        })
    },
};

const GREATER_EQUAL: Expression = Expression::Builtin {
    name: ">=",
    function: |env, list| {
        let evaluated = list
            .iter()
            .flat_map(|e| eval_expression(env, e.clone()))
            .collect::<Vec<Expression>>();

        Ok(if has_float(&evaluated) {
            evaluated[1..]
                .iter()
                .flat_map(|x| x.as_f64())
                .all(|x| evaluated[0].as_f64().unwrap() >= x)
                .into()
        } else {
            evaluated[1..]
                .iter()
                .flat_map(|x| x.as_i64())
                .all(|x| evaluated[0].as_i64().unwrap() >= x)
                .into()
        })
    },
};

const LESS: Expression = Expression::Builtin {
    name: "<",
    function: |env, list| {
        let evaluated = list
            .iter()
            .flat_map(|e| eval_expression(env, e.clone()))
            .collect::<Vec<Expression>>();

        Ok(if has_float(&evaluated) {
            evaluated[1..]
                .iter()
                .flat_map(|x| x.as_f64())
                .all(|x| evaluated[0].as_f64().unwrap() < x)
                .into()
        } else {
            evaluated[1..]
                .iter()
                .flat_map(|x| x.as_i64())
                .all(|x| evaluated[0].as_i64().unwrap() < x)
                .into()
        })
    },
};

const LESS_EQUAL: Expression = Expression::Builtin {
    name: "<=",
    function: |env, list| {
        let evaluated = list
            .iter()
            .flat_map(|e| eval_expression(env, e.clone()))
            .collect::<Vec<Expression>>();

        Ok(if has_float(&evaluated) {
            evaluated[1..]
                .iter()
                .flat_map(|x| x.as_f64())
                .all(|x| evaluated[0].as_f64().unwrap() <= x)
                .into()
        } else {
            evaluated[1..]
                .iter()
                .flat_map(|x| x.as_i64())
                .all(|x| evaluated[0].as_i64().unwrap() <= x)
                .into()
        })
    },
};

const AND: Expression = Expression::Builtin {
    name: "and",
    function: |env, list| {
        let evaluated = list
            .iter()
            .flat_map(|e| eval_expression(env, e.clone()))
            .collect::<Vec<Expression>>();

        Ok(evaluated[1..]
            .iter()
            .flat_map(|x| x.as_boolean())
            .fold(evaluated[0].as_boolean()?, |acc, x| acc && x)
            .into())
    },
};

const OR: Expression = Expression::Builtin {
    name: "or",
    function: |env, list| {
        let evaluated = list
            .iter()
            .flat_map(|e| eval_expression(env, e.clone()))
            .collect::<Vec<Expression>>();

        Ok(evaluated[1..]
            .iter()
            .flat_map(|x| x.as_boolean())
            .fold(evaluated[0].as_boolean()?, |acc, x| acc || x)
            .into())
    },
};

const LET_MANY: Expression = Expression::Builtin {
    name: "let*",
    function: |env, list| {
        let variables = eval_expression(env, list[0].clone())?.as_list()?;

        for variable in variables {
            let var = eval_expression(env, variable.clone())?.as_list()?;

            let name = var[0].clone();
            let value = var[1].clone();
            let evaluated = eval_expression(env, value)?;
            env.as_ref()
                .borrow_mut()
                .set_local(name.as_symbol_string()?, evaluated);
        }

        let result = eval_expression(env, list[1].clone())?;

        Ok(result)
    },
};

const EVAL: Expression = Expression::Builtin {
    name: "eval",
    function: eval_list,
};

const EVAL_LOG: Expression = Expression::Builtin {
    name: "eval-log",
    function: |env, list| {
        LAST_EVALUATION_COUNT.store(EVALUATION_COUNT.load(Ordering::SeqCst), Ordering::SeqCst);

        let result = eval_expression(env, Expression::List(list.to_vec()))?;

        println!(
            "Evaluation count: {}",
            EVALUATION_COUNT.load(Ordering::SeqCst) - LAST_EVALUATION_COUNT.load(Ordering::SeqCst)
        );

        LAST_EVALUATION_COUNT.store(EVALUATION_COUNT.load(Ordering::SeqCst), Ordering::SeqCst);

        Ok(result)
    },
};

const LAZY: Expression = Expression::Builtin {
    name: "lazy",
    function: |_env, list| Ok(list[0].clone()),
};

const TIME: Expression = Expression::Builtin {
    name: "time",
    function: |env, list| {
        let now = std::time::Instant::now();

        let result = eval_expression(env, list[0].clone());

        println!("Took: {} ms", now.elapsed().as_millis());

        result
    },
};

const CONCAT: Expression = Expression::Builtin {
    name: "concat",
    function: |env, list| {
        Ok(Expression::String(
            list.iter()
                .flat_map(|l| eval_expression(env, l.clone()).map(|v| v.as_string()))
                .filter_map(Result::ok)
                .collect::<Vec<String>>()
                .join(""),
        ))
    },
};

const RANGE: Expression = Expression::Builtin {
    name: "range",
    function: |env, list| {
        Ok(Expression::List(
            (eval_expression(env, list[0].clone())?.as_i64()?
                ..eval_expression(env, list[1].clone())?.as_i64()?)
                .map(Expression::Integer)
                .collect(),
        ))
    },
};

const FOR: Expression = Expression::Builtin {
    name: "for",
    function: |env, list| {
        let iterator_name = list[0].clone();
        let iterable = eval_expression(env, list[1].clone())?;
        let func = list[2].clone();

        for i in iterable.as_list()? {
            if let Expression::Builtin {
                name: _,
                function: actual,
            } = LET
            {
                actual(env, &[iterator_name.clone(), i, func.clone()])?;
            }
        }

        Ok(Expression::Nil)
    },
};

const FOR_I: Expression = Expression::Builtin {
    name: "for-i",
    function: |env, list| {
        let iterator_name = list[0].clone().as_list()?[0].clone();
        let iterator_value = list[0].clone().as_list()?[1].clone();
        let condition = list[1].clone();
        let after = list[2].clone();
        let f = list[3].clone();
        let mut current = iterator_value.clone();

        if let Expression::Builtin {
            name: _,
            function: actual,
        } = LET
        {
            loop {
                if !actual(
                    env,
                    &[iterator_name.clone(), current.clone(), condition.clone()],
                )?
                .as_boolean()?
                {
                    break;
                }
                actual(env, &[iterator_name.clone(), current.clone(), f.clone()])?;
                current = actual(env, &[iterator_name.clone(), current, after.clone()])?;
            }
        }

        Ok(Expression::Nil)
    },
};

const MAP: Expression = Expression::Builtin {
    name: "map",
    function: |env, list| {
        let func = eval_expression(env, list[0].clone())?;

        Ok(Expression::List(
            eval_expression(env, list[1].clone())?
                .as_list()?
                .iter()
                .flat_map(|x| eval_list(env, &[func.clone(), x.clone()]))
                .collect::<Vec<Expression>>(),
        ))
    },
};

const FOLD: Expression = Expression::Builtin {
    name: "fold",
    function: |env, list| {
        let func = eval_expression(env, list[0].clone())?;
        let initial = eval_expression(env, list[1].clone())?;

        eval_expression(env, list[2].clone())?
            .as_list()?
            .iter()
            .try_fold(initial, |acc, x| {
                eval_list(env, &[func.clone(), acc.clone(), x.clone()])
            })
    },
};

const FILTER: Expression = Expression::Builtin {
    name: "filter",
    function: |env, list| {
        let func = eval_expression(env, list[0].clone())?;

        Ok(Expression::List(
            eval_expression(env, list[1].clone())?
                .as_list()?
                .iter()
                .filter(|&x| {
                    eval_list(env, &[func.clone(), x.clone()])
                        .unwrap()
                        .as_boolean()
                        .unwrap()
                })
                .cloned()
                .collect::<Vec<Expression>>(),
        ))
    },
};

const PRINT: Expression = Expression::Builtin {
    name: "print",
    function: |env, list| {
        println!("{}", eval_expression(env, list[0].clone())?);

        Ok(Expression::Nil)
    },
};

const TO_STRING: Expression = Expression::Builtin {
    name: "to-string",
    function: |env, list| {
        Ok(Expression::String(
            eval_expression(env, list[0].clone())?.to_string(),
        ))
    },
};

const TO_SYMBOL: Expression = Expression::Builtin {
    name: "to-symbol",
    function: |env, list| {
        Ok(Expression::Symbol(
            eval_expression(env, list[0].clone())?.to_string(),
        ))
    },
};

const AND_THEN: Expression = Expression::Builtin {
    name: "and-then",
    function: |env, list| {
        eval_expression(env, list[0].clone())?;

        eval_expression(env, list[1].clone())
    },
};

const EXISTS: Expression = Expression::Builtin {
    name: "exists",
    function: |env, list| {
        let evaluated = eval_expression(env, list[0].clone())?.as_symbol_string()?;

        Ok(env.as_ref().borrow().get(evaluated).is_some().into())
    },
};

const CONCAT_SYMBOL: Expression = Expression::Builtin {
    name: "concat-symbol",
    function: |_env, list| {
        Ok(Expression::Symbol(
            list.iter()
                .flat_map(|l| l.as_symbol_string())
                .collect::<Vec<String>>()
                .join(""),
        ))
    },
};

const WEB_SERVER: Expression = Expression::Builtin {
    name: "web-server",
    function: |env, list| {
        let port = eval_expression(env, list[0].clone())?;
        let routes = eval_expression(env, list[1].clone())?;

        let mut router: HashMap<String, Expression> = HashMap::new();

        for route in routes.as_list()? {
            let r = eval_expression(env, route.clone())?.as_list()?;

            router.insert(
                eval_expression(env, r[0].clone())?.as_string()?,
                eval_expression(env, r[1].clone())?,
            );
        }

        let server = tiny_http::Server::http(format!("127.0.0.1:{}", port.as_i64()?)).unwrap();

        for request in server.incoming_requests() {
            let response =
                tiny_http::Response::from_string(if let Some(expr) = router.get(request.url()) {
                    eval_expression(env, expr.clone())?.as_string()?
                } else {
                    "404".to_string()
                });

            // request.respond(response.with_header(tiny_http::Header {
            //     field: "Content-Type".parse().unwrap(),
            //     value: "text/html; charset=utf8".parse().unwrap(),
            // }))?;

            request.respond(response)?;
        }

        Ok(Expression::Nil)
    },
};

const APPEND: Expression = Expression::Builtin {
    name: "append",
    function: |env, list| {
        let mut new_list = eval_expression(env, list[1].clone())?.as_list()?.clone();

        new_list.push(eval_expression(env, list[0].clone())?);

        Ok(Expression::List(new_list))
    },
};

const PREPEND: Expression = Expression::Builtin {
    name: "prepend",
    function: |env, list| {
        let mut new_list = vec![eval_expression(env, list[0].clone())?];

        new_list.extend(eval_expression(env, list[1].clone())?.as_list()?);

        Ok(Expression::List(new_list))
    },
};

const ROUND: Expression = Expression::Builtin {
    name: "round",
    function: |env, list| {
        let evaluated = eval_expression(env, list[0].clone())?;

        Ok(Expression::Float(evaluated.as_f64()?.round()))
    },
};

const INDEX: Expression = Expression::Builtin {
    name: "index",
    function: |env, list| {
        let index = eval_expression(env, list[0].clone())?.as_i64()? as usize;
        let l = eval_expression(env, list[1].clone())?.as_list()?;

        Ok(l[index].clone())
    },
};

const SLICE: Expression = Expression::Builtin {
    name: "slice",
    function: |env, list| {
        let start = eval_expression(env, list[0].clone())?.as_i64()?;
        let end = eval_expression(env, list[1].clone())?.as_i64()?;
        let l = eval_expression(env, list[2].clone())?.as_list()?;

        if start < 0 {
            return Err(eyre!("Index below zero: {start}"));
        }

        if start + end > l.len() as i64 {
            return Err(eyre!("Out of bounds: {start} + {end} > {}", l.len()));
        }

        Ok(Expression::List(l[start as usize..end as usize].to_vec()))
    },
};

const REVERSE: Expression = Expression::Builtin {
    name: "reverse",
    function: |env, list| {
        let l = eval_expression(env, list[0].clone())?.as_list()?;

        Ok(Expression::List(l.iter().rev().cloned().collect()))
    },
};

const LENGTH: Expression = Expression::Builtin {
    name: "length",
    function: |env, list| {
        let evaluated = eval_expression(env, list[0].clone())?;

        Ok(Expression::Integer(match evaluated {
            Expression::List(l) => l.len() as i64,
            Expression::String(s) => s.len() as i64,
            _ => Err(eyre!("Doesn't have length: {evaluated}"))?,
        }))
    },
};

const TANGLE: Expression = Expression::Builtin {
    name: "tangle",
    function: |env, list| {
        let with = eval_expression(env, list[0].clone())?;
        let l = eval_expression(env, list[1].clone())?.as_list()?;
        let mut new_list = vec![];

        for i in 0..l.len() {
            new_list.push(l[i].clone());

            if i != l.len() - 1 {
                new_list.push(with.clone());
            }
        }

        Ok(Expression::List(new_list))
    },
};

const TYPE: Expression = Expression::Builtin {
    name: "type",
    function: |env, list| {
        let evaluated = eval_expression(env, list[0].clone())?;

        Ok(Expression::String(evaluated.as_type_string()))
    },
};

const READ: Expression = Expression::Builtin {
    name: "read",
    function: |env, list| {
        let file_name = eval_expression(env, list[0].clone())?.as_string()?;

        let content = std::fs::read_to_string(file_name)?;

        Ok(Expression::String(content))
    },
};

const WRITE: Expression = Expression::Builtin {
    name: "write",
    function: |env, list| {
        let file_name = eval_expression(env, list[0].clone())?;
        let content = eval_expression(env, list[1].clone())?;

        Ok(std::fs::write(file_name.as_string()?, content.as_string()?)
            .is_ok()
            .into())
    },
};

const SPLIT: Expression = Expression::Builtin {
    name: "split",
    function: |env, list| {
        let by = eval_expression(env, list[0].clone())?.as_string()?;
        let content = eval_expression(env, list[1].clone())?.as_string()?;

        Ok(Expression::List(
            content
                .split(&by)
                .map(|s| Expression::String(s.to_string()))
                .collect(),
        ))
    },
};

const ZIP: Expression = Expression::Builtin {
    name: "zip",
    function: |env, list| {
        let a = eval_expression(env, list[0].clone())?.as_list()?;
        let b = eval_expression(env, list[1].clone())?.as_list()?;

        Ok(Expression::List(
            a.iter()
                .cloned()
                .zip(b)
                .map(|(x, y)| Expression::List(vec![x, y]))
                .collect(),
        ))
    },
};

const ZIP_WITH: Expression = Expression::Builtin {
    name: "zip-with",
    function: |env, list| {
        let with = eval_expression(env, list[0].clone())?;
        let a = eval_expression(env, list[1].clone())?.as_list()?;
        let b = eval_expression(env, list[2].clone())?.as_list()?;

        Ok(Expression::List(
            a.iter()
                .zip(b)
                .flat_map(|(x, y)| eval_list(env, &[with.clone(), x.clone(), y.clone()]))
                .collect(),
        ))
    },
};

const IMPORT: Expression = Expression::Builtin {
    name: "import",
    function: |env, list| {
        let path = list[0].clone().as_string()?;

        let content = std::fs::read_to_string(path)?;

        let mut module_env = Rc::new(RefCell::new(Env::new(Some(Rc::new(RefCell::new(
            std_lib(),
        ))))));

        run(&mut module_env, &content)?;

        for (symbol, value) in module_env
            .borrow_mut()
            .get("__EXPORTED".to_string())
            .unwrap()
            .as_table()?
        {
            if let Expression::Table(ref mut table) = env
                .borrow_mut()
                .get_mut_local("__IMPORTED".to_string())
                .unwrap()
            {
                table.insert(symbol, value);
            }
        }

        Ok(Expression::Nil)
    },
};

const EXPORT: Expression = Expression::Builtin {
    name: "export",
    function: |env, list| {
        let symbol = list[0].clone();

        let value = env.borrow_mut().get(symbol.as_symbol_string()?).unwrap();

        if let Expression::Table(ref mut table) = env
            .borrow_mut()
            .get_mut_local("__EXPORTED".to_string())
            .unwrap()
        {
            table.insert(symbol.as_symbol_string()?, value);
        }

        Ok(Expression::Nil)
    },
};

const MODULE: Expression = Expression::Builtin {
    name: "module",
    function: |env, _list| {
        Ok(Expression::Table(HashMap::from([
            (
                "imported".to_string(),
                env.borrow().get("__IMPORTED".to_string()).unwrap(),
            ),
            (
                "exported".to_string(),
                env.borrow().get("__EXPORTED".to_string()).unwrap(),
            ),
        ])))
    },
};

const QUOTE: Expression = Expression::Builtin {
    name: "quote",
    function: |_env, list| Ok(list[0].clone()),
};

const ENV: Expression = Expression::Builtin {
    name: "env",
    function: |env, _list| {
        Ok(Expression::List(
            env.borrow().local.values().cloned().collect(),
        ))
    },
};

const APPLY: Expression = Expression::Builtin {
    name: "apply",
    function: |env, list| {
        let f = eval_expression(env, list[0].clone())?;
        let args = eval_expression(env, list[1].clone())?.as_list()?;
        let args: Vec<Expression> = vec![f.clone()].into_iter().chain(args).collect();

        Ok(eval_list(env, &args)?)
    },
};

pub fn std_lib() -> Env {
    let std: &[Expression] = &[
        PLUS,
        MINUS,
        MULTIPLY,
        DIVIDE,
        MOD,
        EQUAL,
        GREATER,
        GREATER_EQUAL,
        LESS,
        LESS_EQUAL,
        AND,
        OR,
        FUNCTION,
        IF,
        DEFINE,
        LET,
        LET_MANY,
        EVAL,
        EVAL_LOG,
        LAZY,
        TIME,
        CONCAT,
        RANGE,
        FOR,
        FOR_I,
        MAP,
        FOLD,
        FILTER,
        PRINT,
        ROUND,
        WEB_SERVER,
        TO_STRING,
        TO_SYMBOL,
        AND_THEN,
        EXISTS,
        CONCAT_SYMBOL,
        APPEND,
        PREPEND,
        INDEX,
        SLICE,
        REVERSE,
        LENGTH,
        TANGLE,
        TYPE,
        SPLIT,
        READ,
        WRITE,
        ZIP,
        ZIP_WITH,
        IMPORT,
        EXPORT,
        MODULE,
        QUOTE,
        ENV,
        APPLY,
    ];

    /*
        Create namespaces

        So for example list namespace, which would be used like so

        (list/map (function '(x) '(+ x 1)) '(1 2 3))
        (list/fold (function '(acc x) '(+ acc x)) 0 '(1 2 3))

        You could also call (use 'list) to import the list namespace
        and then use it like so

        (use 'list)
        (map (function '(x) '(+ x 1)) '(1 2 3))
        (fold '+ 0 '(1 2 3))
    */

    let mut env = Env {
        local: HashMap::from_iter(std.iter().map(|pair| {
            if let Expression::Builtin { name, function: _ } = pair {
                (name.to_string(), pair.clone())
            } else {
                unreachable!()
            }
        })),
        parent: None,
    };

    env.set_global("t".to_string(), crate::expression::TRUE.clone());

    env
}
