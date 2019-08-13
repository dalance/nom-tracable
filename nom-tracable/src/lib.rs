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
//! use nom_tracable::{tracable_parser, TracableInfo};
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
//!     // Configure trace setting
//!     let info = TracableInfo::new().forward(true).backward(true);
//!     let ret = term(LocatedSpanEx::new_extra("1", info));
//!     assert_eq!("\"1\"", format!("{:?}", ret.unwrap().1));
//! }
//! ```

#[cfg(feature = "trace")]
use nom::IResult;
pub use nom_tracable_macros::tracable_parser;
use std::collections::HashMap;

pub trait Tracable: HasTracableInfo {
    fn inc_depth(self) -> Self;
    fn dec_depth(self) -> Self;
    fn format(&self) -> String;
    fn header(&self) -> String;
}

pub trait HasTracableInfo {
    fn get_tracable_info(&self) -> TracableInfo;
    fn set_tracable_info(self, info: TracableInfo) -> Self;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TracableInfo {
    #[cfg(feature = "trace")]
    depth: usize,
    #[cfg(feature = "trace")]
    forward: bool,
    #[cfg(feature = "trace")]
    backward: bool,
    #[cfg(feature = "trace")]
    custom: bool,
    #[cfg(feature = "trace")]
    color: bool,
    #[cfg(feature = "trace")]
    count_width: usize,
    #[cfg(feature = "trace")]
    parser_width: usize,
    #[cfg(feature = "trace")]
    fold: u64,
}

impl Default for TracableInfo {
    fn default() -> Self {
        TracableInfo {
            #[cfg(feature = "trace")]
            depth: 0,
            #[cfg(feature = "trace")]
            forward: true,
            #[cfg(feature = "trace")]
            backward: true,
            #[cfg(feature = "trace")]
            custom: true,
            #[cfg(feature = "trace")]
            color: true,
            #[cfg(feature = "trace")]
            count_width: 10,
            #[cfg(feature = "trace")]
            parser_width: 96,
            #[cfg(feature = "trace")]
            fold: 0,
        }
    }
}

#[cfg(feature = "trace")]
impl TracableInfo {
    pub fn new() -> Self {
        TracableInfo::default()
    }

    pub fn depth(mut self, x: usize) -> Self {
        self.depth = x;
        self
    }

    pub fn forward(mut self, x: bool) -> Self {
        self.forward = x;
        self
    }

    pub fn backward(mut self, x: bool) -> Self {
        self.backward = x;
        self
    }

    pub fn custom(mut self, x: bool) -> Self {
        self.custom = x;
        self
    }

    pub fn color(mut self, x: bool) -> Self {
        self.color = x;
        self
    }

    pub fn count_width(mut self, x: usize) -> Self {
        self.count_width = x;
        self
    }

    pub fn parser_width(mut self, x: usize) -> Self {
        self.parser_width = x;
        self
    }

    pub fn fold(mut self, x: &str) -> Self {
        let index =
            crate::TRACABLE_STORAGE.with(|storage| storage.borrow_mut().get_parser_index(x));

        let val = 1u64 << index;
        let mask = !(1u64 << index);

        self.fold = (self.fold & mask) | val;
        self
    }

    pub fn enabled(self) -> bool {
        self.forward | self.backward | self.custom
    }

    pub fn folded(self, x: &str) -> bool {
        let index =
            crate::TRACABLE_STORAGE.with(|storage| storage.borrow_mut().get_parser_index(x));

        if index < 64 {
            ((self.fold >> index) & 1u64) == 1u64
        } else {
            false
        }
    }
}

#[cfg(not(feature = "trace"))]
impl TracableInfo {
    pub fn new() -> Self {
        TracableInfo::default()
    }

    pub fn depth(self, _x: usize) -> Self {
        self
    }

    pub fn forward(self, _x: bool) -> Self {
        self
    }

    pub fn backward(self, _x: bool) -> Self {
        self
    }

    pub fn custom(self, _x: bool) -> Self {
        self
    }

    pub fn color(self, _x: bool) -> Self {
        self
    }

    pub fn count_width(self, _x: usize) -> Self {
        self
    }

    pub fn parser_width(self, _x: usize) -> Self {
        self
    }

    pub fn fold(self, _x: &str) -> Self {
        self
    }

    pub fn folded(self, _x: &str) -> bool {
        false
    }
}

impl HasTracableInfo for TracableInfo {
    fn get_tracable_info(&self) -> TracableInfo {
        *self
    }

    fn set_tracable_info(self, info: TracableInfo) -> Self {
        info
    }
}

#[cfg(feature = "trace")]
impl<T: std::fmt::Display, U: HasTracableInfo> HasTracableInfo for nom_locate::LocatedSpanEx<T, U> {
    fn get_tracable_info(&self) -> TracableInfo {
        self.extra.get_tracable_info()
    }

    fn set_tracable_info(mut self, info: TracableInfo) -> Self {
        self.extra = self.extra.set_tracable_info(info);
        self
    }
}

#[cfg(feature = "trace")]
impl<T: std::fmt::Display, U: HasTracableInfo> Tracable for nom_locate::LocatedSpanEx<T, U> {
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

    fn format(&self) -> String {
        format!("{:<8} : {}", self.offset, self.fragment)
    }

    fn header(&self) -> String {
        format!("{:<8} : {}", "offset", "fragment")
    }
}

#[derive(Debug, Default)]
pub struct TracableStorage {
    forward_count: usize,
    backward_count: usize,
    parser_indexes: HashMap<String, usize>,
    parser_index_next: usize,
}

impl TracableStorage {
    fn new() -> Self {
        TracableStorage::default()
    }

    pub fn init(&mut self) {
        self.forward_count = 0;
        self.backward_count = 0;
    }

    pub fn get_forward_count(&self) -> usize {
        self.forward_count
    }

    pub fn get_backward_count(&self) -> usize {
        self.backward_count
    }

    pub fn inc_forward_count(&mut self) {
        self.forward_count += 1
    }

    pub fn inc_backward_count(&mut self) {
        self.backward_count += 1
    }

    pub fn get_parser_index(&mut self, key: &str) -> usize {
        if let Some(x) = self.parser_indexes.get(key) {
            *x
        } else {
            let new_index = self.parser_index_next;
            self.parser_index_next += 1;
            self.parser_indexes.insert(String::from(key), new_index);
            new_index
        }
    }
}

thread_local!(
    pub static TRACABLE_STORAGE: core::cell::RefCell<crate::TracableStorage> = {
        core::cell::RefCell::new(crate::TracableStorage::new())
    }
);

#[cfg(feature = "trace")]
pub fn forward_trace<T: Tracable>(input: T, name: &str) -> (TracableInfo, T) {
    let info = input.get_tracable_info();
    let depth = info.depth;

    if depth == 0 {
        crate::TRACABLE_STORAGE.with(|storage| {
            storage.borrow_mut().init();
        });
        let forward_backword = if info.forward & info.backward {
            format!(
                "{:<count_width$} {:<count_width$}",
                "forward",
                "backward",
                count_width = info.count_width
            )
        } else if info.forward {
            format!(
                "{:<count_width$}",
                "forward",
                count_width = info.count_width
            )
        } else {
            format!(
                "{:<count_width$}",
                "backward",
                count_width = info.count_width
            )
        };

        let control_witdh = if info.color { 11 } else { 0 };

        println!(
            "\n{} : {:<parser_width$} : {}",
            forward_backword,
            "parser",
            input.header(),
            parser_width = info.parser_width - control_witdh,
        );
    }

    if info.forward {
        let forward_count = crate::TRACABLE_STORAGE.with(|storage| {
            storage.borrow_mut().inc_forward_count();
            storage.borrow().get_forward_count()
        });

        let forward_backword = if info.backward {
            format!(
                "{:<count_width$} {:<count_width$}",
                forward_count,
                "",
                count_width = info.count_width
            )
        } else {
            format!(
                "{:<count_width$}",
                forward_count,
                count_width = info.count_width
            )
        };

        let color = if info.color { "\u{001b}[1;37m" } else { "" };
        let reset = if info.color { "\u{001b}[0m" } else { "" };
        let folded = if info.folded(name) { "+" } else { " " };

        println!(
            "{} : {:<parser_width$} : {}",
            forward_backword,
            format!(
                "{}{}-> {} {}{}",
                color,
                " ".repeat(depth),
                name,
                folded,
                reset
            ),
            input.format(),
            parser_width = info.parser_width,
        );
    }

    let input = if info.folded(name) {
        let info = info.forward(false).backward(false).custom(false);
        input.set_tracable_info(info)
    } else {
        input
    };

    let input = input.inc_depth();
    (info, input)
}

#[cfg(feature = "trace")]
pub fn backward_trace<T: Tracable, U>(
    input: IResult<T, U>,
    name: &str,
    info: TracableInfo,
) -> IResult<T, U> {
    let depth = info.depth;

    if info.backward {
        let backward_count = crate::TRACABLE_STORAGE.with(|storage| {
            storage.borrow_mut().inc_backward_count();
            storage.borrow().get_backward_count()
        });

        let forward_backword = if info.forward {
            format!(
                "{:<count_width$} {:<count_width$}",
                "",
                backward_count,
                count_width = info.count_width
            )
        } else {
            format!(
                "{:<count_width$}",
                backward_count,
                count_width = info.count_width
            )
        };

        let color_ok = if info.color { "\u{001b}[1;32m" } else { "" };
        let color_err = if info.color { "\u{001b}[1;31m" } else { "" };
        let reset = if info.color { "\u{001b}[0m" } else { "" };
        let folded = if info.folded(name) { "+" } else { " " };

        match input {
            Ok((s, x)) => {
                println!(
                    "{} : {:<parser_width$} : {}",
                    forward_backword,
                    format!(
                        "{}{}<- {} {}{}",
                        color_ok,
                        " ".repeat(depth),
                        name,
                        folded,
                        reset
                    ),
                    s.format(),
                    parser_width = info.parser_width,
                );

                let s = if info.folded(name) {
                    let info = s
                        .get_tracable_info()
                        .forward(info.forward)
                        .backward(info.backward)
                        .custom(info.custom);
                    s.set_tracable_info(info)
                } else {
                    s
                };

                Ok((s.dec_depth(), x))
            }
            Err(x) => {
                println!(
                    "{} : {:<parser_width$}",
                    forward_backword,
                    format!(
                        "{}{}<- {} {}{}",
                        color_err,
                        " ".repeat(depth),
                        name,
                        folded,
                        reset
                    ),
                    parser_width = info.parser_width,
                );
                Err(x)
            }
        }
    } else {
        input
    }
}

#[cfg(feature = "trace")]
pub fn custom_trace<T: Tracable>(input: &T, name: &str, message: &str, color: &str) {
    let info = input.get_tracable_info();

    if info.custom {
        let depth = info.depth;
        let forward_backword = format!(
            "{:<count_width$} {:<count_width$}",
            "",
            "",
            count_width = info.count_width
        );

        let color = if info.color { color } else { "" };
        let reset = if info.color { "\u{001b}[0m" } else { "" };

        println!(
            "{} : {:<parser_width$} : {}",
            forward_backword,
            format!("{}{}   {}{}", color, " ".repeat(depth), name, reset),
            message,
            parser_width = info.parser_width,
        );
    }
}
