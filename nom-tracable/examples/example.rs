use nom::branch::*;
use nom::character::complete::*;
use nom::IResult;
use nom_locate::LocatedSpanEx;
use nom_tracable::{tracable_parser, Tracable, TracableInfo};

// Input type must implement trait Tracable
// nom_locate::LocatedSpanEx<T, TracableInfo> implements it.
type Span<'a> = LocatedSpanEx<&'a str, TracableInfo>;

// Apply tracable_parser by custom attribute
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
    let (s, x) = char('1')(s)?;
    Ok((s, x.to_string()))
}

fn main() {
    let ret = expr(LocatedSpanEx::new_extra(
        "1-1+1+1-1+1+1-1+1",
        TracableInfo::default(),
    ));
    println!("{:?}", ret.unwrap().1);
}
