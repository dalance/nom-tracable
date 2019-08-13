use nom::branch::*;
use nom::character::complete::*;
use nom::IResult;
use nom_locate::LocatedSpanEx;
use nom_tracable::{tracable_parser, TracableInfo};

type Span<'a> = LocatedSpanEx<&'a str, TracableInfo>;

#[tracable_parser]
pub fn expr(s: Span) -> IResult<Span, String> {
    alt((expr_plus, expr_minus, term))(s)
}

#[tracable_parser]
pub fn expr_plus(s: Span) -> IResult<Span, String> {
    let (s, x) = term(s)?;
    let (s, y) = char('+')(s)?;
    let (s, z) = expr(s)?;
    let ret = format!("{}{}{}", x, y, z);
    Ok((s, ret))
}

#[tracable_parser]
pub fn expr_minus(s: Span) -> IResult<Span, String> {
    let (s, x) = term(s)?;
    let (s, y) = char('-')(s)?;
    let (s, z) = expr(s)?;
    let ret = format!("{}{}{}", x, y, z);
    Ok((s, ret))
}

#[tracable_parser]
pub fn term(s: Span) -> IResult<Span, String> {
    term_inner(s)
}

#[tracable_parser]
pub fn term_inner(s: Span) -> IResult<Span, String> {
    let (s, x) = char('1')(s)?;
    Ok((s, x.to_string()))
}

#[test]
fn test() {
    let ret = expr(LocatedSpanEx::new_extra("1", TracableInfo::new()));
    assert_eq!("\"1\"", format!("{:?}", ret.unwrap().1));

    let ret = expr(LocatedSpanEx::new_extra("1+1", TracableInfo::new()));
    assert_eq!("\"1+1\"", format!("{:?}", ret.unwrap().1));

    let ret = expr(LocatedSpanEx::new_extra(
        "1-1+1+1-1+1+1-1+1",
        TracableInfo::new(),
    ));
    assert_eq!("\"1-1+1+1-1+1+1-1+1\"", format!("{:?}", ret.unwrap().1));

    let ret = expr(LocatedSpanEx::new_extra(
        "1-1+1+1-1+1+1-1+1",
        TracableInfo::new().fold("term"),
    ));
    assert_eq!("\"1-1+1+1-1+1+1-1+1\"", format!("{:?}", ret.unwrap().1));

    let ret = expr(LocatedSpanEx::new_extra(
        "1-1+1+1-1+1+1-1+1",
        TracableInfo::new().fold("term").color(false),
    ));
    assert_eq!("\"1-1+1+1-1+1+1-1+1\"", format!("{:?}", ret.unwrap().1));
}
