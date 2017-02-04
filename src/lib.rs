//! This crate provides streaming CSV (comma separated values) wrappers for the
//! reader and writer in the [`csv`][csv] crate. It works with the
//! [`rustc_serialize`][rustc_serialize] crate to do type based encoding and
//! decoding.
//!
//! *Unlike the [`csv`][csv] crate*, the reader requires that the header names
//! match the field names in the decoded record type, and the writer
//! automatically adds a header row using the field names in the record type.
//! Otherwise, this crate's interface is effectively just a subset of the
//! [`csv`][csv] crate's interface.
//!
//! If you don't care if the headers match the decoded type or you want to
//! write your own headers, see the [`csv`][csv] crate.
//!
//! # Examples
//!
//! See the documentation for [`Reader`](struct.Reader.html) and
//! [`Writer`](struct.Writer.html).
//!
//! # Compliance with RFC 4180
//!
//! See the [documentation](http://burntsushi.net/rustdoc/csv/) for the
//! [`csv`][csv] crate.
//!
//! # License
//!
//! Significant portions of this crate are closely based on code from the
//! [`csv`][csv] crate, which is dual-licensed under the Unlicense and MIT
//! licenses. Many thanks to [BurntSushi](http://burntsushi.net/) (Andrew
//! Gallant) for creating such a fast and featureful CSV crate!
//!
//! This crate is similarly dual-licensed under the Unlicense and MIT licenses.
//! See `COPYING` for more information.
//!
//! [csv]: https://github.com/BurntSushi/rust-csv
//! [rustc_serialize]: https://doc.rust-lang.org/rustc-serialize/rustc_serialize/index.html

extern crate csv;
extern crate rustc_serialize;

mod reader;
mod writer;

pub use csv::{Error, LocatableError, ParseError, QuoteStyle, RecordTerminator, Result};
pub use reader::{DecodedRecords, Reader};
pub use writer::Writer;
