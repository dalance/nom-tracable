//! `nom-tracable` is an extension of [nom](https://docs.rs/nom) to trace parser.
//!
//! ## Examples
//!
//! The following example show a quick example.
//!
//! ```
//! use nom::character::complete::*;
//! use nom::IResult;
//! use nom_locate::LocatedSpan;
//! use nom_tracable::{tracable_parser, TracableInfo};
//!
//! // Input type must implement trait Tracable
//! // nom_locate::LocatedSpan<T, TracableInfo> implements it.
//! type Span<'a> = LocatedSpan<&'a str, TracableInfo>;
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
//!     let ret = term(LocatedSpan::new_extra("1", info));
//!     assert_eq!("\"1\"", format!("{:?}", ret.unwrap().1));
//! }
//! ```

#[cfg(feature = "trace")]
use nom::IResult;
/// Custom attribute to enable trace
pub use nom_tracable_macros::tracable_parser;
use std::collections::HashMap;

/// Trait to indicate the type can display as fragment.
pub trait FragmentDisplay {
    fn display(&self, width: usize) -> String;
}

impl FragmentDisplay for &[u8] {
    fn display(&self, width: usize) -> String {
        self.iter()
            .take(width / 2)
            .map(|x| format!("{:X}", x))
            .collect()
    }
}

impl FragmentDisplay for &str {
    fn display(&self, width: usize) -> String {
        self.lines()
            .next()
            .unwrap_or_else(|| "")
            .chars()
            .take(width)
            .collect()
    }
}

/// Trait to indicate the type has information for tracing.
pub trait Tracable: HasTracableInfo {
    fn inc_depth(self) -> Self;
    fn dec_depth(self) -> Self;
    fn format(&self) -> String;
    fn header(&self) -> String;
}

/// Trait to indicate `TracableInfo` is provided.
pub trait HasTracableInfo {
    fn get_tracable_info(&self) -> TracableInfo;
    fn set_tracable_info(self, info: TracableInfo) -> Self;
}

/// Struct to have trace configuration.
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
    fragment_width: usize,
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
            fragment_width: 96,
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

    fn depth(mut self, x: usize) -> Self {
        self.depth = x;
        self
    }

    /// Set whether forward trace is displayed.
    pub fn forward(mut self, x: bool) -> Self {
        self.forward = x;
        self
    }

    /// Set whether backward trace is displayed.
    pub fn backward(mut self, x: bool) -> Self {
        self.backward = x;
        self
    }

    /// Set whether custom trace is displayed.
    pub fn custom(mut self, x: bool) -> Self {
        self.custom = x;
        self
    }

    /// Set whether color is enabled.
    pub fn color(mut self, x: bool) -> Self {
        self.color = x;
        self
    }

    /// Set the width of forward/backward count.
    pub fn count_width(mut self, x: usize) -> Self {
        self.count_width = x;
        self
    }

    /// Set the width of parser name.
    pub fn parser_width(mut self, x: usize) -> Self {
        self.parser_width = x;
        self
    }

    /// Set the name of folding parser.
    pub fn fold(mut self, x: &str) -> Self {
        let index =
            crate::TRACABLE_STORAGE.with(|storage| storage.borrow_mut().get_parser_index(x));

        let val = 1u64 << index;
        let mask = !(1u64 << index);

        self.fold = (self.fold & mask) | val;
        self
    }

    fn folded(self, x: &str) -> bool {
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
impl<T, U: HasTracableInfo> HasTracableInfo for nom_locate1::LocatedSpanEx<T, U> {
    fn get_tracable_info(&self) -> TracableInfo {
        self.extra.get_tracable_info()
    }

    fn set_tracable_info(mut self, info: TracableInfo) -> Self {
        self.extra = self.extra.set_tracable_info(info);
        self
    }
}

#[cfg(feature = "trace")]
impl<T: FragmentDisplay, U: HasTracableInfo> Tracable for nom_locate1::LocatedSpanEx<T, U> {
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
        let info = self.get_tracable_info();
        let fragment = self.fragment.display(info.fragment_width);
        format!("{:<8} : {}", self.offset, fragment)
    }

    fn header(&self) -> String {
        format!("{:<8} : {}", "offset", "fragment")
    }
}

#[cfg(feature = "trace")]
impl<T, U: HasTracableInfo> HasTracableInfo for nom_locate::LocatedSpan<T, U> {
    fn get_tracable_info(&self) -> TracableInfo {
        self.extra.get_tracable_info()
    }

    fn set_tracable_info(mut self, info: TracableInfo) -> Self {
        self.extra = self.extra.set_tracable_info(info);
        self
    }
}

#[cfg(feature = "trace")]
impl<T: FragmentDisplay + nom::AsBytes, U: HasTracableInfo> Tracable
    for nom_locate::LocatedSpan<T, U>
{
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
        let info = self.get_tracable_info();
        let fragment = self.fragment().display(info.fragment_width);
        format!("{:<8} : {}", self.location_offset(), fragment)
    }

    fn header(&self) -> String {
        format!("{:<8} : {}", "offset", "fragment")
    }
}

#[derive(Debug, Default)]
struct TracableStorage {
    forward_count: usize,
    backward_count: usize,
    parser_indexes: HashMap<String, usize>,
    parser_index_next: usize,
    histogram: HashMap<String, usize>,
    cumulative_histogram: HashMap<String, usize>,
    cumulative_working: HashMap<(String, usize), usize>,
}

#[allow(dead_code)]
impl TracableStorage {
    fn new() -> Self {
        TracableStorage::default()
    }

    fn init(&mut self) {
        self.forward_count = 0;
        self.backward_count = 0;
        self.histogram.clear();
        self.cumulative_histogram.clear();
        self.cumulative_working.clear();
    }

    fn get_forward_count(&self) -> usize {
        self.forward_count
    }

    fn get_backward_count(&self) -> usize {
        self.backward_count
    }

    fn inc_forward_count(&mut self) {
        self.forward_count += 1
    }

    fn inc_backward_count(&mut self) {
        self.backward_count += 1
    }

    fn inc_histogram(&mut self, key: &str) {
        let next = if let Some(x) = self.histogram.get(key) {
            x + 1
        } else {
            1
        };
        self.histogram.insert(String::from(key), next);
    }

    fn inc_cumulative_histogram(&mut self, key: &str, cnt: usize) {
        let next = if let Some(x) = self.cumulative_histogram.get(key) {
            x + cnt
        } else {
            cnt
        };
        self.cumulative_histogram.insert(String::from(key), next);
    }

    fn add_cumulative(&mut self, key: &str, depth: usize) {
        self.cumulative_working
            .insert((String::from(key), depth), 0);
    }

    fn inc_cumulative(&mut self) {
        for val in self.cumulative_working.values_mut() {
            *val = *val + 1;
        }
    }

    fn del_cumulative(&mut self, key: &str, depth: usize) {
        self.cumulative_working.remove(&(key.to_string(), depth));
    }

    fn get_cumulative(&mut self, key: &str, depth: usize) -> Option<&usize> {
        self.cumulative_working.get(&(key.to_string(), depth))
    }

    fn get_parser_index(&mut self, key: &str) -> usize {
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

#[cfg(feature = "trace")]
thread_local!(
    static TRACABLE_STORAGE: core::cell::RefCell<crate::TracableStorage> = {
        core::cell::RefCell::new(crate::TracableStorage::new())
    }
);

/// Show histogram of parser call count.
///
/// The statistics information to generate histogram is reset at each parser call.
/// Therefore `histogram` should be called before next parser call.
/// The information is thread independent because it is stored at thread local storage.
///
/// ```
/// # use nom::character::complete::*;
/// # use nom::IResult;
/// # use nom_locate::LocatedSpan;
/// # use nom_tracable::{cumulative_histogram, tracable_parser, TracableInfo};
/// #
/// # type Span<'a> = LocatedSpan<&'a str, TracableInfo>;
/// #
/// # #[tracable_parser]
/// # pub fn term(s: Span) -> IResult<Span, String> {
/// #     let (s, x) = char('1')(s)?;
/// #     Ok((s, x.to_string()))
/// # }
/// #
/// # #[test]
/// # fn test() {
///     let ret = term(LocatedSpan::new_extra("1", TracableInfo::new()));
///     histogram(); // Show histogram of "1" parsing
///
///     let ret = term(LocatedSpan::new_extra("11", TracableInfo::new()));
///     histogram(); // Show histogram of "11" parsing
/// # }
/// ```
pub fn histogram() {
    histogram_internal();
}

#[cfg(feature = "trace")]
fn histogram_internal() {
    crate::TRACABLE_STORAGE.with(|storage| {
        let storage = storage.borrow();
        show_histogram("histogram", &storage.histogram);
    });
}

#[cfg(not(feature = "trace"))]
fn histogram_internal() {}

/// Show cumulative histogram of parser call count.
///
/// The call count includes the counts of children parsers.
///
/// The statistics information to generate histogram is reset at each parser call.
/// Therefore `cumulative_histogram` should be called before next parser call.
/// The information is thread independent because it is stored at thread local storage.
///
/// ```
/// # use nom::character::complete::*;
/// # use nom::IResult;
/// # use nom_locate::LocatedSpan;
/// # use nom_tracable::{cumulative_histogram, tracable_parser, TracableInfo};
/// #
/// # type Span<'a> = LocatedSpan<&'a str, TracableInfo>;
/// #
/// # #[tracable_parser]
/// # pub fn term(s: Span) -> IResult<Span, String> {
/// #     let (s, x) = char('1')(s)?;
/// #     Ok((s, x.to_string()))
/// # }
/// #
/// # #[test]
/// # fn test() {
///     let ret = term(LocatedSpan::new_extra("1", TracableInfo::new()));
///     cumulative_histogram(); // Show cumulative histogram of "1" parsing
///
///     let ret = term(LocatedSpan::new_extra("11", TracableInfo::new()));
///     cumulative_histogram(); // Show cumulative histogram of "11" parsing
/// # }
/// ```
pub fn cumulative_histogram() {
    cumulative_histogram_internal();
}

#[cfg(feature = "trace")]
fn cumulative_histogram_internal() {
    crate::TRACABLE_STORAGE.with(|storage| {
        let storage = storage.borrow();
        show_histogram("cumulative histogram", &storage.cumulative_histogram);
    });
}

#[cfg(not(feature = "trace"))]
fn cumulative_histogram_internal() {}

#[allow(dead_code)]
fn show_histogram(title: &str, map: &HashMap<String, usize>) {
    let mut result = Vec::new();
    let mut max_parser_len = "parser".len();
    let mut max_count = 0;
    let mut max_count_len = "count".len();
    for (p, c) in map {
        result.push((p, c));
        max_parser_len = max_parser_len.max(p.len());
        max_count = max_count.max(*c);
        max_count_len = max_count_len.max(format!("{}", c).len());
    }

    result.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

    let bar_length = 50;

    println!(
        "\n{:<parser$} | {:<bar$} | {}",
        "parser",
        title,
        "count",
        parser = max_parser_len,
        bar = bar_length,
    );
    println!(
        "{:<parser$} | {:<bar$} | {}",
        "-".repeat(max_parser_len),
        "-".repeat(bar_length),
        "-".repeat(max_count_len),
        parser = max_parser_len,
        bar = bar_length,
    );
    for (p, c) in &result {
        let bar = *c * bar_length / max_count;
        if bar > 0 {
            println!(
                "{:<parser$} | {}{} | {}",
                p,
                ".".repeat(bar),
                " ".repeat(bar_length - bar),
                c,
                parser = max_parser_len,
            );
        }
    }
    println!("");
}

/// Function to display forward trace.
/// This is inserted by `#[tracable_parser]`.
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

    crate::TRACABLE_STORAGE.with(|storage| {
        storage.borrow_mut().inc_histogram(name);
        storage.borrow_mut().add_cumulative(name, depth);
        storage.borrow_mut().inc_cumulative();
    });

    let input = if info.folded(name) {
        let info = info.forward(false).backward(false).custom(false);
        input.set_tracable_info(info)
    } else {
        input
    };

    let input = input.inc_depth();
    (info, input)
}

/// Function to display backward trace.
/// This is inserted by `#[tracable_parser]`.
#[cfg(feature = "trace")]
pub fn backward_trace<T: Tracable, U, V>(
    input: IResult<T, U, V>,
    name: &str,
    info: TracableInfo,
) -> IResult<T, U, V> {
    let depth = info.depth;

    crate::TRACABLE_STORAGE.with(|storage| {
        let cnt = *storage.borrow_mut().get_cumulative(name, depth).unwrap();
        storage.borrow_mut().inc_cumulative_histogram(name, cnt);
    });

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

/// Function to display custom trace.
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
