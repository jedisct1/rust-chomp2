//! Chomp is a fast monadic-style parser combinator library for the Rust
//! programming language. It was written as the culmination of the experiments
//! detailed in these blog posts:
//!
//! * [Part 1](http://m4rw3r.github.io/parser-combinator-experiments-rust)
//! * [Part 2](http://m4rw3r.github.io/parser-combinator-experiments-errors)
//! * [Part 3](http://m4rw3r.github.io/parser-combinator-experiments-part-3)
//! * [Chomp 0.1 Announcement](http://m4rw3r.github.io/parser-combinators-road-chomp-0-1)
//!
//! For its current capabilities, you will find that Chomp performs consistently
//! as well, if not better, than optimized C parsers, while being vastly more
//! expressive. For an example that builds a performant HTTP parser out of
//! smaller parsers, see [`http_parser.rs`](examples/http_parser.rs).
//!
//! # Example
//!
//! ```
//! # #[macro_use] extern crate chomp1;
//! # fn main() {
//! use chomp1::prelude::*;
//!
//! #[derive(Debug, Eq, PartialEq)]
//! struct Name<B: Buffer> {
//!     first: B,
//!     last: B,
//! }
//!
//! fn name<I: U8Input>(i: I) -> SimpleResult<I, Name<I::Buffer>> {
//!     parse! {i;
//!         let first = take_while1(|c| c != b' ');
//!                     token(b' ');  // skipping this char
//!         let last  = take_while1(|c| c != b'\n');
//!
//!         ret Name{
//!             first: first,
//!             last:  last,
//!         }
//!     }
//! }
//!
//! assert_eq!(
//!     parse_only(name, "Martin Wernstål\n".as_bytes()),
//!     Ok(Name {
//!         first: &b"Martin"[..],
//!         last: "Wernstål".as_bytes()
//!     })
//! );
//! # }
//! ```
//!
//! # Usage
//!
//! Chomp's functionality is split between three modules:
//!
//! * `parsers` contains the basic parsers used to parse streams of input.
//! * `combinators` contains functions which take parsers and return new ones.
//! * `primitives` contains the building blocks used to make new parsers. This
//!   is advanced usage and is far more involved than using the pre-existing
//!   parsers, but is sometimes unavoidable.
//!
//! A parser is, at its simplest, a function that takes a slice of input and
//! returns a `ParserResult<I, T, E>`, where `I`, `T`, and `E` are the input,
//! output, and error types, respectively. Parsers are usually parameterized
//! over values or other parsers as well, so these appear as extra arguments in
//! the parsing function. As an example, here is the signature of the
//! `token` parser, which matches a particular input.
//!
//! ```ignore
//! fn token<I: Input>(i: I, t: I::Token) -> ParseResult<I, I::Token, Error<I::Token>> { ... }
//! ```
//!
//! Notice that the first argument is an `Input<I>`, and the second argument is
//! some `I`. `Input<I>` is just a datatype over the current state of the parser
//! and a slice of input `I`, and prevents the parser writer from accidentally
//! mutating the state of the parser. Later, when we introduce the `parse!`
//! macro, we will see that using a parser in this macro just means
//! supplying all of the arguments but the input, as so:
//!
//! ```ignore
//! token(b'T');
//! ```
//!
//! Note that you cannot do this outside of the `parse!` macro. `SimpleResult<I,
//! T>` is a convenience type alias over `ParseResult<I, T, Error<u8>>`, and
//! `Error<I>` is just a convenient "default" error type that will be sufficient
//! for most uses. For more sophisticated usage, one can always write a custom
//! error type.
//!
//! A very useful parser is the `satisfy` parser:
//!
//! ```ignore
//! fn satisfy<I: Input, F>(mut i: I, f: F) -> ParseResult<I, I::Token, Error<I::Token>>
//!   where F: FnOnce(I::Token) -> bool { ... }
//! ```
//!
//! Besides the input state, satisfy's only parameter is a predicate function
//! and will succeed only if the next piece of input satisfies the supplied
//! predicate. Here's an example that might be used in the `parse!` macro:
//!
//! ```
//! # #[macro_use] extern crate chomp1;
//! # fn main() {
//! # use chomp1::prelude::*;
//! # let r = parse_only(parser!{
//! satisfy(|c| match c {
//!     b'c' | b'h' | b'a' | b'r' => true,
//!     _ => false,
//! })
//! # }, b"h");
//! # assert_eq!(r, Ok(b'h'));
//! # }
//! ```
//!
//! This parser will only succeed if the character is one of the characters in
//! "char".
//!
//! Lastly, here is the parser combinator `count`, which will attempt to run a
//! parser a number of times on its input.
//!
//! ```ignore
//! pub fn count<I: Input, T, E, F, U>(i: I, num: usize, p: F) -> ParseResult<I, T, E>
//!   where F: FnMut(I) -> ParseResult<I, U, E>,
//!         T: FromIterator<U> { ... }
//! ```
//!
//! Using parsers is almost entirely done using the `parse!` macro, which
//! enables us to do three distinct things:
//!
//! * Sequence parsers over the remaining input
//! * Store intermediate results into datatypes
//! * Return a datatype at the end, which may be the result of any arbitrary
//!   computation over the
//! intermediate results.
//!
//! In other words, just as a normal Rust function usually looks something like
//! this:
//!
//! ```
//! # fn launch_missiles() {}
//! # fn read_number() -> u8 { 3 }
//! fn f() -> (u8, u8, u8) {
//!     let a = read_number();
//!     let b = read_number();
//!     launch_missiles();
//!     return (a, b, a + b);
//! }
//! # assert_eq!(f(), (3, 3, 6));
//! ```
//!
//! A Chomp parser with a similar structure looks like this:
//!
//! ```
//! # #[macro_use] extern crate chomp1;
//! # use chomp1::prelude::*;
//! fn f<I: U8Input>(i: I) -> SimpleResult<I, (u8, u8, u8)> {
//!     parse! {i;
//!         let a = digit();
//!         let b = digit();
//!                 string(b"missiles");
//!         ret (a, b, a + b)
//!     }
//! }
//!
//! fn digit<I: U8Input>(i: I) -> SimpleResult<I, u8> {
//!     satisfy(i, |c| b'0' <= c && c <= b'9').map(|c| c - b'0')
//! }
//! # fn main() {
//! #     let r = parse_only(f, b"33missiles");
//! #     assert_eq!(r, Ok((3, 3, 6)));
//! # }
//! ```
//!
//! Readers familiar with Haskell or F# will recognize this as a "monadic
//! computation" or "computation expression".
//!
//! You use the `parse!` macro as follows:
//!
//! - Write the input parameter first, with a semicolon.
//! - Write any number of valid parser actions or identifier bindings, where:
//!    - a parser action takes the form `parser(params*)`, with the input
//!      parameter omitted.
//!    - an identifier binding takes the form `let identifer =
//!      parser(params*);`, with the input parameter omitted.
//! - Write the final line of the macro, which must always be either a parser
//!   action, or a return statement which takes the form `ret expression`. The
//!   type of `expression` becomes the return type of the entire parser, should
//!   it succeed.
//!
//! The entire grammar for the macro is listed elsewhere in this documentation.
//!
//! # Features
//!
//! * `backtrace`:
#![cfg_attr(feature = "backtrace", doc = " enabled.")]
#![cfg_attr(not(feature = "backtrace"), doc = " disabled (default).")]
//!    This feature enables backtraces for parse-errors, either by calling
//! `Error::trace` or by    printing it using `fmt::Debug`.
//!
//!    This incurs a performance-hit every time a `chomp1::parsers` parser fails
//! since a backtrace    must be collected.
//!
//!    In the `dev` and `test` profiles backtraces will always be enabled. This
//! does not incur any    cost when built using the `release` profile unless the
//! `backtrace` feature is enabled.
//!
//! * `noop_error`:
#![cfg_attr(not(feature = "noop_error"), doc = " disabled (default).")]
#![cfg_attr(feature = "noop_error", doc = " enabled.")]
//!    The built-in `chomp1::parsers::Error` type is zero-sized and carry no
//! error-information. This    increases performance somewhat.
//!
//! * `std`:
#![cfg_attr(not(feature = "std"), doc = " disabled.")]
#![cfg_attr(feature = "std", doc = " enabled (default).")]
//!    Chomp includes all features which rely on Rust's `std` library. If this
//! is diabled Chomp    will use the `no_std` feature, only using Rust's `core`
//! library.
//!
//!    Items excluded when `std` is disabled:
//!
//!     * `ascii::float` support for `type::Buffer` implementations other than
//!       `&[u8]`.
//!     * `buffer` module.
//!     * `combinators::choice` combinator.
//!     * `parsers::Error` no longer implements the `std::error::Error` trait.
//!     * `types::Buffer::to_vec`
//!     * `types::Buffer::into_vec`

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(has_specialization, feature(specialization))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(
    feature = "clippy",
    warn(
        nonminimal_bool,
        option_unwrap_used,
        print_stdout,
        result_unwrap_used,
        shadow_reuse,
        shadow_same,
        shadow_unrelated,
        single_match_else
    )
)]
#![cfg_attr(feature = "clippy", allow(inline_always, many_single_char_names))]
// `std` is required for tests.
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

/// Inner module to emulate std when using the `core` crate.
///
/// Skipped when using test since we use std for tests.
#[cfg(all(not(feature = "std"), not(test)))]
mod std {
    pub use core::{cell, cmp, fmt, iter, marker, mem, ops, ptr, str};
}

#[macro_use]
extern crate bitflags;

#[macro_use]
mod macros;
mod parse;

pub mod ascii;
// TODO: Rework buffer module so that at least a part of it can be exposed
// provided the user provides their own buffers allocated from outside.
#[cfg(feature = "std")]
pub mod buffer;
pub mod combinators;
pub mod parsers;
pub mod primitives;
pub mod types;

pub use crate::parse::parse_only;
pub use crate::parse::parse_only_str;
pub use crate::parse::run_parser;

/// Basic prelude.
pub mod prelude {
    pub use either::*;

    pub use crate::combinators::{
        count, either, many, many1, many_till, matched_by, option, or, sep_by, sep_by1, skip_many,
        skip_many1,
    };
    pub use crate::macros::*;
    pub use crate::parse_only;
    pub use crate::parse_only_str;
    pub use crate::parsers::{
        any, eof, not_token, peek, peek_next, run_scanner, satisfy, satisfy_with, scan, skip_while,
        string, take, take_remainder, take_till, take_while, take_while1, token,
    };
    pub use crate::parsers::{Error, SimpleResult};
    pub use crate::types::{Buffer, Input, ParseResult, U8Input};
}
