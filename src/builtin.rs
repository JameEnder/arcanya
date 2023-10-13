use hashbrown::HashMap;

use crate::{eval::*, Env, Expression};
use anyhow::{anyhow, Result};

fn has_float(list: &[Expression]) -> bool {
    list.iter().any(|x| matches!(x, Expression::Float(_)))
}

const PLUS: Expression = Expression::Builtin(|env, list| {
    let evaluated = list
        .iter()
        .flat_map(|e| eval_expression(env, e.clone()))
        .collect::<Vec<Expression>>();

    if has_float(&evaluated) {
        Ok(Expression::Float(
            evaluated.iter().flat_map(|x| x.as_f64()).sum(),
        ))
    } else {
        Ok(Expression::Integer(
            evaluated.iter().flat_map(|x| x.as_i64()).sum(),
        ))
    }
});

const MINUS: Expression = Expression::Builtin(|env, list| {
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
});

const MULTIPLY: Expression = Expression::Builtin(|env, list| {
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
});

const DIVIDE: Expression = Expression::Builtin(|env, list| {
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
});

const FUNCTION: Expression = Expression::Builtin(|_env, list| {
    let args = list[0].clone();
    let body = list[1].clone();

    if let Expression::List(a) = args {
        Ok(Expression::Function {
            arguments: a,
            body: Box::new(body),
        })
    } else {
        Err(anyhow!("Incorrect args"))
    }
});

const IF: Expression = Expression::Builtin(|env, list| {
    let condition = eval_expression(env, list[0].clone())?;
    let then = list[1].clone();

    let has_else = list.len() > 2;

    if let Expression::Boolean(a) = condition {
        if a {
            eval_expression(env, then)
        } else if has_else {
            eval_expression(env, list[2].clone())
        } else {
            Ok(Expression::Void)
        }
    } else {
        Err(anyhow!("Not a boolean condition"))
    }
});

const DEFINE: Expression = Expression::Builtin(|env, list| {
    let name = list[0].clone();
    let value = list[1].clone();

    if let Expression::Symbol(_) = name {
        let evaluated = eval_expression(env, value)?;

        env.as_ref()
            .borrow_mut()
            .set_global(name.as_symbol_string()?, evaluated);
    }

    Ok(Expression::Void)
});

const LET: Expression = Expression::Builtin(|env, list| {
    let name = list[0].clone();
    let value = list[1].clone();

    if let Expression::Symbol(_) = name {
        let evaluated = eval_expression(env, value)?;
        env.as_ref()
            .borrow_mut()
            .set_local(name.as_symbol_string()?, evaluated);
    }

    eval_expression(env, list[2].clone())
});

const EQUAL: Expression = Expression::Builtin(|env, list| {
    let evaluated = list
        .iter()
        .flat_map(|e| eval_expression(env, e.clone()))
        .collect::<Vec<Expression>>();

    Ok(Expression::Boolean(
        evaluated[1..].iter().all(|x| evaluated[0] == *x),
    ))
});

const GREATER: Expression = Expression::Builtin(|env, list| {
    let evaluated = list
        .iter()
        .flat_map(|e| eval_expression(env, e.clone()))
        .collect::<Vec<Expression>>();

    Ok(Expression::Boolean(if has_float(&evaluated) {
        evaluated[1..]
            .iter()
            .flat_map(|x| x.as_f64())
            .all(|x| evaluated[0].as_f64().unwrap() > x)
    } else {
        evaluated[1..]
            .iter()
            .flat_map(|x| x.as_i64())
            .all(|x| evaluated[0].as_i64().unwrap() > x)
    }))
});

const GREATER_EQUAL: Expression = Expression::Builtin(|env, list| {
    let evaluated = list
        .iter()
        .flat_map(|e| eval_expression(env, e.clone()))
        .collect::<Vec<Expression>>();

    Ok(Expression::Boolean(if has_float(&evaluated) {
        evaluated[1..]
            .iter()
            .flat_map(|x| x.as_f64())
            .all(|x| evaluated[0].as_f64().unwrap() >= x)
    } else {
        evaluated[1..]
            .iter()
            .flat_map(|x| x.as_i64())
            .all(|x| evaluated[0].as_i64().unwrap() >= x)
    }))
});

const LESS: Expression = Expression::Builtin(|env, list| {
    let evaluated = list
        .iter()
        .flat_map(|e| eval_expression(env, e.clone()))
        .collect::<Vec<Expression>>();

    Ok(Expression::Boolean(if has_float(&evaluated) {
        evaluated[1..]
            .iter()
            .flat_map(|x| x.as_f64())
            .all(|x| evaluated[0].as_f64().unwrap() < x)
    } else {
        evaluated[1..]
            .iter()
            .flat_map(|x| x.as_i64())
            .all(|x| evaluated[0].as_i64().unwrap() < x)
    }))
});

const LESS_EQUAL: Expression = Expression::Builtin(|env, list| {
    let evaluated = list
        .iter()
        .flat_map(|e| eval_expression(env, e.clone()))
        .collect::<Vec<Expression>>();

    Ok(Expression::Boolean(if has_float(&evaluated) {
        evaluated[1..]
            .iter()
            .flat_map(|x| x.as_f64())
            .all(|x| evaluated[0].as_f64().unwrap() <= x)
    } else {
        evaluated[1..]
            .iter()
            .flat_map(|x| x.as_i64())
            .all(|x| evaluated[0].as_i64().unwrap() <= x)
    }))
});

const AND: Expression = Expression::Builtin(|env, list| {
    let evaluated = list
        .iter()
        .flat_map(|e| eval_expression(env, e.clone()))
        .collect::<Vec<Expression>>();

    Ok(Expression::Boolean(
        evaluated[1..]
            .iter()
            .flat_map(|x| x.as_boolean())
            .fold(evaluated[0].as_boolean()?, |acc, x| acc && x),
    ))
});

const OR: Expression = Expression::Builtin(|env, list| {
    let evaluated = list
        .iter()
        .flat_map(|e| eval_expression(env, e.clone()))
        .collect::<Vec<Expression>>();

    Ok(Expression::Boolean(
        evaluated[1..]
            .iter()
            .flat_map(|x| x.as_boolean())
            .fold(evaluated[0].as_boolean()?, |acc, x| acc || x),
    ))
});

const LET_MANY: Expression = Expression::Builtin(|env, list| {
    if let Expression::List(variables) = &list[0] {
        for variable in variables {
            if let Expression::List(var) = variable {
                let name = var[0].clone();
                let value = var[1].clone();
                let evaluated = eval_expression(env, value)?;
                env.as_ref()
                    .borrow_mut()
                    .set_local(name.as_symbol_string()?, evaluated);
            }
        }
    }

    let result = eval_expression(env, list[1].clone())?;

    Ok(result)
});

const EVAL: Expression = Expression::Builtin(eval_call);

const LAZY: Expression = Expression::Builtin(|_env, list| Ok(list[0].clone()));

const TIME: Expression = Expression::Builtin(|env, list| {
    let now = std::time::Instant::now();

    let result = eval_expression(env, list[0].clone());

    println!("Took: {} ms", now.elapsed().as_millis());

    result
});

const CONCAT: Expression = Expression::Builtin(|env, list| {
    Ok(Expression::String(
        list.iter()
            .flat_map(|l| eval_expression(env, l.clone()).map(|v| v.as_string()))
            .filter_map(Result::ok)
            .collect::<Vec<String>>()
            .join(""),
    ))
});

const RANGE: Expression = Expression::Builtin(|env, list| {
    Ok(Expression::List(
        (eval_expression(env, list[0].clone())?.as_i64()?
            ..eval_expression(env, list[1].clone())?.as_i64()?)
            .map(Expression::Integer)
            .collect(),
    ))
});

const FOR: Expression = Expression::Builtin(|env, list| {
    let iterator_name = list[0].clone();
    let iterable = eval_expression(env, list[1].clone())?;
    let func = list[2].clone();

    for i in iterable.as_list()? {
        if let Expression::Builtin(actual) = LET {
            actual(env, &[iterator_name.clone(), i, func.clone()])?;
        }
    }

    Ok(Expression::Void)
});

const MAP: Expression = Expression::Builtin(|env, list| {
    let func = eval_expression(env, list[0].clone())?;

    Ok(Expression::List(
        eval_expression(env, list[1].clone())?
            .as_list()?
            .iter()
            .flat_map(|x| eval_call(env, &[func.clone(), x.clone()]))
            .collect::<Vec<Expression>>(),
    ))
});

const FOLD: Expression = Expression::Builtin(|env, list| {
    let func = eval_expression(env, list[0].clone())?;
    let initial = eval_expression(env, list[1].clone())?;

    eval_expression(env, list[2].clone())?
        .as_list()?
        .iter()
        .try_fold(initial, |acc, x| {
            eval_call(env, &[func.clone(), acc.clone(), x.clone()])
        })
});

const FILTER: Expression = Expression::Builtin(|env, list| {
    let func = eval_expression(env, list[0].clone())?;

    Ok(Expression::List(
        eval_expression(env, list[1].clone())?
            .as_list()?
            .iter()
            .filter(|&x| {
                eval_call(env, &[func.clone(), x.clone()])
                    .unwrap()
                    .as_boolean()
                    .unwrap()
            })
            .cloned()
            .collect::<Vec<Expression>>(),
    ))
});

const PRINT: Expression = Expression::Builtin(|env, list| {
    println!("{}", eval_expression(env, list[0].clone())?);

    Ok(Expression::Void)
});

const TO_STRING: Expression = Expression::Builtin(|env, list| {
    Ok(Expression::String(
        eval_expression(env, list[0].clone())?.to_string(),
    ))
});

const TO_SYMBOL: Expression = Expression::Builtin(|env, list| {
    Ok(Expression::Symbol(
        eval_expression(env, list[0].clone())?.to_string(),
    ))
});

const AND_THEN: Expression = Expression::Builtin(|env, list| {
    eval_expression(env, list[0].clone())?;

    eval_expression(env, list[1].clone())
});

const EXISTS: Expression = Expression::Builtin(|env, list| {
    let evaluated = eval_expression(env, list[0].clone())?.as_symbol_string()?;

    Ok(Expression::Boolean(
        env.as_ref().borrow().get(evaluated).is_some(),
    ))
});

const CONCAT_SYMBOL: Expression = Expression::Builtin(|_env, list| {
    Ok(Expression::Symbol(
        list.iter()
            .flat_map(|l| l.as_symbol_string())
            .collect::<Vec<String>>()
            .join(""),
    ))
});

const WEB_SERVER: Expression = Expression::Builtin(|env, list| {
    let port = eval_expression(env, list[0].clone())?;
    let routes = list[1].clone();

    let mut router: HashMap<String, Expression> = HashMap::new();

    for route in routes.as_list()? {
        let r = route.as_list()?;

        router.insert(
            eval_expression(env, r[0].clone())?.as_string()?,
            r[1].clone(),
        );
    }

    let server = tiny_http::Server::http(format!("127.0.0.1:{port}")).unwrap();

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

    Ok(Expression::Void)
});

const APPEND: Expression = Expression::Builtin(|env, list| {
    let mut new_list = eval_expression(env, list[0].clone())?.as_list()?.clone();

    new_list.push(eval_expression(env, list[1].clone())?);

    Ok(Expression::List(new_list))
});

const PREPEND: Expression = Expression::Builtin(|env, list| {
    let mut new_list = vec![eval_expression(env, list[1].clone())?];

    new_list.extend(eval_expression(env, list[0].clone())?.as_list()?);

    Ok(Expression::List(new_list))
});

const ROUND: Expression = Expression::Builtin(|env, list| {
    let evaluated = eval_expression(env, list[0].clone())?;

    Ok(Expression::Float(evaluated.as_f64()?.round()))
});

const INDEX: Expression = Expression::Builtin(|env, list| {
    let index = eval_expression(env, list[0].clone())?.as_i64()? as usize;
    let l = eval_expression(env, list[1].clone())?.as_list()?;

    Ok(l[index].clone())
});

const SLICE: Expression = Expression::Builtin(|env, list| {
    let start = eval_expression(env, list[0].clone())?.as_i64()?;
    let end = eval_expression(env, list[1].clone())?.as_i64()?;
    let l = eval_expression(env, list[2].clone())?.as_list()?;

    if start < 0 {
        return Err(anyhow!("index < 0 doesnt exist"));
    }

    if start + end > l.len() as i64 {
        return Err(anyhow!("out of bounds"));
    }

    Ok(Expression::List(l[start as usize..end as usize].to_vec()))
});

const LENGTH: Expression = Expression::Builtin(|env, list| {
    let evaluated = eval_expression(env, list[0].clone())?;

    Ok(Expression::Integer(match evaluated {
        Expression::List(l) => l.len() as i64,
        Expression::String(s) => s.len() as i64,
        _ => Err(anyhow!("doesnt have length"))?,
    }))
});

const TANGLE: Expression = Expression::Builtin(|env, list| {
    let with = eval_expression(env, list[0].clone())?;
    let l = list[1].as_list()?;
    let mut new_list = vec![];

    for i in 0..l.len() {
        new_list.push(l[i].clone());

        if i != l.len() - 1 {
            new_list.push(with.clone());
        }
    }

    Ok(Expression::List(new_list))
});

const TYPE: Expression = Expression::Builtin(|env, list| {
    let evaluated = eval_expression(env, list[0].clone())?;

    Ok(Expression::String(evaluated.as_type_string()))
});

const READ: Expression = Expression::Builtin(|env, list| {
    let file_name = eval_expression(env, list[0].clone())?.as_string()?;

    let content = std::fs::read_to_string(file_name)?;

    Ok(Expression::String(content))
});

const WRITE: Expression = Expression::Builtin(|env, list| {
    let file_name = eval_expression(env, list[0].clone())?;

    let content = eval_expression(env, list[1].clone())?;

    Ok(Expression::Boolean(
        std::fs::write(file_name.as_string()?, content.as_string()?).is_ok(),
    ))
});

const SPLIT: Expression = Expression::Builtin(|env, list| {
    let by = eval_expression(env, list[0].clone())?.as_string()?;
    let content = eval_expression(env, list[1].clone())?.as_string()?;

    Ok(Expression::List(
        content
            .split(&by)
            .map(|s| Expression::String(s.to_string()))
            .collect(),
    ))
});

const ZIP: Expression = Expression::Builtin(|env, list| {
    let a = eval_expression(env, list[0].clone())?.as_list()?;
    let b = eval_expression(env, list[1].clone())?.as_list()?;

    Ok(Expression::List(
        a.iter()
            .cloned()
            .zip(b)
            .map(|(x, y)| Expression::List(vec![x, y]))
            .collect(),
    ))
});

const ZIP_WITH: Expression = Expression::Builtin(|env, list| {
    let with = eval_expression(env, list[0].clone())?;
    let a = eval_expression(env, list[1].clone())?.as_list()?;
    let b = eval_expression(env, list[2].clone())?.as_list()?;

    Ok(Expression::List(
        a.iter()
            .zip(b)
            .map(|(x, y)| eval_call(env, &[with.clone(), x.clone(), y.clone()]))
            .flatten()
            .collect(),
    ))
});

pub fn std_lib() -> Env {
    let std: &[(&str, Expression)] = &[
        ("+", PLUS),
        ("-", MINUS),
        ("*", MULTIPLY),
        ("/", DIVIDE),
        ("=", EQUAL),
        (">", GREATER),
        (">=", GREATER_EQUAL),
        ("<", LESS),
        ("<=", LESS_EQUAL),
        ("and", AND),
        ("or", OR),
        ("function", FUNCTION),
        ("if", IF),
        ("define", DEFINE),
        ("let", LET),
        ("let*", LET_MANY),
        ("eval", EVAL),
        ("lazy", LAZY),
        ("time", TIME),
        ("concat", CONCAT),
        ("range", RANGE),
        ("for", FOR),
        ("map", MAP),
        ("fold", FOLD),
        ("reduce", FOLD),
        ("filter", FILTER),
        ("print", PRINT),
        ("round", ROUND),
        ("web-server", WEB_SERVER),
        ("to-string", TO_STRING),
        ("to-symbol", TO_SYMBOL),
        ("and-then", AND_THEN),
        ("exists", EXISTS),
        ("concat-symbol", CONCAT_SYMBOL),
        ("append", APPEND),
        ("prepend", PREPEND),
        ("index", INDEX),
        ("slice", SLICE),
        ("length", LENGTH),
        ("tangle", TANGLE),
        ("type", TYPE),
        ("split", SPLIT),
        ("read", READ),
        ("write", WRITE),
        ("zip", ZIP),
        ("zip-with", ZIP_WITH),
    ];

    let env = Env {
        local: HashMap::from_iter(std.iter().map(|pair| (pair.0.to_string(), pair.1.clone()))),
        parent: None,
    };

    env
}
