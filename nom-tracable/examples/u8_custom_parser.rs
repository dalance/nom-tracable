use nom::branch::*;
use nom::character::complete::*;
use nom::{IResult, Input, Needed, Parser};
use nom_locate::LocatedSpan;
use nom_tracable::{cumulative_histogram, histogram, tracable_parser, TracableInfo};
#[cfg(feature = "trace")]
use nom_tracable::{HasTracableInfo, Tracable};

#[derive(Clone)]
pub struct Span<'a>(LocatedSpan<&'a [u8], TracableInfo>);

#[cfg(feature = "trace")]
impl<'a> HasTracableInfo for Span<'a> {
    fn get_tracable_info(&self) -> TracableInfo {
        self.0.get_tracable_info()
    }

    fn set_tracable_info(mut self, info: TracableInfo) -> Self {
        self.0 = self.0.set_tracable_info(info);
        self
    }
}
#[cfg(feature = "trace")]
impl<'a> Tracable for Span<'a> {
    fn inc_depth(mut self) -> Self {
        self.0 = self.0.inc_depth();
        self
    }

    fn dec_depth(mut self) -> Self {
        self.0 = self.0.dec_depth();
        self
    }

    // Customize fragment format for &[u8]
    fn format(&self) -> String {
        let info = self.get_tracable_info();
        let fragment: String = String::from_utf8_lossy(self.0.fragment())
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(info.fragment_width)
            .collect();
        format!("{:<8} : {}", self.0.location_offset(), fragment)
    }

    fn header(&self) -> String {
        self.0.header()
    }
}
impl<'a> Input for Span<'a> {
    type Item = u8;
    type Iter = std::iter::Copied<std::slice::Iter<'a, Self::Item>>;
    type IterIndices = std::iter::Enumerate<Self::Iter>;
    #[inline]
    fn input_len(&self) -> usize {
        self.0.len()
    }
    #[inline]
    fn iter_elements(&self) -> Self::Iter {
        self.0.iter_elements()
    }
    #[inline]
    fn iter_indices(&self) -> Self::IterIndices {
        self.0.iter_indices()
    }
    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.0.position(predicate)
    }
    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        self.0.slice_index(count)
    }
    #[inline]
    fn take(&self, index: usize) -> Self {
        Self(self.0.take(index))
    }
    #[inline]
    fn take_from(&self, index: usize) -> Self {
        Self(self.0.take_from(index))
    }
    #[inline]
    fn take_split(&self, index: usize) -> (Self, Self) {
        let (a, b) = self.0.take_split(index);
        (Self(a), Self(b))
    }
}

// Apply tracable_parser by custom attribute
#[tracable_parser]
pub fn expr(s: Span) -> IResult<Span, String> {
    alt((expr_plus, expr_minus, term)).parse(s)
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
    let info = TracableInfo::new()
        .parser_width(64)
        .fragment_width(20)
        .fold("term");
    let _ret = expr(Span(LocatedSpan::new_extra("1-1+1+1-1".as_bytes(), info)));

    // Show histogram
    histogram();
    cumulative_histogram();
}
