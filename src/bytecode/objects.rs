use std::io::{Read, Write};
use std::collections::HashMap;

use super::types::*;
use super::program::Code;
use super::bytecode::OpCode;

use super::serializable;
use super::serializable::*;
use serde::__private::Formatter;
use crate::bytecode::objects::HeapObject::Array;
use std::ops::Deref;

#[derive(PartialEq,Debug,Clone)]
pub enum ProgramObject {
    /**
     * Represents a 32 bit integer. Used by the `Literal` instruction.
     *
     * Serialized with tag `0x00`.
     */
    Integer(i32),

    /**
     * Represents a boolean. Used by the `Literal` instruction.
     *
     * Serialized with tag `0x06`.
     */
    Boolean(bool),

    /**
     * Represents the unit value. Used by the `Literal` instruction.
     *
     * Serialized with tag `0x01`.
     */
    Null,

    /**
     * Represents a character string. Strings are used to:
     *   - represent the names of functions, slots, methods, and labels,
     *   - as format strings in the `Print`.
     *
     * Serialized with tag `0x02`.
     */
    String(String),

    /**
     * Represents one of two things:
     *   - a field member (aka slot) of an object when it is referred to from a `Class` object, or
     *   - a global variable when referred to from the list of Global slots.
     *
     * Contains an index that refers to a `ProgramObject::String` object. The string object
     * represents this slot's name.
     *
     * Serialized with tag `0x04`.
     */
    Slot { name: ConstantPoolIndex },

    /**
     * Represents one of two things:
     *   - a method member of an object, or
     *   - a global function.
     *
     * Contains:
     *   - `name`: an index that refers to a `ProgramObject::String` object, which represents this
     *             method's name,
     *   - `arguments`: the number of arguments this function takes,
     *   - `locals`: the number of local variables defined in this method,
     *   - `code`: a vector containing all the instructions in this method.
     *
     * Serialized with tag `0x03`.
     */
    Method {
        name: ConstantPoolIndex,
        arguments: Arity,
        locals: Size,
        code: AddressRange,
    },

    /**
     * Represents an object structure consisting of field (aka slot) and method members for each
     * type of object created by `object`.
     *
     * It contains a vector containing indices to all the slots in the objects. Each index refers to
     * either:
     *   - a `ProgramObject::Slot` object representing a member field, or
     *   - a `ProgramObject::Method` object representing a member method.
     *
     * Serialized with tag `0x05`.
     */
    Class(Vec<ConstantPoolIndex>),
}

impl ProgramObject {
    pub fn is_literal(&self) -> bool {
        match self {
            ProgramObject::Null => true,
            ProgramObject::Boolean(_) => true,
            ProgramObject::Integer(_) => true,
            _ => false,
        }
    }

    pub fn as_str(&self) -> anyhow::Result<&str> {
        match self {
            ProgramObject::String(string) => Ok(string),
            _ => anyhow::bail!("Expecting a program object representing a String, found `{}`", self)
        }
    }

    pub fn as_class_definition(&self) -> anyhow::Result<&Vec<ConstantPoolIndex>> {
        match self {
            ProgramObject::Class(members) => Ok(members),
            _ => anyhow::bail!("Expecting a program object representing a Class, found `{}`", self)
        }
    }

    pub fn is_slot(&self) -> bool {
        match self {
            ProgramObject::Slot { .. } => true,
            _ => false,
        }
    }

    pub fn is_method(&self) -> bool {
        match self {
            ProgramObject::Method { .. } => true,
            _ => false,
        }
    }

    pub fn as_slot_index(&self) -> anyhow::Result<&ConstantPoolIndex> {
        match self {
            ProgramObject::Slot { name } => Ok(name),
            _ => anyhow::bail!("Expecting a program object representing a Slot, found `{}`", self)
        }
    }

    // pub fn as_method(&self) -> bool {
    //     match self {
    //         ProgramObject::Method { name, arguments, locals, code } => true,
    //         _ => false,
    //     }
    // }
}

impl ProgramObject {
    fn tag(&self) -> u8 {
        use ProgramObject::*;
        match &self {
            Integer(_)                                         => 0x00,
            Null                                               => 0x01,
            String(_)                                          => 0x02,
            Method {name: _, arguments: _, locals: _, code: _} => 0x03,
            Slot {name:_}                                      => 0x04,
            Class(_)                                           => 0x05,
            Boolean(_)                                         => 0x06,
        }
    }
}

impl std::fmt::Display for ProgramObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramObject::Integer(n) => write!(f, "{}", n),
            ProgramObject::Boolean(b) => write!(f, "{}", b),
            ProgramObject::Null => write!(f, "null"),
            ProgramObject::String(s) => write!(f, "\"{}\"", s),
            ProgramObject::Slot { name } => write!(f, "slot {}", name),
            ProgramObject::Method { name, locals, arguments, code } => {
                write!(f, "method {} args:{} locals:{} {}", name, arguments, locals, code)
            },
            ProgramObject::Class(members) => {
                let members = members.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                write!(f, "class {}", members)
            },
        }
    }
}

impl SerializableWithContext for ProgramObject {
    fn serialize<W: Write>(&self, sink: &mut W, code: &Code) -> anyhow::Result<()> {
        serializable::write_u8(sink, self.tag())?;
        use ProgramObject::*;
        match &self {
            Null        => Ok(()),
            Integer(n)  => serializable::write_i32(sink, *n),
            Boolean(b)  => serializable::write_bool(sink, *b),
            String(s)   => serializable::write_utf8(sink, s),
            Class(v)    => ConstantPoolIndex::write_cpi_vector(sink, v),
            Slot {name} => name.serialize(sink),

            Method {name, arguments, locals, code: range} => {
                name.serialize(sink)?;
                arguments.serialize(sink)?;
                locals.serialize(sink)?;
                OpCode::write_opcode_vector(sink, &code.addresses_to_code_vector(range))
            }
        }
    }

    fn from_bytes<R: Read>(input: &mut R, code: &mut Code) -> Self {
        let tag = serializable::read_u8(input);
        match tag {
            0x00 => ProgramObject::Integer(serializable::read_i32(input)),
            0x01 => ProgramObject::Null,
            0x02 => ProgramObject::String(serializable::read_utf8(input)),
            0x03 => ProgramObject::Method { name: ConstantPoolIndex::from_bytes(input),
                                            arguments: Arity::from_bytes(input),
                                            locals: Size::from_bytes(input),
                                            code: code.register_opcodes(OpCode::read_opcode_vector(input))},
            0x04 => ProgramObject::Slot { name: ConstantPoolIndex::from_bytes(input) },
            0x05 => ProgramObject::Class(ConstantPoolIndex::read_cpi_vector(input)),
            0x06 => ProgramObject::Boolean(serializable::read_bool(input)),
            _    => panic!("Cannot deserialize value: unrecognized value tag: {}", tag)
        }
    }
}

impl ProgramObject {

    #[allow(dead_code)]
    pub fn null() -> Self {
        ProgramObject::Null
    }

    #[allow(dead_code)]
    pub fn from_bool(b: bool) -> Self {
        ProgramObject::Boolean(b)
    }

    pub fn from_str(string: &str) -> Self {
        ProgramObject::String(string.to_string())
    }

    #[allow(dead_code)]
    pub fn from_string(string: String) -> Self {
        ProgramObject::String(string)
    }

    #[allow(dead_code)]
    pub fn from_i32(n: i32) -> Self {
        ProgramObject::Integer(n)
    }

    #[allow(dead_code)]
    pub fn from_usize(n: usize) -> Self {
        ProgramObject::Integer(n as i32)
    }

    pub fn slot_from_index(index: ConstantPoolIndex) -> Self {
        ProgramObject::Slot { name: index }
    }

    #[allow(dead_code)]
    pub fn slot_from_u16(index: u16) -> Self {
        ProgramObject::Slot { name: ConstantPoolIndex::new(index) }
    }

    #[allow(dead_code)]
    pub fn class_from_vec(indices: Vec<u16>) -> Self {
        ProgramObject::Class(indices.iter().map(|n| ConstantPoolIndex::new(*n)).collect())
    }
}

#[derive(PartialEq,Eq,Debug,Hash,Clone,Copy)]
pub struct HeapPointer(usize);

impl From<usize> for HeapPointer {
    fn from(n: usize) -> Self {
        HeapPointer(n)
    }
}

impl From<&Pointer> for HeapPointer {
    fn from(p: &Pointer) -> Self {
        match p {
            Pointer::Reference(p) => p.clone(),
            Pointer::Null => panic!("Cannot create heap reference from a null-tagged pointer"),
            Pointer::Integer(_) => panic!("Cannot create heap reference from an integer-tagged pointer"),
            Pointer::Boolean(_) => panic!("Cannot create heap reference from a boolean-tagged pointer"),
        }
    }
}

impl From<Pointer> for HeapPointer {
    fn from(p: Pointer) -> Self {
        match p {
            Pointer::Reference(p) => p.clone(),
            Pointer::Null => panic!("Cannot create heap reference from a null-tagged pointer"),
            Pointer::Integer(_) => panic!("Cannot create heap reference from an integer-tagged pointer"),
            Pointer::Boolean(_) => panic!("Cannot create heap reference from a boolean-tagged pointer"),
        }
    }
}

impl HeapPointer {
    pub fn as_usize(&self) -> usize { self.0 }
}

impl std::fmt::Display for HeapPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x>8}", self.0)
    }
}

#[derive(PartialEq,Eq,Debug,Hash,Clone,Copy)]
pub enum Pointer {
    Null,
    Integer(i32),
    Boolean(bool),
    Reference(HeapPointer),
}

// impl Deref for Pointer {
//     type Target = Pointer;
//     fn deref(&self) -> &Self::Target {
//         self.clone()
//     }
// }

impl Pointer {
    pub fn from_literal(program_object: &ProgramObject) -> anyhow::Result<Pointer> {
        match program_object {
            ProgramObject::Null => Ok(Self::Null),
            ProgramObject::Integer(value) => Ok(Self::Integer(*value)),
            ProgramObject::Boolean(value) => Ok(Self::Boolean(*value)),
            _ => anyhow::bail!("Expecting either a null, an integer, or a boolean, but found `{}`.", program_object),
        }
    }

    pub fn is_heap_reference(&self) -> bool {
        match self {
            Pointer::Reference(_) => true,
            _ => false,
        }
    }
    pub fn as_heap_reference(&self) -> Option<&HeapPointer> {
        match self {
            Pointer::Reference(reference) => Some(reference),
            _ => None,
        }
    }
    pub fn into_heap_reference(self) -> Option<HeapPointer> {
        match self {
            Pointer::Reference(reference) => Some(reference),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Pointer::Null => true,
            _ => false,
        }
    }
    pub fn as_null(&self) -> Option<()> {
        match self {
            Pointer::Null => Some(()),
            _ => None,
        }
    }

    pub fn is_i32(&self) -> bool {
        match self {
            Pointer::Integer(_) => true,
            _ => false,
        }
    }
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Pointer::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Pointer::Boolean(_) => true,
            _ => false,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Pointer::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn evaluate_as_condition(&self) -> bool {
        match self {
            Pointer::Null => false,
            Pointer::Integer(_) => true,
            Pointer::Boolean(b) => *b,
            Pointer::Reference(_) => true,
        }
    }
}

impl Into<bool> for Pointer {
    fn into(self) -> bool {
        match self {
            Pointer::Boolean(b) => b,
            p => panic!("Cannot cast `{}` into a boolean pointer.", p),
        }
    }
}

impl Into<i32> for Pointer {
    fn into(self) -> i32 {
        match self {
            Pointer::Integer(i) => i,
            p => panic!("Cannot cast `{}` into an integer pointer.", p),
        }
    }
}

// impl Into<HeapPointer> for Pointer {
//     fn into(self) -> HeapPointer {
//         match self {
//             Pointer::Reference(heap_pointer) => heap_pointer,
//             p => panic!("Cannot cast `{}` into an untagged pointer.", p),
//         }
//     }
// }

impl From<&ProgramObject> for Pointer {
    fn from(constant: &ProgramObject) -> Self {
        match constant {
            ProgramObject::Null => Self::Null,
            ProgramObject::Integer(value) => Self::Integer(*value),
            ProgramObject::Boolean(value) => Self::Boolean(*value),
            _ => unimplemented!(),
        }
    }
}

impl From<ProgramObject> for Pointer {
    fn from(constant: ProgramObject) -> Self {
        match constant {
            ProgramObject::Null => Self::Null,
            ProgramObject::Integer(value) => Self::Integer(value),
            ProgramObject::Boolean(value) => Self::Boolean(value),
            _ => unimplemented!(),
        }
    }
}

impl From<&HeapPointer> for Pointer {
    fn from(p: &HeapPointer) -> Self {
        Pointer::Reference(p.clone())
    }
}

impl From<HeapPointer> for Pointer {
    fn from(p: HeapPointer) -> Self {
        Pointer::Reference(p)
    }
}

impl From<usize> for Pointer {
    fn from(n: usize) -> Self {
        Pointer::from(HeapPointer::from(n))
    }
}

impl From<i32> for Pointer {
    fn from(i: i32) -> Self {
        Pointer::Integer(i)
    }
}

impl From<bool> for Pointer {
    fn from(b: bool) -> Self {
        Pointer::Boolean(b)
    }
}

impl std::fmt::Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pointer::Null => write!(f, "null"),
            Pointer::Integer(i) => write!(f, "{}", i),
            Pointer::Boolean(b) => write!(f, "{}", b),
            Pointer::Reference(p) => write!(f, "{}", p),
        }
    }
}

#[derive(PartialEq,Debug,Clone)]
pub enum HeapObject {
    // Null,
    // Integer(i32),
    // Boolean(bool),
    Array(Vec<Pointer>),
    Object {
        parent: Pointer,
        fields: HashMap<String, Pointer>,
        methods: HashMap<String, ProgramObject>,
    },
}

impl HeapObject {
    pub fn empty_object() -> Self {
        HeapObject::Object {
            parent: Pointer::Null,
            fields: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    pub fn empty_array() -> Self {
        HeapObject::Array(Vec::new())
    }

    pub fn from_pointers(v: Vec<Pointer>) -> Self {
        HeapObject::Array(v)
    }

    // pub fn from_i32(n: i32) -> Self {
    //     HeapObject::Integer(n)
    // }
    //
    // pub fn from_bool(b: bool) -> Self {
    //     HeapObject::Boolean(b)
    // }

    // pub fn from_constant(constant: &ProgramObject) -> Self {
    //     match constant {
    //         ProgramObject::Null => HeapObject::Null,
    //         ProgramObject::Integer(value) => HeapObject::Integer(*value),
    //         ProgramObject::Boolean(value) => HeapObject::Boolean(*value),
    //         _ => unimplemented!(),
    //     }
    // }

    pub fn from(parent: Pointer, fields: HashMap<String, Pointer>, methods: HashMap<String, ProgramObject>) -> Self {
        HeapObject::Object { parent, fields, methods }
    }
}

impl std::fmt::Display for HeapObject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // HeapObject::Null => write!(f, "null"),
            // HeapObject::Integer(n) => write!(f, "{}", n),
            // HeapObject::Boolean(b) => write!(f, "{}", b),
            HeapObject::Array(elements) => {
                write!(f, "[{}]", {
                    elements.iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                })
            },
            HeapObject::Object { parent, fields, methods:_ } => {
                write!(f, "object(..={}, {})", parent, fields.iter()
                    .map(|(name, field)| format!("{}={}", name, field))
                    .collect::<Vec<String>>()
                    .join(", "))
            }
        }
    }
}