use std::cell::RefCell;
use std::rc::Rc;

use crate::builtin::std_lib;
use crate::expression::Expression;
use crate::run;

#[test]
fn add_two_integers() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(+ 1 2)").unwrap();

    assert_eq!(result, Expression::Integer(3));
}

#[test]
fn add_two_floats() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(+ 1.0 2.0)").unwrap();

    assert_eq!(result, Expression::Float(3.0));
}

#[test]
fn map_numbers_add() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(map (function '(x) '(+ x 1)) '(1 2 3))").unwrap();

    assert_eq!(
        result,
        Expression::List(vec![
            Expression::Integer(2),
            Expression::Integer(3),
            Expression::Integer(4)
        ])
    );
}

#[test]
fn fold() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(fold + 0 '(1 2 3))").unwrap();

    assert_eq!(result, Expression::Integer(6));
}

#[test]
fn filter() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(filter (function '(x) '(< x 3)) '(1 2 3 4 5))").unwrap();

    assert_eq!(
        result,
        Expression::List(vec![Expression::Integer(1), Expression::Integer(2)])
    );

    let result = run(&mut std, "(filter (function '(x) '(> x 3)) '(1 2 3 4 5))").unwrap();

    assert_eq!(
        result,
        Expression::List(vec![Expression::Integer(4), Expression::Integer(5)])
    );
}

#[test]
fn concat_strings() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, r#"(concat "hello" " " "world")"#).unwrap();

    assert_eq!(result, Expression::String("hello world".into()));
}

#[test]
fn create_add_xy_function() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(function '(x y) '(+ x y))").unwrap();

    assert_eq!(
        result,
        Expression::Function {
            arguments: vec![
                Expression::Symbol("x".into()),
                Expression::Symbol("y".into())
            ],
            body: Box::new(Expression::List(vec![
                Expression::Symbol("+".into()),
                Expression::Symbol("x".into()),
                Expression::Symbol("y".into())
            ]))
        }
    );
}

#[test]
fn add_xy_function() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "((function '(x y) '(+ x y)) 1 2)").unwrap();

    assert_eq!(result, Expression::Integer(3));
}

#[test]
fn fibonacci() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(
        &mut std,
        "(define 'fibonacci (function '(x)
            '(if (<= x 2) 1 (+ (fibonacci (- x 1)) (fibonacci (- x 2)))))
        )",
    )
    .unwrap();

    assert_eq!(result, Expression::Nil);

    let correct_results = vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144];

    for i in 1..=10 {
        let result = run(&mut std, &format!("(fibonacci {i})")).unwrap();
        assert_eq!(result, Expression::Integer(correct_results[i]));
    }
}

#[test]
fn let_chaining() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(
        &mut std,
        "(let 'x 1
            (let 'y 2
                (let 'z 3
                    (+ x y z)
                )
            )
        )",
    )
    .unwrap();

    assert_eq!(result, Expression::Integer(6));
}

#[test]
fn let_multi() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(
        &mut std,
        "(let* '(
            (x 1)
            (y 2)
            (z 3)
        ) (+ x y z))",
    )
    .unwrap();

    assert_eq!(result, Expression::Integer(6));
}

#[test]
fn partial_application_left() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(
        &mut std,
        "(define 'add-xyz (
            function '(x y z) '(+ x y z)
        ))",
    )
    .unwrap();

    assert_eq!(result, Expression::Nil);

    let result = run(&mut std, "add-xyz").unwrap();

    assert_eq!(
        result,
        Expression::Function {
            arguments: vec![
                Expression::Symbol("x".into()),
                Expression::Symbol("y".into()),
                Expression::Symbol("z".into())
            ],
            body: Box::new(Expression::List(vec![
                Expression::Symbol("+".into()),
                Expression::Symbol("x".into()),
                Expression::Symbol("y".into()),
                Expression::Symbol("z".into())
            ]))
        }
    );

    let result = run(&mut std, "(add-xyz 1)").unwrap();

    assert_eq!(
        result,
        Expression::Function {
            arguments: vec![
                Expression::Symbol("y".into()),
                Expression::Symbol("z".into())
            ],
            body: Box::new(Expression::List(vec![
                run(&mut std, "add-xyz").unwrap(),
                Expression::Integer(1),
            ]))
        }
    );

    let result = run(&mut std, "(add-xyz 1 2)").unwrap();

    assert_eq!(
        result,
        Expression::Function {
            arguments: vec![Expression::Symbol("z".into())],
            body: Box::new(Expression::List(vec![
                run(&mut std, "add-xyz").unwrap(),
                Expression::Integer(1),
                Expression::Integer(2),
            ]))
        }
    );

    let result = run(&mut std, "(add-xyz 1 2 3)").unwrap();

    assert_eq!(result, Expression::Integer(6));
}

// #[test]
// fn partial_application_right() {
//     let mut std = Rc::new(RefCell::new(std_lib()));

//     let result = run(
//         &mut std,
//         "(define add-xyz (
//             function '(x y z) '(+ x y z)
//         ))",
//     )
//     .unwrap();

//     assert_eq!(result, Expression::Nil);

//     let result = run(&mut std, "add-xyz").unwrap();

//     assert_eq!(
//         result,
//         Expression::Function {
//             arguments: vec![
//                 Expression::Symbol("x".into()),
//                 Expression::Symbol("y".into()),
//                 Expression::Symbol("z".into())
//             ],
//             body: Box::new(Expression::List(vec![
//                 Expression::Symbol("+".into()),
//                 Expression::Symbol("x".into()),
//                 Expression::Symbol("y".into()),
//                 Expression::Symbol("z".into())
//             ]))
//         }
//     );

//     let result = run(&mut std, "(add-xyz _ _ 3)").unwrap();

//     assert_eq!(
//         result,
//         Expression::Function {
//             arguments: vec![
//                 Expression::Symbol("x".into()),
//                 Expression::Symbol("y".into())
//             ],
//             body: Box::new(Expression::List(vec![
//                 Expression::Symbol("+".into()),
//                 Expression::Symbol("x".into()),
//                 Expression::Symbol("y".into()),
//                 Expression::Integer(3)
//             ]))
//         }
//     );

//     let result = run(&mut std, "(add-xyz _ 2 3)").unwrap();

//     assert_eq!(
//         result,
//         Expression::Function {
//             arguments: vec![Expression::Symbol("x".into())],
//             body: Box::new(Expression::List(vec![
//                 Expression::Symbol("+".into()),
//                 Expression::Symbol("x".into()),
//                 Expression::Integer(2),
//                 Expression::Integer(3)
//             ]))
//         }
//     );

//     let result = run(&mut std, "(add-xyz 1 2 3)").unwrap();

//     assert_eq!(result, Expression::Integer(6));
// }

#[test]
fn and_then() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(
        &mut std,
        "(and-then
            (define 'add (function '(x y) '(+ x y)))
            (add 1 2)
        )",
    )
    .unwrap();

    assert_eq!(result, Expression::Integer(3));
}

#[test]
fn range() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(range 1 5)").unwrap();

    assert_eq!(
        result,
        Expression::List(vec![
            Expression::Integer(1),
            Expression::Integer(2),
            Expression::Integer(3),
            Expression::Integer(4),
            Expression::Integer(5),
        ])
    );
}

#[test]
fn round() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(round 1.5)").unwrap();

    assert_eq!(result, Expression::Float(2.0));

    let result = run(&mut std, "(round 1.2)").unwrap();

    assert_eq!(result, Expression::Float(1.0));

    let result = run(&mut std, "(round 1.8)").unwrap();

    assert_eq!(result, Expression::Float(2.0));
}

#[test]
fn append() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(append 4 '(1 2 3))").unwrap();

    assert_eq!(
        result,
        Expression::List(vec![
            Expression::Integer(1),
            Expression::Integer(2),
            Expression::Integer(3),
            Expression::Integer(4),
        ])
    );
}

#[test]
fn prepend() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(prepend 1 '(2 3 4))").unwrap();

    assert_eq!(
        result,
        Expression::List(vec![
            Expression::Integer(1),
            Expression::Integer(2),
            Expression::Integer(3),
            Expression::Integer(4),
        ])
    );
}

#[test]
fn nth() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(nth 0 '(1 2 3))").unwrap();

    assert_eq!(result, Expression::Integer(1));

    let result = run(&mut std, "(nth 1 '(1 2 3))").unwrap();

    assert_eq!(result, Expression::Integer(2));

    let result = run(&mut std, "(nth 2 '(1 2 3))").unwrap();

    assert_eq!(result, Expression::Integer(3));
}

#[test]
fn slice() {
    let mut std = Rc::new(RefCell::new(std_lib()));

    let result = run(&mut std, "(slice 0 4 '(1 2 3))");

    assert!(result.is_err());

    let result = run(&mut std, "(slice -1 1 '(1 2 3))");

    assert!(result.is_err());

    let result = run(&mut std, "(slice 2 2 '(1 2 3))");

    assert!(result.is_err());

    let result = run(&mut std, "(slice 0 3 '(1 2 3))").unwrap();

    assert_eq!(
        result,
        Expression::List(vec![
            Expression::Integer(1),
            Expression::Integer(2),
            Expression::Integer(3)
        ])
    );

    let result = run(&mut std, "(slice 0 2 '(1 2 3))").unwrap();

    assert_eq!(
        result,
        Expression::List(vec![Expression::Integer(1), Expression::Integer(2),])
    );

    let result = run(&mut std, "(slice 0 1 '(1 2 3))").unwrap();

    assert_eq!(result, Expression::List(vec![Expression::Integer(1)]));

    let result = run(&mut std, "(slice 0 0 '(1 2 3))").unwrap();

    assert_eq!(result, Expression::List(vec![]));
}
