use nom::branch::*;
use nom::character::complete::*;
use nom::{AsBytes, IResult, InputIter, Offset, Slice};
use nom_locate::LocatedSpan;
use nom_tracable::{
    cumulative_histogram, histogram, tracable_parser, HasTracableInfo, Tracable, TracableInfo,
};

#[derive(Clone)]
pub struct Span<'a>(LocatedSpan<&'a [u8], TracableInfo>);

impl<'a> HasTracableInfo for Span<'a> {
    fn get_tracable_info(&self) -> TracableInfo {
        self.0.extra.get_tracable_info()
    }

    fn set_tracable_info(mut self, info: TracableInfo) -> Self {
        self.0.extra = self.0.extra.set_tracable_info(info);
        self
    }
}
impl<'a> Tracable for Span<'a> {
    fn inc_depth(self) -> Self {
        let info = self.get_tracable_info();
        let info = info.depth(info.depth + 1);
        self.set_tracable_info(info)
    }

    fn dec_depth(self) -> Self {
        let info = self.get_tracable_info();
        let info = info.depth(info.depth - 1);
        self.set_tracable_info(info)
    }

    // Customize fragment format for &[u8]
    fn format(&self) -> String {
        let info = self.get_tracable_info();
        let fragment: String = String::from_utf8_lossy(self.0.fragment())
            .lines()
            .next()
            .unwrap_or_else(|| "")
            .chars()
            .take(info.fragment_width)
            .collect();
        format!("{:<8} : {}", self.0.location_offset(), fragment)
    }

    fn header(&self) -> String {
        format!("{:<8} : {}", "offset", "fragment")
    }
}
impl<'a> InputIter for Span<'a> {
    type Item = u8;
    type Iter = std::iter::Enumerate<Self::IterElem>;
    type IterElem = std::iter::Map<std::slice::Iter<'a, Self::Item>, fn(&u8) -> u8>;
    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        self.0.fragment().iter_indices()
    }
    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.0.fragment().iter_elements()
    }
    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.0.fragment().position(predicate)
    }
    #[inline]
    fn slice_index(&self, count: usize) -> Option<usize> {
        self.0.fragment().slice_index(count)
    }
}
impl<'a> Slice<std::ops::RangeFrom<usize>> for Span<'a> {
    fn slice(&self, range: std::ops::RangeFrom<usize>) -> Self {
        if (|range: &std::ops::RangeFrom<usize>| range.start == 0)(&range) {
            return self.clone();
        }
        let next_fragment = self.0.fragment().slice(range);
        let consumed_len = self.0.fragment().offset(&next_fragment);
        if consumed_len == 0 {
            let span = unsafe {
                LocatedSpan::new_from_raw_offset(
                    self.0.location_offset(),
                    self.0.location_line(),
                    next_fragment,
                    self.0.extra.clone(),
                )
            };
            return Span(span);
        }

        let consumed = self.0.fragment().slice(..consumed_len);
        let next_offset = self.0.location_offset() + consumed_len;

        let consumed_as_bytes = consumed.as_bytes();
        let iter = memchr::Memchr::new(b'\n', consumed_as_bytes);
        let number_of_lines = iter.count() as u32;
        let next_line = self.0.location_line() + number_of_lines;

        let span = unsafe {
            LocatedSpan::new_from_raw_offset(
                next_offset,
                next_line,
                next_fragment,
                self.0.extra.clone(),
            )
        };
        Span(span)
    }
}

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
    let info = TracableInfo::new()
        .parser_width(64)
        .fragment_width(20)
        .fold("term");
    let _ret = expr(Span(LocatedSpan::new_extra("1-1+1+1-1".as_bytes(), info)));

    // Show histogram
    histogram();
    cumulative_histogram();
}
