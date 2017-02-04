use csv::{ByteString, Error, Result};
use rustc_serialize::Encoder;

/// Encoder to extract field names from types that implement
/// `rustc_serialize::Encodable`.
#[derive(Debug)]
pub struct FieldNamesEncoder {
    record: Vec<ByteString>,
}

impl FieldNamesEncoder {
    /// Creates a new `FieldNamesEncoder`. The value returned can be passed to
    /// `Encodable::encode`.
    pub fn new() -> FieldNamesEncoder {
        FieldNamesEncoder { record: vec![] }
    }

    /// Once a record has been encoded into this value, `into_field_names` can
    /// be used to access the raw field names.
    pub fn into_field_names(self) -> Vec<ByteString> {
        self.record
    }

    fn push_bytes<'a, S>(&mut self, s: S) -> Result<()>
        where S: Into<Vec<u8>>
    {
        self.record.push(s.into());
        Ok(())
    }

    fn push_string<'a, S>(&mut self, s: S) -> Result<()>
        where S: Into<String>
    {
        self.push_bytes(s.into().into_bytes())
    }

    fn push_to_string<T: ToString>(&mut self, t: T) -> Result<()> {
        self.push_string(t.to_string())
    }
}

impl Encoder for FieldNamesEncoder {
    type Error = Error;

    fn emit_nil(&mut self) -> Result<()> {
        unimplemented!()
    }
    fn emit_usize(&mut self, _: usize) -> Result<()> {
        Ok(())
    }
    fn emit_u64(&mut self, _: u64) -> Result<()> {
        Ok(())
    }
    fn emit_u32(&mut self, _: u32) -> Result<()> {
        Ok(())
    }
    fn emit_u16(&mut self, _: u16) -> Result<()> {
        Ok(())
    }
    fn emit_u8(&mut self, _: u8) -> Result<()> {
        Ok(())
    }
    fn emit_isize(&mut self, _: isize) -> Result<()> {
        Ok(())
    }
    fn emit_i64(&mut self, _: i64) -> Result<()> {
        Ok(())
    }
    fn emit_i32(&mut self, _: i32) -> Result<()> {
        Ok(())
    }
    fn emit_i16(&mut self, _: i16) -> Result<()> {
        Ok(())
    }
    fn emit_i8(&mut self, _: i8) -> Result<()> {
        Ok(())
    }
    fn emit_bool(&mut self, _: bool) -> Result<()> {
        Ok(())
    }
    fn emit_f64(&mut self, _: f64) -> Result<()> {
        Ok(())
    }
    fn emit_f32(&mut self, _: f32) -> Result<()> {
        Ok(())
    }
    fn emit_char(&mut self, _: char) -> Result<()> {
        Ok(())
    }
    fn emit_str(&mut self, _: &str) -> Result<()> {
        Ok(())
    }
    fn emit_enum<F>(&mut self, _: &str, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        Ok(())
    }
    fn emit_enum_variant<F>(&mut self, _: &str, _: usize, _: usize, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        Ok(())
    }
    fn emit_enum_variant_arg<F>(&mut self, _: usize, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        Ok(())
    }
    fn emit_enum_struct_variant<F>(&mut self, _: &str, _: usize, _: usize, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        Ok(())
    }
    fn emit_enum_struct_variant_field<F>(&mut self, _: &str, _: usize, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        Ok(())
    }
    fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        f(self)
    }
    fn emit_struct_field<F>(&mut self, f_name: &str, f_idx: usize, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        // Heuristic to ignore field names in tuple structs.
        // See https://github.com/rust-lang/rust/issues/19756
        if f_name != format!("_field{}", f_idx) {
            self.push_to_string(f_name)?;
        }
        Ok(())
    }
    fn emit_tuple<F>(&mut self, _: usize, f: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        f(self)
    }
    fn emit_tuple_arg<F>(&mut self, _: usize, f: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        f(self)
    }
    fn emit_tuple_struct<F>(&mut self, _: &str, _: usize, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        unimplemented!()
    }
    fn emit_tuple_struct_arg<F>(&mut self, _: usize, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        unimplemented!()
    }
    fn emit_option<F>(&mut self, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        Ok(())
    }
    fn emit_option_none(&mut self) -> Result<()> {
        Ok(())
    }
    fn emit_option_some<F>(&mut self, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        Ok(())
    }
    fn emit_seq<F>(&mut self, _: usize, f: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        f(self)
    }
    fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        f(self)
    }
    fn emit_map<F>(&mut self, _: usize, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        unimplemented!()
    }
    fn emit_map_elt_key<F>(&mut self, _: usize, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        unimplemented!()
    }
    fn emit_map_elt_val<F>(&mut self, _: usize, _: F) -> Result<()>
        where F: FnOnce(&mut Self) -> Result<()>
    {
        unimplemented!()
    }
}
