use csv::{ByteString, Error, Result};
use rustc_serialize::Decoder;
use std::default::Default;

/// Decoder to extract field names from types that implement
/// `rustc_serialize::Decodable`.
#[derive(Debug)]
pub struct FieldNamesDecoder {
    field_names: Vec<ByteString>,
}

impl FieldNamesDecoder {
    /// Creates a new `FieldNamesDecoder` from a record of byte strings.
    pub fn new() -> FieldNamesDecoder {
        FieldNamesDecoder { field_names: Vec::new() }
    }

    pub fn into_field_names(self) -> Vec<ByteString> {
        self.field_names
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

    fn read_enum_variant<T, F>(&mut self, _: &[&str], mut f: F) -> Result<T>
        where F: FnMut(&mut Self, usize) -> Result<T>
    {
        f(self, 0)
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
        f(self, false)
    }

    fn read_seq<T, F>(&mut self, _: F) -> Result<T>
        where F: FnOnce(&mut Self, usize) -> Result<T>
    {
        unimplemented!();
    }

    fn read_seq_elt<T, F>(&mut self, _: usize, _: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        unimplemented!();
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
