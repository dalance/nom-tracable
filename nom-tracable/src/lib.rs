//! `nom-recursive` is an extension of [nom](https://docs.rs/nom) to handle left recursion.
//!
//! ## Examples
//!
//! The following example show a quick example.
//! If `#[recursive_parser]` is removed, stack overflow will occur because of infinite recursion.
//!

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

/// The type of payload used by tracable parser
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
