use std::io::{Read, Write};

use super::program::Code;

use anyhow::*;

pub trait SerializableWithContext {
    fn serialize<W: Write>(&self, sink: &mut W, code: &Code) -> Result<()>;
    fn from_bytes<R: Read>(input: &mut R, code: &mut Code) -> Self;
}

pub trait Serializable {
    fn serialize<W: Write>(&self, sink: &mut W) -> Result<()>;
    fn from_bytes<R: Read>(input: &mut R) -> Self;
}

pub fn read_u8<R: Read>(reader: &mut R) -> u8 {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).expect("Problem reading u8 from data stream");
    u8::from_le_bytes(buf)
}

pub fn read_bool<R: Read>(reader: &mut R) -> bool {
    match read_u8(reader) {
        0 => false,
        1 => true,
        n => panic!("Problem reading boolfrom data stream: unrecognized value: {}", n),
    }
}

pub fn read_u16<R: Read>(reader: &mut R) -> u16 {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf).expect("Problem reading u16 from data stream");
    u16::from_le_bytes(buf)
}

pub fn read_u32<R: Read>(reader: &mut R) -> u32 {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).expect("Problem reading u32 from data stream");
    u32::from_le_bytes(buf)
}

pub fn read_i32<R: Read>(reader: &mut R) -> i32 {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).expect("Problem reading i32 from data stream");
    i32::from_le_bytes(buf)
}

pub fn read_utf8<R: Read>(reader: &mut R) -> String {
    let length = read_u32_as_usize(reader);
    let mut bytes = vec![0u8; length];
    for i in 0..length {
        bytes[i] = read_u8(reader);
    }
    String::from_utf8(bytes).expect(&format!(
        "Problem reading UTF-8 string of size {} \
                                              from data sink",
        length
    ))
}

pub fn read_u16_vector<R: Read>(reader: &mut R) -> Vec<u16> {
    let length = read_u16_as_usize(reader);
    let mut ints = vec![0u16; length];
    for i in 0..length {
        ints[i] = read_u16(reader);
    }
    ints
}

#[allow(dead_code)]
pub fn read_u32_vector<R: Read>(reader: &mut R) -> Vec<u32> {
    let length = read_u16_as_usize(reader);
    let mut ints = vec![0u32; length];
    for i in 0..length {
        ints[i] = read_u32(reader);
    }
    ints
}

// Reads u16 and converts it to usize, for compatibility between Rust types and Feeny bytecode.
pub fn read_u16_as_usize<R: Read>(reader: &mut R) -> usize {
    read_u16(reader) as usize
}

// Reads u32 and converts it to usize, for compatibility between Rust types and Feeny bytecode.
pub fn read_u32_as_usize<R: Read>(reader: &mut R) -> usize {
    read_u32(reader) as usize
}

pub fn write_u8<W: Write>(writer: &mut W, value: u8) -> Result<()> {
    writer.write(&[value])?; //.expect(&format!("Problem writing u8 to data sink: {}", value)?;
    Ok(())
}

pub fn write_bool<W: Write>(writer: &mut W, value: bool) -> Result<()> {
    let byte = if value { 1 } else { 0 };
    writer.write(&[byte])?; //.expect(&format!("Problem writing bool to data sink: {}", value))?;
    Ok(())
}

pub fn write_u16<W: Write>(writer: &mut W, value: u16) -> Result<()> {
    let buf = value.to_le_bytes();
    writer.write(&buf)?; //.expect(&format!("Problem writing u16 to data sink: {}", value))?;
    Ok(())
}

pub fn write_u32<W: Write>(writer: &mut W, value: u32) -> Result<()> {
    let buf = value.to_le_bytes();
    writer.write(&buf)?; //.expect(&format!("Problem writing u32 to data sink: {}", value));
    Ok(())
}

pub fn write_i32<W: Write>(writer: &mut W, value: i32) -> Result<()> {
    let buf = value.to_le_bytes();
    writer.write(&buf)?; //.expect(&format!("Problem writing i32 to data sink: {}", value));
    Ok(())
}

pub fn write_utf8<R: Write>(writer: &mut R, string: &str) -> Result<()> {
    let byte_vector: Vec<u8> = string.bytes().collect();
    let bytes = byte_vector.as_slice();
    write_usize_as_u32(writer, bytes.len())?;
    writer.write(bytes)?;
    Ok(())
    //.expect(&format!("Problem writing UTF-8 string to data sink: {}", string));
}

pub fn write_u16_vector<R: Write>(writer: &mut R, vector: &Vec<u16>) -> Result<()> {
    write_usize_as_u16(writer, vector.len())?;
    for e in vector {
        write_u16(writer, *e)?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn write_u32_vector<R: Write>(writer: &mut R, vector: &Vec<u32>) -> Result<()> {
    write_usize_as_u16(writer, vector.len())?;
    for e in vector {
        write_u32(writer, *e)?;
    }
    Ok(())
}

pub fn write_usize_as_u16<R: Write>(writer: &mut R, value: usize) -> Result<()> {
    assert!(value <= 65_535usize); // Max u16 value.
    write_u16(writer, value as u16)
}

pub fn write_usize_as_u32<R: Write>(writer: &mut R, value: usize) -> Result<()> {
    assert!(value <= 4_294_967_295usize); // Max u16 value.
    write_u32(writer, value as u32)
}
