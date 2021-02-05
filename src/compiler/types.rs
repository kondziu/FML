use std::io::{Read, Write};

use anyhow::*;

use super::serializable;
use super::serializable::Serializable;

#[derive(PartialEq,Debug,Copy,Clone)] pub struct Arity(u8);
#[derive(PartialEq,Debug,Copy,Clone)] pub struct Size(u16);
#[derive(PartialEq,Debug,Copy,Clone)] pub struct Address(u32);
#[derive(PartialEq,Debug,Copy,Clone)] pub struct ConstantPoolIndex(u16);
#[derive(PartialEq,Debug,Copy,Clone)] pub struct LocalFrameIndex(u16);
#[derive(PartialEq,Debug,Copy,Clone)] pub struct AddressRange { start: Address, length: usize }

impl Arity {
    pub fn new(value: u8)  -> Arity {
        Arity(value)
    }
}
impl Size {
    #[allow(dead_code)] pub fn new(value: u16) -> Size {
        Size(value)
    }
}
impl LocalFrameIndex {
    #[allow(dead_code)] pub fn new(value: u16) -> LocalFrameIndex   {
        LocalFrameIndex(value)
    }
}
impl ConstantPoolIndex {
    pub fn new(value: u16) -> ConstantPoolIndex {
        ConstantPoolIndex(value)
    }
}

impl Arity {
    pub fn from_usize(value: usize) -> Arity {
        assert!(value <= 255usize);
        Arity(value as u8)
    }
    pub fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Size {
    pub fn from_usize(value: usize) -> Size {
        assert!(value <= 65535usize);
        Size(value as u16)
    }
    pub fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl LocalFrameIndex {
    pub fn from_usize(value: usize) -> LocalFrameIndex {
        assert!(value <= 65535usize);
        LocalFrameIndex(value as u16)
    }
}

impl ConstantPoolIndex {
    pub fn from_usize(value: usize) -> ConstantPoolIndex {
        assert!(value <= 65535usize);
        ConstantPoolIndex(value as u16)
    }
}

impl AddressRange {
    pub fn new (start: Address, length: usize) -> Self {
        AddressRange { start, length }
    }

    #[allow(dead_code)]
    pub fn from (start: usize, length: usize) -> Self {
        AddressRange { start: Address::from_usize(start), length }
    }

    pub fn from_addresses (start: Address, end: Address) -> Self {
        AddressRange { start, length: end.value_usize() - start.value_usize() + 1 }
    }

    pub fn start(&self) -> &Address {
       &self.start
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

impl ConstantPoolIndex {
    pub fn read_cpi_vector<R: Read>(input: &mut R) -> Vec<ConstantPoolIndex> {
        println!("ConstantPoolIndex::read_cpi_vector");
        serializable::read_u16_vector(input)
            .into_iter()
            .map(ConstantPoolIndex::new)
            .collect()
    }

    pub fn write_cpi_vector<R: Write>(sink: &mut R, vector: &Vec<ConstantPoolIndex>) -> anyhow::Result<()> {
        let vector_of_u16s: Vec<u16> = vector.iter().map(|cpi| cpi.0).collect();
        serializable::write_u16_vector(sink, &vector_of_u16s)
    }
}

impl ConstantPoolIndex  { pub fn value(&self) -> u16 { self.0 } }
impl LocalFrameIndex    { pub fn value(&self) -> u16 { self.0 } }
impl Size               { pub fn value(&self) -> u16 { self.0 } }
impl Arity              { pub fn value(&self) -> u8  { self.0 } }

impl Address {
    #[allow(dead_code)]
    pub fn from_u32(value: u32) -> Address {
        Address(value)
    }
    pub fn from_usize(value: usize) -> Address {
        assert!(value <= 4_294_967_295usize);
        Address(value as u32)
    }
    pub fn value_u32(&self) -> u32 {
        self.0
    }
    pub fn value_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Serializable for Arity {

    fn serialize<W: Write> (&self, sink: &mut W) -> anyhow::Result<()> {
        serializable::write_u8(sink, self.0)
    }

    fn from_bytes<R: Read>(input: &mut R) -> Self {
        println!("Arity::from_bytes");
        Arity(serializable::read_u8(input))
    }
}

impl Arity {
    #[allow(dead_code)]
    pub fn serialize_plus_one<W: Write> (&self, sink: &mut W) -> Result<()> {
        assert!(self.0 < 255u8);
        serializable::write_u8(sink, self.0 + 1)
    }
    #[allow(dead_code)]
    pub fn from_bytes_minus_one<R: Read>(input: &mut R) -> Self {
        println!("Arity::from_bytes");
        let value = serializable::read_u8(input);
        assert!(value > 0);
        Arity(value - 1)
    }
}

impl Serializable for Size {

    fn serialize<W: Write> (&self, sink: &mut W) -> anyhow::Result<()> {
        serializable::write_u16(sink, self.0)
    }

    fn from_bytes<R: Read>(input: &mut R) -> Self {
        println!("Size::from_bytes");
        Size(serializable::read_u16(input))
    }
}

impl Serializable for Address {
    fn serialize<W: Write> (&self, sink: &mut W) -> anyhow::Result<()> {
        serializable::write_u32(sink, self.0)
    }
    fn from_bytes<R: Read>(input: &mut R) -> Self {
        println!("Address::from_bytes");
        Address(serializable::read_u32(input))
    }
}

impl Serializable for ConstantPoolIndex {
    fn serialize<W: Write> (&self, sink: &mut W) -> anyhow::Result<()> {
        serializable::write_u16(sink, self.0)
    }
    fn from_bytes<R: Read>(input: &mut R) -> Self {
        println!("ConstantPoolIndex::from_bytes");
        ConstantPoolIndex(serializable::read_u16(input))
    }
}

impl Serializable for LocalFrameIndex {
    fn serialize<W: Write> (&self, sink: &mut W) -> anyhow::Result<()> {
        serializable::write_u16(sink, self.0)
    }
    fn from_bytes<R: Read>(input: &mut R) -> Self {
        println!("LocalFrameIndex::from_bytes");
        LocalFrameIndex(serializable::read_u16(input))
    }
}