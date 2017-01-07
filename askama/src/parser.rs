use nom::{self, IResult};
use std::str;

pub enum Expr<'a> {
     Var(&'a [u8]),
     Filter(&'a str, Box<Expr<'a>>),
}

pub enum Node<'a> {
    Lit(&'a [u8]),
    Expr(Expr<'a>),
}

fn take_content(i: &[u8]) -> IResult<&[u8], Node> {
    if i.len() < 1 || i[0] == b'{' {
        return IResult::Error(error_position!(nom::ErrorKind::TakeUntil, i));
    }
    for (j, c) in i.iter().enumerate() {
        if *c == b'{' {
            if i.len() < j + 2 {
                return IResult::Done(&i[..0], Node::Lit(&i[..]));
            } else if i[j + 1] == '{' as u8 {
                return IResult::Done(&i[j..], Node::Lit(&i[..j]));
            } else if i[j + 1] == '%' as u8 {
                return IResult::Done(&i[j..], Node::Lit(&i[..j]));
            }
        }
    }
    IResult::Done(&i[..0], Node::Lit(&i[..]))
}

named!(expr_var<Expr>, map!(nom::alphanumeric, Expr::Var));

fn expr_filtered(i: &[u8]) -> IResult<&[u8], Expr> {
    let (mut left, mut expr) = match expr_var(i) {
        IResult::Error(err) => { return IResult::Error(err); },
        IResult::Incomplete(needed) => { return IResult::Incomplete(needed); },
        IResult::Done(left, res) => (left, res),
    };
    while left[0] == b'|' {
        match nom::alphanumeric(&left[1..]) {
            IResult::Error(err) => {
                return IResult::Error(err);
            },
            IResult::Incomplete(needed) => {
                return IResult::Incomplete(needed);
            },
            IResult::Done(new_left, res) => {
                left = new_left;
                expr = Expr::Filter(str::from_utf8(res).unwrap(), Box::new(expr));
            },
        };
    }
    return IResult::Done(left, expr);
}

named!(expr_node<Node>, map!(
    delimited!(tag_s!("{{"), ws!(expr_filtered), tag_s!("}}")),
    Node::Expr));

named!(parse_template< Vec<Node> >, many1!(alt!(take_content | expr_node)));

pub fn parse<'a>(src: &'a str) -> Vec<Node> {
    match parse_template(src.as_bytes()) {
        IResult::Done(_, res) => res,
        IResult::Error(err) => panic!("problems parsing template source: {}", err),
        IResult::Incomplete(_) => panic!("parsing incomplete"),
    }
}