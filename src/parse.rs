use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, multispace1},
    combinator::{cut, map, map_res, recognize},
    multi::{many0_count, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    IResult,
};

use crate::Expression;

fn parse_bool(input: &str) -> IResult<&str, Expression> {
    map(alt((tag("true"), tag("false"))), |s: &str| match s {
        "true" => Expression::Boolean(true),
        "false" => Expression::Boolean(false),
        _ => unreachable!(),
    })(input)
}

fn parse_void(input: &str) -> IResult<&str, Expression> {
    map(tag("void"), |_| Expression::Void)(input)
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
            map_res(separated_pair(digit1, tag("."), digit1), |(a, b)| {
                format!("{a}.{b}").parse::<f64>()
            }),
            |f: f64| Expression::Float(f),
        ),
        map(
            map_res(
                preceded(tag("-"), separated_pair(digit1, tag("."), digit1)),
                |(a, b)| format!("{a}.{b}").parse::<f64>(),
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

pub fn parse_named(input: &str) -> IResult<&str, Expression> {
    map(
        preceded(
            char(':'),
            separated_pair(recognize(alpha1), multispace0, parse_expression),
        ),
        |(name, value)| Expression::Named {
            name: name.to_string(),
            value: Box::new(value),
        },
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
            parse_named,
            parse_list,
        )),
    )(input)
}
