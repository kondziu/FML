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
use crate::bytecode::interpreter::Heap;
use crate::bytecode;
use anyhow::Context;

use crate::bail_if;
use crate::bytecode::interp::OperandStack;

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
        parameters: Arity,
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

#[derive(PartialEq,Debug,Clone)]
pub struct ObjectTemplate {
    fields: Vec<ConstantPoolIndex>,
    methods: HashMap<ConstantPoolIndex, ProgramObject>
}

impl ObjectTemplate {

}

impl ProgramObject {
    // pub fn as_object_template(&self) -> anyhow::Result<ObjectTemplate> {
    //     let mut fields: Vec<ConstantPoolIndex> = Vec::new();
    //     let mut methods: HashMap<ConstantPoolIndex, ProgramObject> = HashMap::new();
    //
    //     match self {
    //         ProgramObject::Class(members) => {
    //             for member in members {
    //                 match member {
    //                     ProgramObject::Slot { name } => {
    //                         fields.push(*name);
    //                     }
    //                     ProgramObject::Method { name, .. } => {
    //                         methods.insert(*name, member.clone())?;
    //                     }
    //                     _ => anyhow::bail!("Class members must be either a Method or a Slot, but found `{}`.", member)
    //                 }
    //             }
    //         }
    //         _ => anyhow::bail!("Expecting a program object representing a Class, found `{}`", self)
    //     }
    //
    //     Ok(ObjectTemplate { fields, methods })
    // }

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

    pub fn get_method_parameters(&self) -> anyhow::Result<&Arity> {
        match self {                                                                                // FIXME there's gotta be a way to do this cleaner. perhaps locally defined function?
            ProgramObject::Method { parameters, .. } => Ok(parameters),
            pointer => Err(anyhow::anyhow!("Expected a Method but found `{}`", pointer)),
        }
    }

    pub fn get_method_locals(&self) -> anyhow::Result<&Size> {
        match self {                                                                                // FIXME there's gotta be a way to do this cleaner. perhaps locally defined function?
            ProgramObject::Method { locals, .. } => Ok(locals),
            pointer => Err(anyhow::anyhow!("Expected a Method but found `{}`", pointer)),
        }
    }

    pub fn get_method_start_address(&self) -> anyhow::Result<&Address> {
        match self {                                                                                // FIXME there's gotta be a way to do this cleaner. perhaps locally defined function?
            ProgramObject::Method { code, .. } => Ok(code.start()),
            pointer => Err(anyhow::anyhow!("Expected a Method but found `{}`", pointer)),
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
            Method {name: _, parameters: _, locals: _, code: _} => 0x03,
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
            ProgramObject::Method { name, locals, parameters: arguments, code } => {
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

            Method {name, parameters: arguments, locals, code: range} => {
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
                                            parameters: Arity::from_bytes(input),
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
pub struct HeapIndex(usize);

impl From<usize> for HeapIndex {
    fn from(n: usize) -> Self {
        HeapIndex(n)
    }
}

impl From<&Pointer> for HeapIndex {
    fn from(p: &Pointer) -> Self {
        match p {
            Pointer::Reference(p) => p.clone(),
            Pointer::Null => panic!("Cannot create heap reference from a null-tagged pointer"),
            Pointer::Integer(_) => panic!("Cannot create heap reference from an integer-tagged pointer"),
            Pointer::Boolean(_) => panic!("Cannot create heap reference from a boolean-tagged pointer"),
        }
    }
}

impl From<Pointer> for HeapIndex {
    fn from(p: Pointer) -> Self {
        match p {
            Pointer::Reference(p) => p.clone(),
            Pointer::Null => panic!("Cannot create heap reference from a null-tagged pointer"),
            Pointer::Integer(_) => panic!("Cannot create heap reference from an integer-tagged pointer"),
            Pointer::Boolean(_) => panic!("Cannot create heap reference from a boolean-tagged pointer"),
        }
    }
}

impl HeapIndex {
    pub fn as_usize(&self) -> usize { self.0 }
}

impl std::fmt::Display for HeapIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x>8}", self.0)
    }
}

#[derive(PartialEq,Eq,Debug,Hash,Clone,Copy)]
pub enum Pointer {
    Null,
    Integer(i32),
    Boolean(bool),
    Reference(HeapIndex),
}

// impl Deref for Pointer {
//     type Target = Pointer;
//     fn deref(&self) -> &Self::Target {
//         self.clone()
//     }
// }

impl Pointer {
    pub fn push_onto(self, stack: &mut OperandStack) {
        stack.push(self);
    }
}

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
    pub fn as_heap_reference(&self) -> Option<&HeapIndex> {
        match self {
            Pointer::Reference(reference) => Some(reference),
            _ => None,
        }
    }
    pub fn into_heap_reference(self) -> anyhow::Result<HeapIndex> {
        match self {
            Pointer::Reference(reference) => Ok(reference),
            pointer => Err(anyhow::anyhow!("Expecting a heap reference, but found `{}`.", pointer)),
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
    pub fn as_i32(&self) -> anyhow::Result<i32> {
        match self {
            Pointer::Integer(i) => Ok(*i),
            pointer => Err(anyhow::anyhow!("Expecting an integer, but found `{}`", pointer)),
        }
    }

    pub fn as_usize(&self) -> anyhow::Result<usize> {
        match self {
            Pointer::Integer(i) if *i >= 0 => Ok(*i as usize),
            pointer => Err(anyhow::anyhow!("Expecting a positive integer, but found `{}`", pointer)),
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

    pub fn evaluate_as_string(&self) -> String {
        unimplemented!()
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

impl From<&HeapIndex> for Pointer {
    fn from(p: &HeapIndex) -> Self {
        Pointer::Reference(p.clone())
    }
}

impl From<HeapIndex> for Pointer {
    fn from(p: HeapIndex) -> Self {
        Pointer::Reference(p)
    }
}

impl From<usize> for Pointer {
    fn from(n: usize) -> Self {
        Pointer::from(HeapIndex::from(n))
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
pub struct ObjectInstance {
    pub parent: Pointer,
    pub fields: HashMap<String, Pointer>, // TODO make private
    pub methods: HashMap<String, ProgramObject> // TODO make private
}

impl ObjectInstance {
    pub fn new() -> Self {
        ObjectInstance  {
            parent: Pointer::Null,
            fields: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    pub fn get_field(&self, name: &str) -> anyhow::Result<&Pointer> {
        self.fields.get(name)
            .with_context(|| format!("There is no field named `{}` in object `{}`", name, self))
    }

    pub fn set_field(&mut self, name: &str, pointer: Pointer) -> anyhow::Result<Pointer> {
        self.fields.insert(name.to_owned(), pointer)
            .with_context(|| format!("There is no field named `{}` in object `{}`", name, self))
    }
}

impl std::fmt::Display for ObjectInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "object(..={}, {})", self.parent, self.fields.iter()
            .map(|(name, field)| format!("{}={}", name, field))
            .collect::<Vec<String>>()
            .join(", "))

    }
}

#[derive(PartialEq,Debug,Clone)]
pub struct ArrayInstance(Vec<Pointer>);

impl ArrayInstance {
    pub fn new() -> Self {
        ArrayInstance(vec![])
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=&Pointer> + 'a {
        self.0.iter()
    }

    pub fn length(&self) -> usize {
        self.0.len()
    }

    pub fn get_element(&self, index: usize) -> anyhow::Result<&Pointer> {
        let length = self.0.len();
        bail_if!(index >= length,
                 "Index out of range {} for array `{}` with length {}",
                 index, self, length);
        Ok(&self.0[index])
    }

    pub fn set_element(&mut self, index: usize, value_pointer: Pointer) -> anyhow::Result<&Pointer> {
        let length = self.0.len();
        bail_if!(index >= length,
                 "Index out of range {} for array `{}` with length {}",
                 index, self, length);
        self.0[index] = value_pointer;
        Ok(&self.0[index])
    }
}

impl From<Vec<Pointer>> for ArrayInstance {
    fn from(v: Vec<Pointer>) -> Self {
        ArrayInstance(v)
    }
}

impl std::fmt::Display for ArrayInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.iter()
            .map(|pointer| format!("{}", pointer))
            .collect::<Vec<String>>()
            .join(", "))
    }
}

#[derive(PartialEq,Debug,Clone)]
pub enum HeapObject {
    // Null,
    // Integer(i32),
    // Boolean(bool),
    Array(ArrayInstance),
    Object(ObjectInstance)
}

impl HeapObject {
    pub fn new_object(parent: Pointer, fields: HashMap<String, Pointer>, methods: HashMap<String, ProgramObject>) -> Self {
        HeapObject::Object(ObjectInstance { parent, fields, methods })
    }

    pub fn as_object_instance(&self) -> anyhow::Result<&ObjectInstance> {
        match self {
            HeapObject::Object(instance) => Ok(instance),
            array => Err(anyhow::anyhow!("Attempt to cast an array as an object instance `{}`.", array)),
        }
    }

    pub fn as_object_instance_mut(&mut self) -> anyhow::Result<&mut ObjectInstance> {
        match self {
            HeapObject::Object(instance) => Ok(instance),
            array => Err(anyhow::anyhow!("Attempt to cast an array as an object instance `{}`.", array)),
        }
    }
    
    // pub fn with_object<F, R>(&self, mut f: F) -> anyhow::Result<R>
    //     where F: FnMut(&Self, &Pointer, &HashMap<String, Pointer>, &HashMap<String, ProgramObject>) -> anyhow::Result<R> {
    //     match self {
    //         HeapObject::Object { parent, fields, methods } => f(self, parent, fields, methods),
    //         array => Err(anyhow::anyhow!("Expecting an object instance, but found `{}`.", array)),
    //     }
    // }
    //
    // pub fn with_object_mut<F, R>(&mut self, mut f: F) -> anyhow::Result<R>
    //     where F: FnMut(&mut Self, &mut Pointer, &mut HashMap<String, Pointer>, &mut HashMap<String, ProgramObject>) -> anyhow::Result<R> {
    //     match self {
    //         HeapObject::Object { parent, fields, methods } => f(parent, fields, methods),
    //         array => Err(anyhow::anyhow!("Expecting an object instance, but found `{}`.", array)),
    //     }
    // }

    pub fn empty_object() -> Self {
        HeapObject::Object(ObjectInstance::new())
    }

    pub fn empty_array() -> Self {
        HeapObject::Array(ArrayInstance::new())
    }

    pub fn from_pointers(v: Vec<Pointer>) -> Self {
        HeapObject::Array(ArrayInstance::from(v))
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
        HeapObject::Object(ObjectInstance { parent, fields, methods })
    }
}

impl std::fmt::Display for HeapObject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
                HeapObject::Array(array) => write!(f, "{}", array),
                HeapObject::Object(object) => write!(f, "{}", object),
        }
    }
}