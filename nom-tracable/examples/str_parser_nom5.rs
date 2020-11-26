use nom5::branch::*;
use nom5::character::complete::*;
use nom5::IResult;
use nom_locate2::LocatedSpan;
use nom_tracable::{cumulative_histogram, histogram, tracable_parser, TracableInfo};

// Input type must implement trait Tracable
// nom_locate::LocatedSpan<T, TracableInfo> implements it.
type Span<'a> = LocatedSpan<&'a str, TracableInfo>;

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
    let (s, x) = term_internal(s)?;
    Ok((s, x))
}

#[tracable_parser]
pub fn term_internal(s: Span) -> IResult<Span, String> {
    let (s, x) = char('1')(s)?;
    Ok((s, x.to_string()))
}

fn main() {
    // Configure trace setting
    let info = TracableInfo::new().parser_width(64).fold("term");
    let _ret = expr(LocatedSpan::new_extra("1-1+1+1-1", info));

    // Show histogram
    histogram();
    cumulative_histogram();
}
