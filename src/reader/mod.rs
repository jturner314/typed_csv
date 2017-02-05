mod field_names_decoder;

use self::field_names_decoder::FieldNamesDecoder;

use csv::{self, ByteString, Decoded, Error, NextField, RecordTerminator, Result};
use rustc_serialize::Decodable;
use std::ascii::AsciiExt;
use std::fs::File;
use std::io::{Cursor, Read};
use std::iter;
use std::marker::PhantomData;
use std::path::Path;

/// A CSV reader that checks the headers.
///
/// This reader parses CSV data and exposes records via iterators that decode
/// into types that implement [`rustc_serialize::Decodable`][Decodable]. This
/// reader wraps the reader from the [`csv`][csv] crate to provide a
/// [`decode()`](#method.decode) method that checks that the headers match the
/// field names in the record type.
///
/// If the ordering of the headers in the file doesn't matter for your use
/// case, you can ask the reader to [reorder](#method.reorder) the columns to
/// match the headers to the corresponding field names. You also specify that
/// the headers are [case-insensitive](#method.ignore_ascii_case).
///
/// If you don't care whether the headers match the field names at all, see the
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
/// Note that the headers must match the field names in `Record` (although you
/// can ask the reader to [reorder](#method.reorder) the columns to match the
/// headers to the field names). If the header row is incorrect, the iterator
/// will return an `Err`:
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
pub struct Reader<R: Read> {
    csv: csv::Reader<R>,
    reorder: bool,
    ignore_ascii_case: bool,
}

impl<R: Read> Reader<R> {
    /// Creates a new typed CSV reader from a normal CSV reader.
    ///
    /// *Do not make this public!* If it was public, a CSV reader with
    /// `flexible = true` or `has_headers = false` could be passed in.
    fn from_csv_reader(csv: csv::Reader<R>) -> Reader<R> {
        Reader {
            csv: csv,
            reorder: false,
            ignore_ascii_case: false,
        }
    }

    /// Creates a new CSV reader from an arbitrary `io::Read`.
    ///
    /// The reader is buffered for you automatically.
    pub fn from_reader(r: R) -> Reader<R> {
        Reader::from_csv_reader(csv::Reader::from_reader(r))
    }
}

impl Reader<File> {
    /// Creates a new CSV reader for the data at the file path given.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Reader<File>> {
        Ok(Reader::from_csv_reader(csv::Reader::from_file(path)?))
    }
}

impl Reader<Cursor<Vec<u8>>> {
    /// Creates a CSV reader for an in memory string buffer.
    pub fn from_string<S>(s: S) -> Reader<Cursor<Vec<u8>>>
        where S: Into<String>
    {
        Reader::from_csv_reader(csv::Reader::from_string(s))
    }

    /// Creates a CSV reader for an in memory buffer of bytes.
    pub fn from_bytes<V>(bytes: V) -> Reader<Cursor<Vec<u8>>>
        where V: Into<Vec<u8>>
    {
        Reader::from_csv_reader(csv::Reader::from_bytes(bytes))
    }
}

impl<R: Read> Reader<R> {
    /// Uses type-based decoding to read a single record from CSV data.
    ///
    /// The type that is being decoded into should correspond to *one full CSV
    /// record*. This can be a single struct, or arbitrarily nested tuples and
    /// structs, as long as all scalar types (integers, floats, characters,
    /// strings, single-element tuple structs containing a scalar type, and
    /// enums with 0 or 1 scalar arguments) are fields in structs.
    ///
    /// If the headers don't match the field names or a record cannot be
    /// decoded into the type requested, an error is returned. See the
    /// [`reorder`](method.reorder) method if you'd like for the reader to
    /// automatically reorder columns to match headers to field names.
    ///
    /// Enums are supported in a limited way. Namely, its variants must have
    /// exactly `1` parameter each. Each variant decodes based on its
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
            p: self.csv,
            reorder: self.reorder,
            ignore_ascii_case: self.ignore_ascii_case,
            done_first: false,
            errored: false,
            column_mapping: Vec::new(),
            record_type: PhantomData,
        }
    }
}

impl<R: Read> Reader<R> {
    /// Allow the reader to reorder columns to match headers to field names.
    ///
    /// By default, the headers must match the field names of the decodable
    /// type exactly, including the order. However, the ordering of field names
    /// may not be relevant to your data type, so this option is available.
    ///
    /// In the case of duplicate field names, the ordering of columns
    /// corresponding to those fields will be preserved.
    ///
    /// # Examples
    ///
    /// This is a simple example that demonstrates reordering the columns to
    /// match headers to field names:
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
    /// count,description,animal
    /// 7,happy,penguin
    /// 10,fast,cheetah
    /// 4,armored,armadillo
    /// ";
    ///
    /// let rdr = typed_csv::Reader::from_string(data);
    /// let rows = rdr.reorder(true).decode().collect::<typed_csv::Result<Vec<Record>>>().unwrap();
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
    /// Duplicate field names in decodable types are fine, and ordering of
    /// columns with duplicate headers is preserved:
    ///
    /// ```rust
    /// extern crate rustc_serialize;
    /// # extern crate typed_csv;
    /// # fn main() {
    ///
    /// #[derive(Debug, PartialEq, RustcDecodable)]
    /// struct Animal {
    ///     count: usize,
    ///     animal: String,
    /// }
    ///
    /// let data = "\
    /// count,animal,animal,count
    /// 7,penguin,\"red panda\",2
    /// 10,cheetah,fennec,9
    /// 4,armadillo,quokka,3
    /// ";
    ///
    /// type Record = (Animal, Animal);
    ///
    /// let rdr = typed_csv::Reader::from_string(data);
    /// let rows = rdr.reorder(true).decode().collect::<typed_csv::Result<Vec<Record>>>().unwrap();
    ///
    /// assert_eq!(rows,
    ///            vec![(Animal { count: 7, animal: "penguin".to_string() },
    ///                  Animal { count: 2, animal: "red panda".to_string() }),
    ///                 (Animal { count: 10, animal: "cheetah".to_string() },
    ///                  Animal { count: 9, animal: "fennec".to_string() }),
    ///                 (Animal { count: 4, animal: "armadillo".to_string() },
    ///                  Animal { count: 3, animal: "quokka".to_string() })]);
    /// # }
    /// ```
    pub fn reorder(mut self, reorder: bool) -> Reader<R> {
        self.reorder = reorder;
        self
    }

    /// When matching headers to field names, use an ASCII case-insensitive
    /// match.
    ///
    /// The default value is `false`.
    pub fn ignore_ascii_case(mut self, yes: bool) -> Reader<R> {
        self.ignore_ascii_case = yes;
        self
    }

    /// The delimiter to use when reading CSV data.
    ///
    /// Since the CSV reader is meant to be mostly encoding agnostic, you must
    /// specify the delimiter as a single ASCII byte. For example, to read
    /// tab-delimited data, you would use `b'\t'`.
    ///
    /// The default value is `b','`.
    pub fn delimiter(mut self, delimiter: u8) -> Reader<R> {
        self.csv = self.csv.delimiter(delimiter);
        self
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
    pub fn record_terminator(mut self, term: RecordTerminator) -> Reader<R> {
        self.csv = self.csv.record_terminator(term);
        self
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
    pub fn quote(mut self, quote: u8) -> Reader<R> {
        self.csv = self.csv.quote(quote);
        self
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
    pub fn escape(mut self, escape: Option<u8>) -> Reader<R> {
        self.csv = self.csv.escape(escape);
        self
    }

    /// Enable double quote escapes.
    ///
    /// When disabled, doubled quotes are not interpreted as escapes.
    pub fn double_quote(mut self, yes: bool) -> Reader<R> {
        self.csv = self.csv.double_quote(yes);
        self
    }

    /// A convenience method for reading ASCII delimited text.
    ///
    /// This sets the delimiter and record terminator to the ASCII unit
    /// separator (`\x1f`) and record separator (`\x1e`), respectively.
    ///
    /// Since ASCII delimited text is meant to be unquoted, this also sets
    /// `quote` to `None`.
    pub fn ascii(mut self) -> Reader<R> {
        self.csv = self.csv.ascii();
        self
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
    reorder: bool,
    ignore_ascii_case: bool,
    done_first: bool,
    errored: bool,
    /// Indices are column numbers and values are field numbers.
    column_mapping: Vec<usize>,
    record_type: PhantomData<D>,
}

/// Determinines mapping of columns to fields according to headers and field names.
///
/// The mapping is a `Vec` of indices, where the indices of the `Vec` are the
/// column indices, and the values of the `Vec` are the field indices.
fn map_headers(headers: &[ByteString],
               field_names: &[ByteString],
               reorder: bool,
               ignore_ascii_case: bool)
               -> Result<Vec<usize>> {
    if headers.len() != field_names.len() {
        return Err(Error::Decode(format!("The decodable type has {} field names, but there are \
                                          {} headers",
                                         field_names.len(),
                                         headers.len())));
    }
    let predicate = if ignore_ascii_case {
        <[u8]>::eq_ignore_ascii_case
    } else {
        <[u8]>::eq
    };
    if reorder {
        let mut mapping = Vec::with_capacity(headers.len());
        // Whether fields have been used yet.
        let mut used = iter::repeat(false).take(headers.len()).collect::<Vec<_>>();
        for header in headers {
            // Search for the first matching field that hasn't been used yet.
            let found = field_names.iter()
                .enumerate()
                .find(|&(field_index, field)| predicate(header, field) && !used[field_index]);
            match found {
                Some((field_index, _)) => {
                    mapping.push(field_index);
                    used[field_index] = true;
                }
                None => {
                    return Err(Error::Decode("Headers don't match field names".to_string()));
                }
            }
        }
        Ok(mapping)
    } else if headers.iter().zip(field_names).all(|(h, f)| predicate(h, f)) {
        Ok((0..headers.len()).collect())
    } else {
        Err(Error::Decode("Headers don't match field names".to_string()))
    }
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

            // Get the field names of the decodable type.
            let mut field_names_decoder = FieldNamesDecoder::new();
            if let Err(e) = D::decode(&mut field_names_decoder) {
                return Some(Err(e));
            }
            let field_names = field_names_decoder.into_field_names();

            // Determine mapping of headers to field names.
            match map_headers(&headers, &field_names, self.reorder, self.ignore_ascii_case) {
                Ok(mapping) => {
                    self.column_mapping = mapping;
                }
                Err(err) => {
                    return Some(Err(err));
                }
            }
        }

        if self.p.done() || self.errored {
            return None;
        }

        let mut record =
            iter::repeat(Vec::new()).take(self.column_mapping.len()).collect::<Vec<_>>();
        let mut column = 0;
        loop {
            match self.p.next_bytes() {
                NextField::EndOfRecord | NextField::EndOfCsv => {
                    if record.is_empty() {
                        return None;
                    }
                    break;
                }
                NextField::Error(err) => {
                    return Some(Err(err));
                }
                NextField::Data(field) => {
                    if column < self.column_mapping.len() {
                        record[self.column_mapping[column]] = field.to_vec();
                        column += 1;
                    } else {
                        return Some(Err(Error::Decode("More data columns than headers"
                            .to_string())));
                    }
                }
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
            other => other,
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
    fn test_struct_allow_reorder() {
        let rdr = Reader::from_string("b,a\n0,1\n2,3\n");
        let records = rdr.reorder(true).decode().collect::<Result<Vec<SimpleStruct>>>().unwrap();
        assert_eq!(records,
                   vec![SimpleStruct { a: 1, b: 0 }, SimpleStruct { a: 3, b: 2 }]);
    }

    #[test]
    fn test_struct_ignore_ascii_case() {
        let rdr = Reader::from_string("a,B\n0,1\n2,3\n");
        let records =
            rdr.ignore_ascii_case(true).decode().collect::<Result<Vec<SimpleStruct>>>().unwrap();
        assert_eq!(records,
                   vec![SimpleStruct { a: 0, b: 1 }, SimpleStruct { a: 2, b: 3 }]);
    }

    #[test]
    fn test_struct_reordered_headers() {
        let rdr = Reader::from_string("b,a\n0,1\n2,3\n");
        let err = rdr.decode().collect::<Result<Vec<SimpleStruct>>>().unwrap_err();
        assert_eq!(format!("{}", err),
                   "CSV decode error: Headers don't match field names".to_string());
    }

    #[test]
    fn test_struct_wrong_case() {
        let rdr = Reader::from_string("a,B\n0,1\n2,3\n");
        let err = rdr.decode().collect::<Result<Vec<SimpleStruct>>>().unwrap_err();
        assert_eq!(format!("{}", err),
                   "CSV decode error: Headers don't match field names".to_string());
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
                   "CSV decode error: The decodable type has 2 field names, but there are 1 \
                    headers"
                       .to_string());
    }

    #[test]
    fn test_struct_extra_header() {
        let rdr = Reader::from_string("a,b,c\n0,1\n");
        let err = rdr.decode().collect::<Result<Vec<SimpleStruct>>>().unwrap_err();
        assert_eq!(format!("{}", err),
                   "CSV decode error: The decodable type has 2 field names, but there are 3 \
                    headers"
                       .to_string());
    }

    #[test]
    fn test_struct_extra_data_column() {
        let rdr = Reader::from_string("a,b\n0,1,2\n");
        let err = rdr.decode().collect::<Result<Vec<SimpleStruct>>>().unwrap_err();
        assert_eq!(format!("{}", err),
                   "CSV decode error: More data columns than headers".to_string());
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
    fn test_tuple_of_structs_allow_reorder() {
        let rdr = Reader::from_string("b,a,a,b\n0,1,2,3\n\n4,5,6,7\n");
        let records = rdr.reorder(true)
            .decode()
            .collect::<Result<Vec<(SimpleStruct, SimpleStruct)>>>()
            .unwrap();
        assert_eq!(records,
                   vec![(SimpleStruct { a: 1, b: 0 }, SimpleStruct { a: 2, b: 3 }),
                        (SimpleStruct { a: 5, b: 4 }, SimpleStruct { a: 6, b: 7 })]);
    }

    #[test]
    fn test_tuple_of_structs_misnamed_headers() {
        let rdr = Reader::from_string("a,b,c,d\n0,1,2,3\n4,5,6,7\n");
        let err = rdr.decode().collect::<Result<Vec<(SimpleStruct, SimpleStruct)>>>().unwrap_err();
        assert_eq!(format!("{}", err),
                   "CSV decode error: Headers don't match field names".to_string());
    }
}
