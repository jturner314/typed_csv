mod field_names_decoder;

use std::fs::File;
use std::io::{Cursor, Read};
use std::marker::PhantomData;
use std::path::Path;

use csv::{self, Decoded, Error, NextField, RecordTerminator, Result};
use rustc_serialize::Decodable;

use self::field_names_decoder::FieldNamesDecoder;

/// A CSV reader that checks the headers.
///
/// This reader parses CSV data and exposes records via iterators that decode
/// into types that implement [`rustc_serialize::Decodable`][Decodable]. This
/// reader wraps the reader from the [`csv`][csv] crate to provide a
/// [`decode()`](#method.decode) method that checks that the headers match the
/// field names in the record type.
///
/// If you don't care whether the headers match the field names, see the
/// [`csv`][csv] crate.
///
/// # Example
///
/// This example shows how to do type-based decoding for each record in the CSV
/// data:
///
/// ```rust
/// extern crate rustc_serialize;
/// # extern crate typed_csv;
/// # fn main() {
///
/// #[derive(RustcDecodable)]
/// struct Record {
///     count: usize,
///     animal: String,
///     description: String,
/// }
///
/// let data = "\
/// count,animal,description
/// 7,penguin,happy
/// 10,cheetah,fast
/// 4,armadillo,armored
/// 9,platypus,unique
/// 7,mouse,small
/// ";
///
/// let rdr = typed_csv::Reader::from_string(data);
/// for row in rdr.decode() {
///     let Record { animal, description, count } = row.unwrap();
///     println!("{}, {}: {}", animal, description, count);
/// }
/// # }
/// ```
///
/// Note that the headers must match the field names in `Record`. If the header
/// row is incorrect, the iterator will return an `Err`:
///
/// ```rust
/// # extern crate rustc_serialize;
/// # extern crate typed_csv;
/// # fn main() {
/// #
/// # #[derive(RustcDecodable)]
/// # struct Record {
/// #     count: usize,
/// #     animal: String,
/// #     description: String,
/// # }
/// #
/// let bad_data = "\
/// bad,header,row
/// 7,penguin,happy
/// 10,cheetah,fast
/// 7,mouse,small
/// ";
///
/// assert!(typed_csv::Reader::from_string(bad_data)
///     .decode()
///     .collect::<typed_csv::Result<Vec<Record>>>()
///     .is_err());
/// # }
/// ```
///
/// [csv]: https://github.com/BurntSushi/rust-csv
/// [Decodable]: https://doc.rust-lang.org/rustc-serialize/rustc_serialize/trait.Decodable.html
pub struct Reader<R: Read>(csv::Reader<R>);

impl<R: Read> Reader<R> {
    /// Creates a new CSV reader from an arbitrary `io::Read`.
    ///
    /// The reader is buffered for you automatically.
    pub fn from_reader(r: R) -> Reader<R> {
        Reader(csv::Reader::from_reader(r))
    }
}

impl Reader<File> {
    /// Creates a new CSV reader for the data at the file path given.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Reader<File>> {
        Ok(Reader(csv::Reader::from_file(path)?))
    }
}

impl Reader<Cursor<Vec<u8>>> {
    /// Creates a CSV reader for an in memory string buffer.
    pub fn from_string<'a, S>(s: S) -> Reader<Cursor<Vec<u8>>>
        where S: Into<String>
    {
        Reader(csv::Reader::from_string(s))
    }

    /// Creates a CSV reader for an in memory buffer of bytes.
    pub fn from_bytes<'a, V>(bytes: V) -> Reader<Cursor<Vec<u8>>>
        where V: Into<Vec<u8>>
    {
        Reader(csv::Reader::from_bytes(bytes))
    }
}

impl<R: Read> Reader<R> {
    /// Uses type-based decoding to read a single record from CSV data.
    ///
    /// The type that is being decoded into should correspond to *one full CSV
    /// record*. This can be a single struct, or arbitrarily nested tuples and
    /// structs, as long as all scalar types (integers, floats, characters,
    /// strings, single-element tuple structs containing a scalar type, and
    /// enums with 0 or 1 scalar arguments) are fields in structs. If the
    /// headers don't match the field names or a record cannot be decoded into
    /// the type requested, an error is returned.
    ///
    /// Enums are also supported in a limited way. Namely, its variants must
    /// have exactly `1` parameter each. Each variant decodes based on its
    /// constituent type and variants are tried in the order that they appear
    /// in their `enum` definition. See below for examples.
    ///
    /// Note that single-element tuple structs (the newtype pattern) are
    /// supported. Unfortunately, to provide this functionality, a heuristic is
    /// necessary to differentiate field names in normal structs from those in
    /// tuple structs. As a result, field names in normal structs should not be
    /// of the form `_field{}` where `{}` is its position in the struct.
    ///
    /// # Examples
    ///
    /// This example shows how to decode records into a struct. Note that
    /// the headers must match the names of the struct members.
    ///
    ///
    /// ```rust
    /// extern crate rustc_serialize;
    /// # extern crate typed_csv;
    /// # fn main() {
    ///
    /// #[derive(Debug, PartialEq, RustcDecodable)]
    /// struct Record {
    ///     count: usize,
    ///     animal: String,
    ///     description: String,
    /// }
    ///
    /// let data = "\
    /// count,animal,description
    /// 7,penguin,happy
    /// 10,cheetah,fast
    /// 4,armadillo,armored
    /// ";
    ///
    /// let rdr = typed_csv::Reader::from_string(data);
    /// let rows = rdr.decode().collect::<typed_csv::Result<Vec<Record>>>().unwrap();
    ///
    /// assert_eq!(rows,
    ///            vec![Record {
    ///                     count: 7,
    ///                     animal: "penguin".to_string(),
    ///                     description: "happy".to_string(),
    ///                 },
    ///                 Record {
    ///                     count: 10,
    ///                     animal: "cheetah".to_string(),
    ///                     description: "fast".to_string(),
    ///                 },
    ///                 Record {
    ///                     count: 4,
    ///                     animal: "armadillo".to_string(),
    ///                     description: "armored".to_string(),
    ///                 }]);
    /// # }
    /// ```
    ///
    /// We can get a little crazier with custom enum types, `Option` types,
    /// single-element tuple structs (the newtype pattern), and tuples of
    /// structs. An `Option` type in particular is useful when a column doesn't
    /// contain valid data in every record (whether it be empty or malformed).
    ///
    /// ```rust
    /// extern crate rustc_serialize;
    /// # extern crate typed_csv;
    /// # fn main() {
    ///
    /// #[derive(Debug, PartialEq, RustcDecodable)]
    /// struct MyUint(u32);
    ///
    /// #[derive(Debug, PartialEq, RustcDecodable)]
    /// enum Number { Integer(i64), Float(f64) }
    ///
    /// #[derive(Debug, PartialEq, RustcDecodable)]
    /// struct Part1 {
    ///     name1: String,
    ///     name2: String,
    ///     dist: Option<MyUint>,
    ///     dist2: Number,
    /// }
    ///
    /// #[derive(Debug, PartialEq, RustcDecodable)]
    /// struct Part2 {
    ///     size: usize,
    /// }
    ///
    /// let data = "\
    /// name1,\"name2\",dist,dist2,size
    /// foo,bar,1,1,2
    /// foo,baz,,1.5,3
    /// ";
    ///
    /// let mut rdr = typed_csv::Reader::from_string(data);
    /// let rows = rdr.decode().collect::<typed_csv::Result<Vec<(Part1, Part2)>>>().unwrap();
    ///
    /// assert_eq!(rows,
    ///            vec![(Part1 {
    ///                      name1: "foo".to_string(),
    ///                      name2: "bar".to_string(),
    ///                      dist: Some(MyUint(1)),
    ///                      dist2: Number::Integer(1),
    ///                  },
    ///                  Part2 { size: 2 }),
    ///                 (Part1 {
    ///                      name1: "foo".to_string(),
    ///                      name2: "baz".to_string(),
    ///                      dist: None,
    ///                      dist2: Number::Float(1.5),
    ///                  },
    ///                  Part2 { size: 3 })]);
    /// # }
    /// ```
    pub fn decode<D: Decodable>(self) -> DecodedRecords<R, D> {
        DecodedRecords {
            p: self.0,
            done_first: false,
            errored: false,
            record_len: 0,
            record_type: PhantomData,
        }
    }
}

impl<R: Read> Reader<R> {
    /// The delimiter to use when reading CSV data.
    ///
    /// Since the CSV reader is meant to be mostly encoding agnostic, you must
    /// specify the delimiter as a single ASCII byte. For example, to read
    /// tab-delimited data, you would use `b'\t'`.
    ///
    /// The default value is `b','`.
    pub fn delimiter(self, delimiter: u8) -> Reader<R> {
        Reader(self.0.delimiter(delimiter))
    }

    /// Set the record terminator to use when reading CSV data.
    ///
    /// In the vast majority of situations, you'll want to use the default
    /// value, `RecordTerminator::CRLF`, which automatically handles `\r`,
    /// `\n` or `\r\n` as record terminators. (Notably, this is a special
    /// case since two characters can correspond to a single terminator token.)
    ///
    /// However, you may use `RecordTerminator::Any` to specify any ASCII
    /// character to use as the record terminator. For example, you could
    /// use `RecordTerminator::Any(b'\n')` to only accept line feeds as
    /// record terminators, or `b'\x1e'` for the ASCII record separator.
    pub fn record_terminator(self, term: RecordTerminator) -> Reader<R> {
        Reader(self.0.record_terminator(term))
    }

    /// Set the quote character to use when reading CSV data.
    ///
    /// Since the CSV reader is meant to be mostly encoding agnostic, you must
    /// specify the quote as a single ASCII byte. For example, to read
    /// single quoted data, you would use `b'\''`.
    ///
    /// The default value is `b'"'`.
    ///
    /// If `quote` is `None`, then no quoting will be used.
    pub fn quote(self, quote: u8) -> Reader<R> {
        Reader(self.0.quote(quote))
    }

    /// Set the escape character to use when reading CSV data.
    ///
    /// Since the CSV reader is meant to be mostly encoding agnostic, you must
    /// specify the escape as a single ASCII byte.
    ///
    /// When set to `None` (which is the default), the "doubling" escape
    /// is used for quote character.
    ///
    /// When set to something other than `None`, it is used as the escape
    /// character for quotes. (e.g., `b'\\'`.)
    pub fn escape(self, escape: Option<u8>) -> Reader<R> {
        Reader(self.0.escape(escape))
    }

    /// Enable double quote escapes.
    ///
    /// When disabled, doubled quotes are not interpreted as escapes.
    pub fn double_quote(self, yes: bool) -> Reader<R> {
        Reader(self.0.double_quote(yes))
    }

    /// A convenience method for reading ASCII delimited text.
    ///
    /// This sets the delimiter and record terminator to the ASCII unit
    /// separator (`\x1f`) and record separator (`\x1e`), respectively.
    ///
    /// Since ASCII delimited text is meant to be unquoted, this also sets
    /// `quote` to `None`.
    pub fn ascii(self) -> Reader<R> {
        Reader(self.0.ascii())
    }
}

/// An iterator of decoded records.
///
/// The lifetime parameter `'a` refers to the lifetime of the underlying typed
/// CSV reader.
///
/// The `R` type parameter refers to the type of the underlying reader.
///
/// The `D` type parameter refers to the decoded type.
pub struct DecodedRecords<R, D: Decodable> {
    p: csv::Reader<R>,
    done_first: bool,
    errored: bool,
    record_len: usize,
    record_type: PhantomData<D>,
}


impl<R: Read, D: Decodable> DecodedRecords<R, D> {
    /// This is wrapped in the `next()` method to ensure that `self.errored` is
    /// always set properly.
    fn next_impl(&mut self) -> Option<Result<D>> {
        if !self.done_first {
            // Never do this special first record processing again.
            self.done_first = true;

            // Always consume the header record. If headers have been read
            // before this point, then this is equivalent to a harmless clone
            // (and no parser progression).
            let headers = self.p.byte_headers();

            // If the header row is empty, then the CSV data contains
            // no records. Never return zero-length records!
            if headers.as_ref().map(|r| r.is_empty()).unwrap_or(false) {
                assert!(self.p.done());
                return None;
            }

            // Otherwise, unwrap the headers.
            let headers = match headers {
                Ok(h) => h,
                Err(e) => return Some(Err(e)),
            };

            // Check that the headers match the decodable type.
            let mut field_names_decoder = FieldNamesDecoder::new();
            if let Err(e) = D::decode(&mut field_names_decoder) {
                return Some(Err(e));
            }
            let field_names = field_names_decoder.into_field_names();
            if headers != field_names {
                return Some(Err(Error::Decode("Headers don't match field names".to_string())));
            }

            // Set the record length.
            self.record_len = headers.len();
        }

        if self.p.done() || self.errored {
            return None;
        }
        let mut record = Vec::with_capacity(self.record_len);
        loop {
            match self.p.next_bytes() {
                NextField::EndOfRecord | NextField::EndOfCsv => {
                    if record.len() == 0 {
                        return None;
                    }
                    break;
                }
                NextField::Error(err) => {
                    return Some(Err(err));
                }
                NextField::Data(field) => record.push(field.to_vec()),
            }
        }
        Some(Decodable::decode(&mut Decoded::new(record)))
    }
}

impl<R: Read, D: Decodable> Iterator for DecodedRecords<R, D> {
    type Item = Result<D>;

    fn next(&mut self) -> Option<Result<D>> {
        match self.next_impl() {
            Some(Err(err)) => {
                self.errored = true;
                Some(Err(err))
            }
            other @ _ => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Reader, Result};

    #[derive(Debug, PartialEq, RustcDecodable)]
    struct SimpleStruct {
        a: usize,
        b: usize,
    }

    #[test]
    fn test_struct() {
        let rdr = Reader::from_string("a,b\n0,1\n2,3\n");
        let records = rdr.decode().collect::<Result<Vec<SimpleStruct>>>().unwrap();
        assert_eq!(records,
                   vec![SimpleStruct { a: 0, b: 1 }, SimpleStruct { a: 2, b: 3 }]);
    }

    #[test]
    fn test_struct_misnamed_headers() {
        let rdr = Reader::from_string("c,d\n0,1\n");
        let err = rdr.decode().collect::<Result<Vec<SimpleStruct>>>().unwrap_err();
        assert_eq!(format!("{}", err),
                   "CSV decode error: Headers don't match field names".to_string());
    }

    #[test]
    fn test_struct_missing_header() {
        let rdr = Reader::from_string("a\n0\n");
        let err = rdr.decode().collect::<Result<Vec<SimpleStruct>>>().unwrap_err();
        assert_eq!(format!("{}", err),
                   "CSV decode error: Headers don't match field names".to_string());
    }

    #[test]
    fn test_struct_extra_header() {
        let rdr = Reader::from_string("a,b,c\n0,1\n");
        let err = rdr.decode().collect::<Result<Vec<SimpleStruct>>>().unwrap_err();
        assert_eq!(format!("{}", err),
                   "CSV decode error: Headers don't match field names".to_string());
    }

    #[test]
    fn test_tuple_of_structs() {
        let rdr = Reader::from_string("a,b,a,b\n0,1,2,3\n4,5,6,7\n");
        let records = rdr.decode().collect::<Result<Vec<(SimpleStruct, SimpleStruct)>>>().unwrap();
        assert_eq!(records,
                   vec![(SimpleStruct { a: 0, b: 1 }, SimpleStruct { a: 2, b: 3 }),
                        (SimpleStruct { a: 4, b: 5 }, SimpleStruct { a: 6, b: 7 })]);
    }

    #[test]
    fn test_tuple_of_structs_misnamed_headers() {
        let rdr = Reader::from_string("a,b,c,d\n0,1,2,3\n4,5,6,7\n");
        let err = rdr.decode().collect::<Result<Vec<(SimpleStruct, SimpleStruct)>>>().unwrap_err();
        assert_eq!(format!("{}", err),
                   "CSV decode error: Headers don't match field names".to_string());
    }
}
