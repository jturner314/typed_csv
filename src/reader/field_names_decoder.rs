use std::default::Default;

use csv::{ByteString, Error, Result};
use rustc_serialize::Decoder;

/// Decoder to extract field names from types that implement
/// `rustc_serialize::Decodable`.
#[derive(Debug)]
pub struct FieldNamesDecoder {
    stack: Vec<ByteString>,
    popped: usize,
    field_names: Vec<ByteString>,
}

impl FieldNamesDecoder {
    /// Creates a new `FieldNamesDecoder` from a record of byte strings.
    pub fn new(mut bytes: Vec<ByteString>) -> FieldNamesDecoder {
        bytes.reverse();
        FieldNamesDecoder {
            stack: bytes,
            popped: 0,
            field_names: Vec::new(),
        }
    }

    pub fn into_field_names(self) -> Vec<ByteString> {
        self.field_names
    }

    fn len(&self) -> usize {
        self.stack.len()
    }
}

impl FieldNamesDecoder {
    fn pop(&mut self) -> Result<ByteString> {
        self.popped += 1;
        match self.stack.pop() {
            None => {
                self.err(format!("Expected a record with length at least {}, but got a record \
                                  with length {}.",
                                 self.popped,
                                 self.popped - 1))
            }
            Some(bytes) => Ok(bytes),
        }
    }

    fn pop_string(&mut self) -> Result<String> {
        String::from_utf8(try!(self.pop())).map_err(|bytes| {
            Error::Decode(format!("Could not convert bytes '{:?}' to UTF-8.", bytes))
        })
    }

    fn push(&mut self, s: ByteString) {
        self.stack.push(s);
    }

    fn push_string(&mut self, s: String) {
        self.push(s.into_bytes());
    }

    fn err<'a, T, S>(&self, msg: S) -> Result<T>
        where S: Into<String>
    {
        Err(Error::Decode(msg.into()))
    }
}

impl Decoder for FieldNamesDecoder {
    type Error = Error;

    fn error(&mut self, err: &str) -> Error {
        Error::Decode(err.into())
    }

    fn read_nil(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn read_usize(&mut self) -> Result<usize> {
        Ok(Default::default())
    }

    fn read_u64(&mut self) -> Result<u64> {
        Ok(Default::default())
    }

    fn read_u32(&mut self) -> Result<u32> {
        Ok(Default::default())
    }

    fn read_u16(&mut self) -> Result<u16> {
        Ok(Default::default())
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(Default::default())
    }

    fn read_isize(&mut self) -> Result<isize> {
        Ok(Default::default())
    }

    fn read_i64(&mut self) -> Result<i64> {
        Ok(Default::default())
    }

    fn read_i32(&mut self) -> Result<i32> {
        Ok(Default::default())
    }

    fn read_i16(&mut self) -> Result<i16> {
        Ok(Default::default())
    }

    fn read_i8(&mut self) -> Result<i8> {
        Ok(Default::default())
    }

    fn read_bool(&mut self) -> Result<bool> {
        Ok(Default::default())
    }

    fn read_f64(&mut self) -> Result<f64> {
        Ok(Default::default())
    }

    fn read_f32(&mut self) -> Result<f32> {
        Ok(Default::default())
    }

    fn read_char(&mut self) -> Result<char> {
        Ok(Default::default())
    }

    fn read_str(&mut self) -> Result<String> {
        Ok(Default::default())
    }

    fn read_enum<T, F>(&mut self, _: &str, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_enum_variant<T, F>(&mut self, names: &[&str], mut f: F) -> Result<T>
        where F: FnMut(&mut Self, usize) -> Result<T>
    {
        for i in 0..names.len() {
            let cur = self.pop_string()?;
            self.push_string(cur.clone());
            match f(self, i) {
                Ok(v) => return Ok(v),
                Err(_) => {
                    self.push_string(cur);
                }
            }
        }
        self.err(format!("Could not load value into any variant in {:?}", names))
    }

    fn read_enum_variant_arg<T, F>(&mut self, _: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F) -> Result<T>
        where F: FnMut(&mut Self, usize) -> Result<T>
    {
        self.read_enum_variant(names, f)
    }

    fn read_enum_struct_variant_field<T, F>(&mut self, _: &str, f_idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        self.read_enum_variant_arg(f_idx, f)
    }

    fn read_struct<T, F>(&mut self, _: &str, _: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_struct_field<T, F>(&mut self, f_name: &str, f_idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        // Heuristic to ignore field names in tuple structs.
        // See https://github.com/rust-lang/rust/issues/15659
        // and https://github.com/rust-lang/rust/issues/17158
        if f_name != format!("_field{}", f_idx) {
            self.field_names.push(f_name.to_owned().into_bytes());
        }
        f(self)
    }

    fn read_tuple<T, F>(&mut self, _: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_tuple_arg<T, F>(&mut self, _: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_tuple_struct<T, F>(&mut self, _: &str, _: usize, _: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        unimplemented!()
    }

    fn read_tuple_struct_arg<T, F>(&mut self, _: usize, _: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        unimplemented!()
    }

    fn read_option<T, F>(&mut self, mut f: F) -> Result<T>
        where F: FnMut(&mut Self, bool) -> Result<T>
    {
        let s = try!(self.pop_string());
        if s.is_empty() {
            f(self, false)
        } else {
            self.push_string(s);
            match f(self, true) {
                Ok(v) => Ok(v),
                Err(_) => f(self, false),
            }
        }
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T>
        where F: FnOnce(&mut Self, usize) -> Result<T>
    {
        let len = self.len();
        f(self, len)
    }

    fn read_seq_elt<T, F>(&mut self, _: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, _: F) -> Result<T>
        where F: FnOnce(&mut Self, usize) -> Result<T>
    {
        unimplemented!()
    }

    fn read_map_elt_key<T, F>(&mut self, _: usize, _: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        unimplemented!()
    }

    fn read_map_elt_val<T, F>(&mut self, _: usize, _: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        unimplemented!()
    }
}
