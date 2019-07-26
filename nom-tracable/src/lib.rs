//! `nom-tracable` is an extension of [nom](https://docs.rs/nom) to trace parser.
//!
//! ## Examples
//!
//! The following example show a quick example.
//!
//! ```
//! use nom::branch::*;
//! use nom::character::complete::*;
//! use nom::IResult;
//! use nom_locate::LocatedSpanEx;
//! use nom_tracable::{tracable_parser, Tracable, TracableInfo};
//!
//! // Input type must implement trait Tracable
//! // nom_locate::LocatedSpanEx<T, TracableInfo> implements it.
//! type Span<'a> = LocatedSpanEx<&'a str, TracableInfo>;
//!
//! // Apply tracable_parser by custom attribute
//! #[tracable_parser]
//! pub fn term(s: Span) -> IResult<Span, String> {
//!     let (s, x) = char('1')(s)?;
//!     Ok((s, x.to_string()))
//! }
//!
//! #[test]
//! fn test() {
//!     let ret = term(LocatedSpanEx::new_extra("1", TracableInfo::default()));
//!     assert_eq!("\"1\"", format!("{:?}", ret.unwrap().1));
//! }
//! ```

pub use nom_tracable_macros::tracable_parser;

pub trait Tracable {
    fn get_depth(&self) -> usize;
    fn inc_depth(self) -> Self;
    fn dec_depth(self) -> Self;
    fn format(&self) -> String;
}

pub trait HasTracableInfo {
    fn get_tracable_info(&self) -> TracableInfo;
    fn set_tracable_info(self, info: TracableInfo) -> Self;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TracableInfo {
    #[cfg(any(feature = "forward_trace", feature = "backward_trace"))]
    depth: usize,
}

impl HasTracableInfo for TracableInfo {
    fn get_tracable_info(&self) -> TracableInfo {
        *self
    }

    fn set_tracable_info(self, info: TracableInfo) -> Self {
        info
    }
}

#[cfg(any(feature = "forward_trace", feature = "backward_trace"))]
impl<T: std::fmt::Display, U: HasTracableInfo> Tracable for nom_locate::LocatedSpanEx<T, U> {
    fn get_depth(&self) -> usize {
        self.extra.get_tracable_info().depth
    }

    fn inc_depth(mut self) -> Self {
        let depth = self.extra.get_tracable_info().depth + 1;
        self.extra = self.extra.set_tracable_info(TracableInfo { depth });
        self
    }

    fn dec_depth(mut self) -> Self {
        let depth = self.extra.get_tracable_info().depth - 1;
        self.extra = self.extra.set_tracable_info(TracableInfo { depth });
        self
    }

    fn format(&self) -> String {
        format!("{:<8} : {}", self.offset, self.fragment)
    }
}
