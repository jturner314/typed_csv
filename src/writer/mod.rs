mod field_names_encoder;

use self::field_names_encoder::FieldNamesEncoder;

use csv::{self, Result};
use rustc_serialize::Encodable;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::marker::PhantomData;
use std::path::Path;

/// A CSV writer that automatically writes the headers.
///
/// This writer provides a convenient interface for encoding CSV data. While
/// creating CSV data is much easier than parsing it, having a writer can be
/// convenient because it can handle quoting for you automatically. Moreover,
/// this particular writer supports [`rustc_serialize::Encodable`][Encodable]
/// types, which makes it easy to write your custom types as CSV records and
/// automatically generate headers.
///
/// All CSV data produced by this writer, with default options, conforms with
/// [RFC 4180](http://tools.ietf.org/html/rfc4180).
///
/// One slight deviation is that records with a single empty field are always
/// encoded as `""`. This ensures that the record is not skipped since some
/// CSV parsers will ignore consecutive record terminators (like the one in
/// this crate).
///
/// If you don't care want the writer to automatically write the header row,
/// use the [`csv`](https://github.com/BurntSushi/rust-csv) crate instead.
///
/// # Example
///
/// Here's an example that encodes a zoo of animals:
///
/// ```rust
/// extern crate rustc_serialize;
/// # extern crate typed_csv;
/// # fn main() {
///
/// #[derive(RustcEncodable)]
/// struct Record {
///     count: usize,
///     animal: &'static str,
///     description: &'static str,
/// }
///
/// let records = vec![
///     Record { count: 7, animal: "penguin", description: "happy" },
///     Record { count: 10, animal: "cheetah", description: "fast" },
///     Record { count: 4, animal: "armadillo", description: "armored" },
///     Record { count: 9, animal: "platypus", description: "unique" },
/// ];
///
/// let mut wtr = typed_csv::Writer::from_memory();
/// for record in records.into_iter() {
///     wtr.encode(record).unwrap();
/// }
///
/// assert_eq!(wtr.as_string(), "\
/// count,animal,description
/// 7,penguin,happy
/// 10,cheetah,fast
/// 4,armadillo,armored
/// 9,platypus,unique
/// ");
/// # }
/// ```
///
/// [Encodable]: https://doc.rust-lang.org/rustc-serialize/rustc_serialize/trait.Encodable.html
pub struct Writer<W: Write, E: Encodable> {
    csv: csv::Writer<W>,
    first_row: bool,
    record_type: PhantomData<E>,
}

impl<E: Encodable> Writer<File, E> {
    /// Creates a new typed CSV writer that writes to the file path given.
    ///
    /// The file is created if it does not already exist and is truncated
    /// otherwise.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Writer<File, E>> {
        Ok(Self::from_csv_writer(csv::Writer::from_file(path)?))
    }
}

impl<W: Write, E: Encodable> Writer<W, E> {
    /// Creates a new typed CSV writer that writes to the `io::Write` given.
    ///
    /// Note that the writer is buffered for you automatically.
    pub fn from_writer(w: W) -> Writer<W, E> {
        Self::from_csv_writer(csv::Writer::from_writer(w))
    }

    /// Creates a new typed CSV writer that writes to the CSV writer given.
    ///
    /// This lets you specify options to the underlying CSV writer (e.g. to use
    /// a different delimiter).
    pub fn from_csv_writer(w: csv::Writer<W>) -> Writer<W, E> {
        Writer {
            csv: w,
            first_row: true,
            record_type: PhantomData,
        }
    }

    /// Creates a new typed CSV writer that writes to the buffer given.
    ///
    /// This lets you specify your own buffered writer (e.g., use a different
    /// capacity). All other constructors wrap the writer given in a buffer
    /// with default capacity.
    pub fn from_buffer(buf: BufWriter<W>) -> Writer<W, E> {
        Self::from_csv_writer(csv::Writer::from_buffer(buf))
    }
}

impl<E: Encodable> Writer<Vec<u8>, E> {
    /// Creates a new CSV writer that writes to an in memory buffer. At any
    /// time, `as_string` or `as_bytes` can be called to retrieve the
    /// cumulative CSV data.
    pub fn from_memory() -> Writer<Vec<u8>, E> {
        Self::from_csv_writer(csv::Writer::from_memory())
    }

    /// Returns the written CSV data as a string.
    pub fn as_string<'r>(&'r mut self) -> &'r str {
        self.csv.as_string()
    }

    /// Returns the encoded CSV data as raw bytes.
    pub fn as_bytes<'r>(&'r mut self) -> &'r [u8] {
        self.csv.as_bytes()
    }

    /// Convert the Writer into a string of written CSV data
    pub fn into_string(self) -> String {
        self.csv.into_string()
    }

    /// Convert the Writer into a vector of encoded CSV bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.csv.into_bytes()
    }
}

impl<W: Write, E: Encodable> Writer<W, E> {
    /// Writes a record by encoding any `Encodable` value.
    ///
    /// When the first record is encoded, the headers (the field names in the
    /// struct) are written prior to encoding the record.
    ///
    /// The type that is being encoded into should correspond to *one full CSV
    /// record*. This can be a single struct, or arbitrarily nested tuples,
    /// arrays, Vecs, and structs, as long as all scalar types (integers,
    /// floats, characters, strings, collections containing one scalar, and
    /// enums with 0 or 1 scalar arguments) are fields in structs. Enums with
    /// zero arguments encode to their name, while enums of one argument encode
    /// to their constituent value. `Option` types are also supported. (`None`
    /// encodes to an empty field.)
    ///
    /// Note that single-element tuple structs (the newtype pattern) are
    /// supported. Unfortunately, to provide this functionality, a heuristic is
    /// necessary to differentiate field names in normal structs from those in
    /// tuple structs. As a result, field names in normal structs should not be
    /// of the form `_field{}` where `{}` is its position in the struct.
    ///
    /// # Example
    ///
    /// This example encodes a zoo animals with may not have a description.
    ///
    /// ```rust
    /// extern crate rustc_serialize;
    /// # extern crate typed_csv;
    /// # fn main() {
    ///
    /// #[derive(RustcEncodable)]
    /// struct Count(usize);
    ///
    /// #[derive(RustcEncodable)]
    /// enum Group {
    ///     Bird,
    ///     Mammal,
    /// }
    ///
    /// #[derive(RustcEncodable)]
    /// struct Part1 {
    ///     count: Count,
    ///     animal: &'static str,
    /// }
    ///
    /// #[derive(RustcEncodable)]
    /// struct Part2 {
    ///     group: Group,
    ///     description: Option<&'static str>,
    /// }
    ///
    /// let records = vec![
    ///     (
    ///         Part1 { count: Count(7), animal: "penguin" },
    ///         Part2 { group: Group::Bird, description: Some("happy") },
    ///     ),
    ///     (
    ///         Part1 { count: Count(10), animal: "cheetah" },
    ///         Part2 { group: Group::Mammal, description: Some("fast") },
    ///     ),
    ///     (
    ///         Part1 { count: Count(4), animal: "armadillo" },
    ///         Part2 { group: Group::Mammal, description: Some("armored") },
    ///     ),
    ///     (
    ///         Part1 { count: Count(9), animal: "platypus" },
    ///         Part2 { group: Group::Mammal, description: None },
    ///     ),
    /// ];
    ///
    /// let mut wtr = typed_csv::Writer::from_memory();
    /// for record in records.into_iter() {
    ///     wtr.encode(record).unwrap();
    /// }
    ///
    /// assert_eq!(wtr.as_string(), "\
    /// count,animal,group,description
    /// 7,penguin,Bird,happy
    /// 10,cheetah,Mammal,fast
    /// 4,armadillo,Mammal,armored
    /// 9,platypus,Mammal,
    /// ");
    /// # }
    /// ```
    pub fn encode(&mut self, row: E) -> csv::Result<()> {
        // Write headers if this is the first row.
        if self.first_row {
            let mut field_names_encoder = FieldNamesEncoder::new();
            row.encode(&mut field_names_encoder)?;
            self.csv.write(field_names_encoder.into_field_names().into_iter())?;
            self.first_row = false;
        }
        // Write row.
        let mut erecord = csv::Encoded::new();
        row.encode(&mut erecord)?;
        self.csv.write(erecord.unwrap().into_iter())
    }

    /// Flushes the underlying buffer.
    pub fn flush(&mut self) -> Result<()> {
        self.csv.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::Writer;

    #[derive(RustcEncodable)]
    struct SimpleStruct {
        a: usize,
        b: usize,
    }

    #[test]
    fn test_struct() {
        let mut w = Writer::from_memory();
        let s1 = SimpleStruct { a: 0, b: 1 };
        w.encode(s1).unwrap();
        let s2 = SimpleStruct { a: 3, b: 4 };
        w.encode(s2).unwrap();
        assert_eq!(w.as_string(), "a,b\n0,1\n3,4\n");
    }

    #[test]
    fn test_tuple_of_structs() {
        let mut w = Writer::from_memory();
        let s1 = SimpleStruct { a: 0, b: 1 };
        let s2 = SimpleStruct { a: 2, b: 3 };
        w.encode((s1, s2)).unwrap();
        let s3 = SimpleStruct { a: 4, b: 5 };
        let s4 = SimpleStruct { a: 6, b: 7 };
        w.encode((s3, s4)).unwrap();
        assert_eq!(w.as_string(), "a,b,a,b\n0,1,2,3\n4,5,6,7\n");
    }

    #[test]
    fn test_array_of_structs() {
        let mut w = Writer::from_memory();
        let s1 = SimpleStruct { a: 0, b: 1 };
        let s2 = SimpleStruct { a: 2, b: 3 };
        w.encode([s1, s2]).unwrap();
        let s3 = SimpleStruct { a: 4, b: 5 };
        let s4 = SimpleStruct { a: 6, b: 7 };
        w.encode([s3, s4]).unwrap();
        assert_eq!(w.as_string(), "a,b,a,b\n0,1,2,3\n4,5,6,7\n");
    }

    #[test]
    fn test_vec_of_structs() {
        let mut w = Writer::from_memory();
        let s1 = SimpleStruct { a: 0, b: 1 };
        let s2 = SimpleStruct { a: 2, b: 3 };
        w.encode(vec![s1, s2]).unwrap();
        let s3 = SimpleStruct { a: 4, b: 5 };
        let s4 = SimpleStruct { a: 6, b: 7 };
        w.encode(vec![s3, s4]).unwrap();
        assert_eq!(w.as_string(), "a,b,a,b\n0,1,2,3\n4,5,6,7\n");
    }

    #[test]
    fn test_nested_tuples_of_structs() {
        let mut w = Writer::from_memory();
        w.encode((SimpleStruct { a: 0, b: 1 },
                     (SimpleStruct { a: 2, b: 3 }),
                     (SimpleStruct { a: 4, b: 5 }, (SimpleStruct { a: 6, b: 7 }))))
            .unwrap();
        w.encode((SimpleStruct { a: 8, b: 9 },
                     (SimpleStruct { a: 10, b: 11 }),
                     (SimpleStruct { a: 12, b: 13 }, (SimpleStruct { a: 14, b: 15 }))))
            .unwrap();
        assert_eq!(w.as_string(),
                   "a,b,a,b,a,b,a,b\n0,1,2,3,4,5,6,7\n8,9,10,11,12,13,14,15\n");
    }

    #[derive(RustcEncodable)]
    struct StructWithLengthOneSeqs {
        a: [usize; 1],
        b: Vec<usize>,
        c: (usize),
    }

    #[test]
    fn test_struct_with_len_one_seqs() {
        let mut w = Writer::from_memory();
        let s1 = StructWithLengthOneSeqs {
            a: [0],
            b: vec![1],
            c: (2),
        };
        w.encode(s1).unwrap();
        let s2 = StructWithLengthOneSeqs {
            a: [3],
            b: vec![4],
            c: (5),
        };
        w.encode(s2).unwrap();
        assert_eq!(w.as_string(), "a,b,c\n0,1,2\n3,4,5\n");
    }

    #[derive(RustcEncodable)]
    struct StructOfStruct {
        p: SimpleStruct,
        q: (usize, usize),
    }

    #[should_panic]
    #[test]
    fn test_struct_of_struct() {
        let mut w = Writer::from_memory();
        let s = StructOfStruct {
            p: SimpleStruct { a: 0, b: 1 },
            q: (2, 3),
        };
        w.encode(s).unwrap();
    }

    #[derive(RustcEncodable)]
    struct StructWithLongerSeq {
        a: [usize; 2],
    }

    #[should_panic]
    #[test]
    fn test_struct_with_longer_seq() {
        let mut w = Writer::from_memory();
        let s = StructWithLongerSeq { a: [0, 1] };
        w.encode(s).unwrap();
    }

    #[should_panic]
    #[test]
    fn test_vec() {
        let mut w = Writer::from_memory();
        let array = vec![0, 1];
        w.encode(array).unwrap();
    }
}
