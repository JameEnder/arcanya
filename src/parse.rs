use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, multispace1},
    combinator::{cut, map, map_res, opt, recognize},
    multi::{many0_count, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    IResult,
};

use crate::expression::Expression;

fn parse_bool(input: &str) -> IResult<&str, Expression> {
    map(
        alt((tag("true"), tag("false"), tag("#t"), tag("#f"), tag("nil"))),
        |s: &str| match s {
            "true" | "#t" => true.into(),
            "false" | "#f" | "nil" => false.into(),
            _ => unreachable!(),
        },
    )(input)
}

fn parse_void(input: &str) -> IResult<&str, Expression> {
    map(tag("void"), |_| Expression::Nil)(input)
}

fn parse_symbol(input: &str) -> IResult<&str, Expression> {
    map(
        recognize(tuple((
            alt((
                alpha1,
                tag("_"),
                tag("+"),
                tag("-"),
                tag("/"),
                tag("%"),
                tag("*"),
                tag("="),
                tag(">"),
                tag("<"),
            )),
            many0_count(alt((
                alphanumeric1,
                tag("_"),
                tag("+"),
                tag("-"),
                tag("/"),
                tag("%"),
                tag("*"),
                tag("="),
                tag(">"),
                tag("<"),
            ))),
        ))),
        |s: &str| Expression::Symbol(s.to_string()),
    )(input)
}

pub fn parse_float(input: &str) -> IResult<&str, Expression> {
    alt((
        map(
            map_res(separated_pair(digit1, tag("."), opt(digit1)), |(a, b)| {
                format!("{}.{}", a, b.unwrap_or("0")).parse::<f64>()
            }),
            |f: f64| Expression::Float(f),
        ),
        map(
            map_res(
                preceded(tag("-"), separated_pair(digit1, tag("."), opt(digit1))),
                |(a, b)| format!("{}.{}", a, b.unwrap_or("0")).parse::<f64>(),
            ),
            |f: f64| Expression::Float(-f),
        ),
    ))(input)
}

pub fn parse_integer(input: &str) -> IResult<&str, Expression> {
    alt((
        map(
            map_res(digit1, |digit_str: &str| digit_str.parse::<i64>()),
            |i: i64| Expression::Integer(i),
        ),
        map(
            map_res(preceded(tag("-"), digit1), |digit_str: &str| {
                digit_str.parse::<i64>()
            }),
            |i: i64| Expression::Integer(-i),
        ),
    ))(input)
}

pub fn parse_string(input: &str) -> IResult<&str, Expression> {
    alt((
        map(delimited(char('"'), is_not("\""), char('"')), |s: &str| {
            Expression::String(s.to_string())
        }),
        map(pair(tag("\""), tag("\"")), |_| {
            Expression::String("".to_string())
        }),
    ))(input)
}

pub fn parse_list(input: &str) -> IResult<&str, Expression> {
    delimited(
        char('('),
        map(
            separated_list0(multispace1, parse_expression),
            Expression::List,
        ),
        cut(preceded(multispace0, char(')'))),
    )(input)
}

pub fn parse_list_quoted(input: &str) -> IResult<&str, Expression> {
    map(preceded(char('\''), parse_list), |list| {
        Expression::List(vec![Expression::Symbol("quote".to_string()), list])
    })(input)
}

pub fn parse_list_square(input: &str) -> IResult<&str, Expression> {
    delimited(
        char('['),
        map(separated_list0(multispace1, parse_expression), |exprs| {
            Expression::List(vec![
                Expression::Symbol("quote".to_string()),
                Expression::List(exprs),
            ])
        }),
        cut(preceded(multispace0, char(']'))),
    )(input)
}

pub fn parse_expression(input: &str) -> IResult<&str, Expression> {
    preceded(
        multispace0,
        alt((
            parse_float,
            parse_integer,
            parse_bool,
            parse_void,
            parse_symbol,
            parse_string,
            parse_list,
            parse_list_quoted,
            parse_list_square,
        )),
    )(input)
}
