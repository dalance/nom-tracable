# nom-tracable
Extension of [nom](https://github.com/Geal/nom) to trace parser.

[![Build Status](https://dev.azure.com/dalance/nom-tracable/_apis/build/status/dalance.nom-tracable?branchName=master)](https://dev.azure.com/dalance/nom-tracable/_build/latest?definitionId=1&branchName=master)
[![Crates.io](https://img.shields.io/crates/v/nom-tracable.svg)](https://crates.io/crates/nom-tracable)
[![Docs.rs](https://docs.rs/nom-tracable/badge.svg)](https://docs.rs/nom-tracable)

## Feature

* Tracing parser by colored format
* Forward/backward call count
* Folding the specific parsers
* Histogram/cumulative histogram of parser call count
* Zero-overhead when trace is disabled

![nom-tracable](https://user-images.githubusercontent.com/4331004/63342515-a30bbc00-c386-11e9-994c-432749b168fa.png)

## Requirement

nom must be 5.0.0 or later.
nom-tracable can be applied to function-style parser only.

The input type of nom parser must implement `Tracable` trait.
Therefore `&str` and `&[u8]` can't be used.
You can define a wrapper type of `&str` or `&[u8]` and implement `Tracable`.

nom-tracable is integrated with [nom_locate](https://github.com/fflorent/nom_locate).
You can use `nom_locate::LocatedSpan<T, TracableInfo>` as input type.
This implements `Tracable` in this crate.

## Usage

```Cargo.toml
[features]
default = []
trace   = ["nom-tracable/trace"]

[dependencies]
nom-tracable = "0.5.0"
```

nom-tracable provides `trace` feature, and the crate using nom-tracable must provide the feature too.
When `trace` is enabled, trace dump is enabled.
If not, there is no additional cost.

## Example

You can try an example by the following command.

```
$ cargo run --manifest-path=nom-tracable/Cargo.toml --example example --features trace
```

The example is below:

```rust
use nom::branch::*;
use nom::character::complete::*;
use nom::IResult;
use nom_locate::LocatedSpan;
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
    let ret = expr(LocatedSpan::new_extra("1-1+1+1-1", info));

    // Show histogram
    histogram();
    cumulative_histogram();
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
