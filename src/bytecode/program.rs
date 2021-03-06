use std::io::{Write, Read};
use std::collections::HashMap;

use super::bytecode::OpCode;

use super::serializable;
use super::serializable::*;

use anyhow::*;

use super::serializable::Serializable;
use std::iter::repeat;

use crate::bail_if;

pub struct Program {
    pub constant_pool: ConstantPool,
    pub labels: Labels,
    pub code: Code,
    pub globals: Globals,
    pub entry: Entry,
}

impl Program {
    pub fn new() -> Self {
        Program {
            labels: Labels::new(),
            constant_pool: ConstantPool::new(),
            code: Code::new(),
            globals: Globals::new(),
            entry: Entry::new(),
        }
    }

    pub fn from(code: Code, constant_pool: ConstantPool, globals: Globals, entry: Entry) -> Result<Self> {
        let label_names = code.labels();
        let label_constants = constant_pool.get_all(label_names)?.into_iter();
        let label_addresses = code.label_addresses().into_iter();
        let labels = Labels::from(label_constants.zip(label_addresses)).unwrap();

        Ok(Program { labels, constant_pool, code, globals, entry })
    }

    // pub fn emit_code(&mut self, opcode: OpCode) {
    //     match opcode {
    //         OpCode::Label {name: index} => {
    //             let address = Address::from_usize(self.code.length());
    //             self.code.push(opcode);
    //             let constant = self.get_constant(&index);
    //             match constant {
    //                 Some(ProgramObject::String(name)) => {
    //                     let name = name.to_owned();
    //                     let result = self.labels.register_label_address(name, address);
    //
    //                     if result.is_some() {
    //                          panic!("Emit code error: cannot create label {:?}, \
    //                                            name {:?} already used by another label.",
    //                                                opcode, self.get_constant(&index))
    //                     }
    //                 },
    //                 Some(object) => panic!("Emit code error: cannot create label, \
    //                                         constant at index {:?} should be a String, but is {:?}",
    //                                         index, object),
    //
    //                 None => panic!("Emit code error: cannot create label, \
    //                                 there is no constant at index {:?}", index),
    //             }
    //
    //         }
    //         _ => self.code.opcodes.push(opcode),
    //     }
    // }

//     #[allow(dead_code)]
//     pub fn new(code: Code,
//                constants: Vec<ProgramObject>,
//                globals: Vec<ConstantPoolIndex>,
//                entry: ConstantPoolIndex) -> Program {
//
//         let labels = Program::labels_from_code(&code, &constants);
//
//         Program { code, labels, constants, globals, entry }
//     }
//

//
//     fn labels_from_code(code: &Code, constants: &Vec<ProgramObject>) -> Labels {
//         let mut labels: HashMap<String, Address> = HashMap::new();
//         for (i, opcode) in code.opcodes.iter().enumerate() {
//             if let OpCode::Label { name: index } = opcode {
//                 let constant = constants.get(index.value() as usize)
//                     .expect(&format!("Program initialization: label {:?} expects a constant in the \
//                                       constant pool at index {:?} but none was found",
//                                      opcode, index));
//
//                 let name = match constant {
//                     ProgramObject::String(string) => string,
//                     _ => panic!("Program initialization: label {:?} expects a String in the \
//                                  constant pool at index {:?} but {:?} was found",
//                                 opcode, index, constant),
//                 };
//
//                 if labels.contains_key(name) {
//                     panic!("Program initialization: attempt to define label {:?} with a non-unique \
//                             name: {}", opcode, name)
//                 }
//
//                 labels.insert(name.to_string(), Address::from_usize(i));
//             };
//         }
//         Labels::from(labels)
//     }
//
//     pub fn code(&self) -> &Code {
//         &self.code
//     }
//
//     pub fn constants(&self) -> &Vec<ProgramObject> {
//         &self.constants
//     }
//
//     pub fn labels(&self) -> &HashMap<String, Address> {
//         &self.labels.all()
//     }
//
//     pub fn globals(&self) -> &Vec<ConstantPoolIndex> {
//         &self.globals
//     }
//
//     pub fn entry(&self) -> &ConstantPoolIndex {
//         &self.entry
//     }
//
//     pub fn get_constant(&self, index: &ConstantPoolIndex) -> Option<&ProgramObject> {
//         self.constants.get(index.value() as usize)
//     }
//
//     pub fn get_opcode(&self, address: &Address) -> Option<&OpCode> {
//         self.code.get_opcode(address)
//     }
//
//     pub fn get_label(&self, name: &str) -> Option<&Address> {
//         self.labels.get_label_address(name)
//     }
//
//     //-----------
//

//
//     pub fn register_global(&mut self, constant: ConstantPoolIndex) {
//         if self.globals.contains(&constant) {
//             panic!("Cannot register global {:?}, this index is already registered.", constant)
//         }
//
//         self.globals.push(constant)
//     }
//
// //    fn register_label(&mut self, label: String) -> ConstantPoolIndex {
// //        if let Some(index) = self.labels.get(&label) {
// //            return *index;
// //        }
// //        let index = ConstantPoolIndex::from_usize(self.labels.len());
// //        self.labels.insert(label, index);
// //        index
// //    }
//
//     // pub fn generate_new_label_name(&mut self, name: &str) -> ConstantPoolIndex {
//     //     let label = self.labels.generate_label(name).unwrap();
//     //     self.labels.new_group();
//     //     let constant = ProgramObject::String(label);
//     //     let index = self.register_constant(constant);
//     //
//     //     index
//     // }
//
//     pub fn generate_new_label_names(&mut self, names: Vec<&str>) -> Vec<ConstantPoolIndex> {
//         let labels: Vec<String> = names.into_iter()
//             .map(|name| self.labels.generate_label(name))
//             .map(|label| label.unwrap())
//             .collect();
//
//         self.labels.new_group();
//
//         labels.into_iter()
//             .map(|label| {
//                 self.register_constant(ProgramObject::String(label.clone()))
//             })
//             .collect()
//     }
//
//     pub fn get_current_address(&self) -> Address {
//         let size = self.code.opcodes.len();
//         Address::from_usize(size - 1)
//     }
//
//     pub fn get_upcoming_address(&self) -> Address {
//         let size = self.code.opcodes.len();
//         Address::from_usize(size)
//     }
//
//     pub fn set_entry(&mut self, function_index: ConstantPoolIndex) {
//         self.entry = function_index;
//     }
//
//     pub fn emit_conditionally(&mut self, opcode: OpCode, emit: bool) {
//         if emit { self.emit_code(opcode) }
//     }
//
//     pub fn emit_code(&mut self, opcode: OpCode) {
//         // println!("Emitting code: {:?}", opcode);
//         match opcode {
//             OpCode::Label {name: index} => {
//                 let address = Address::from_usize(self.code.opcodes.len());
//                 self.code.opcodes.push(opcode);
//                 let constant = self.get_constant(&index);
//                 match constant {
//                     Some(ProgramObject::String(name)) => {
//                         let name = name.to_owned();
//                         let result = self.labels.register_label_address(name, address);
//
//                         if result.is_some() {
//                              panic!("Emit code error: cannot create label {:?}, \
//                                                name {:?} already used by another label.",
//                                                    opcode, self.get_constant(&index))
//                         }
//                     },
//                     Some(object) => panic!("Emit code error: cannot create label, \
//                                             constant at index {:?} should be a String, but is {:?}",
//                                             index, object),
//
//                     None => panic!("Emit code error: cannot create label, \
//                                     there is no constant at index {:?}", index),
//                 }
//
//             }
//             _ => self.code.opcodes.push(opcode),
//         }
//     }
}

pub struct Globals(Vec<ConstantPoolIndex>);
impl Globals {
    pub fn new() -> Self { Globals(Vec::new()) }
    pub fn register(&mut self, name_index: ConstantPoolIndex) -> Result<()> {
        bail_if!(self.0.contains(&name_index),
                 "Cannot register global `{}`, index is already registered as a global.",
                 name_index);
        Ok(self.0.push(name_index))
    }
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=ConstantPoolIndex> + 'a{
        self.0.iter().map(|index| index.clone())
    }
}

impl From<Vec<ConstantPoolIndex>> for Globals {
    fn from(vector: Vec<ConstantPoolIndex>) -> Self {
        Globals(vector)
    }
}

pub struct Entry(Option<ConstantPoolIndex>);
impl Entry {
    pub fn new() -> Self { Entry(None) }
    pub fn get(&self) -> Result<ConstantPoolIndex> {
        bail_if!(self.0.is_none(), "Entry point was read, but it was not set yet.", /*bad macro*/);
        Ok(self.0.as_ref().unwrap().clone())
    }
    pub fn set(&mut self, index: ConstantPoolIndex) {
        self.0 = Some(index)
    }
}
impl From<ConstantPoolIndex> for Entry {
    fn from(index: ConstantPoolIndex) -> Self {
        Entry(Some(index))
    }
}
impl From<u16> for Entry {
    fn from(index: u16) -> Self { Entry(Some(ConstantPoolIndex::from(index))) }
}

pub struct Labels { names: HashMap<String, Address>, groups: usize } // FIXME
impl Labels {
    pub fn new() -> Self { Labels { names: HashMap::new(), groups: 0 } }
    pub fn get(&self, label: &str) -> Result<&Address> {
        self.names.get(label).with_context(|| format!("Label `{}` was not previously register.", label))
    }
    pub fn set(&mut self, label: String, index: Address) -> Result<()> {
        let previous = self.names.insert(label.clone(), index);
        match previous {
            Some(old_index) =>
                Err(anyhow!("Attempting to register `{}` at address `{}`, \
                             but it was already registered at address `{}`",
                             label, index, old_index)),
            None => Ok(()),
        }
    }
    pub(crate) fn generate_name_within_group<S>(&self, prefix: S, group: usize) -> Result<String> where S: Into<String> {
        let name = format!("{}:{}", prefix.into(), group);
        bail_if!(self.names.contains_key(&name), "Label `{}` already exists.", name);
        Ok(name)
    }
    pub fn generate_name<S>(&mut self, prefix: S) -> Result<String> where S: Into<String> {
        let name = self.generate_name_within_group(prefix, self.groups + 1)?;
        self.groups = self.groups + 1;
        Ok(name)
    }
    pub fn generate<S>(&mut self, prefix: S) -> Result<ProgramObject> where S: Into<String> {
        self.generate_name(prefix)
            .map(|name| ProgramObject::String(name))
    }
    pub fn create_group(&mut self) -> LabelGroup<'_> {
        let group = self.groups;
        self.groups = self.groups + 1;
        LabelGroup { labels: self, group }
    }

    pub fn from<'a, I>(labels: I) -> Result<Self>
        where I: IntoIterator<Item=(&'a ProgramObject, Address)> {

        let names = labels.into_iter()
            .map(|(program_object, address)| {
                program_object.as_str().map(|name| (name.to_owned(), address))
            })
            .collect::<Result<HashMap<String, Address>>>()?;

        let groups = names.iter().flat_map(|(label, _)| {
            label.split(":").last().map(|s| {
                s.parse::<usize>().map_or(None, |n| Some(n))
            }).flatten()
        }).max().map_or(0, |n| n + 1);

        Ok(Labels { names, groups })
    }
}

pub struct LabelGroup<'a> { labels: &'a Labels, group: usize }

impl LabelGroup<'_> {
    pub fn generate_name<S>(&self, prefix: S) -> Result<String> where S: Into<String> {
        self.labels.generate_name_within_group(prefix, self.group)
    }

    pub fn generate<S>(&self, prefix: S) -> Result<ProgramObject> where S: Into<String> {
        self.labels.generate_name_within_group(prefix, self.group)
            .map(|name| ProgramObject::String(name))
    }
}


// pub struct Dictionary<T>(HashMap<String, T>);                                                       // TODO make implementation just for globals to makje the error messages good.
// impl<T> Dictionary<T> {
//     pub fn new() -> Self { Dictionary(HashMap::new()) }
//     pub fn get(&self, name: &str) -> Result<&T> {
//         self.0.get(name)
//             .with_context(|| Err(anyhow!("Cannot reference `{}`: key not found.", name)))
//     }
//     pub fn set(&mut self, name: String, pointer: T) -> Result<()> {
//         self.0.insert(name, pointer)
//             .with_context(|| Err(anyhow!("Cannot set value of `{}`: key not found.", name)))
//     }
// }

pub struct ConstantPool(Vec<ProgramObject>);
impl ConstantPool {
    pub fn new() -> Self { ConstantPool(Vec::new()) }
    pub fn get(&self, index: &ConstantPoolIndex) -> Result<&ProgramObject> {
        self.0.get(index.as_usize())
            .with_context(||
                format!("Cannot dereference object from the constant pool at index: `{}`", index))
    }
    pub fn get_all(&self, indices: Vec<&ConstantPoolIndex>) -> Result<Vec<&ProgramObject>> {
        indices.iter()
            .map(|index| self.get(index))
            .collect()
    }
    pub fn push(&mut self, program_object: ProgramObject) -> ConstantPoolIndex {
        self.0.push(program_object);
        ConstantPoolIndex::from_usize(self.0.len() - 1)
    }
    pub fn find(&self, program_object: &ProgramObject) -> Option<ConstantPoolIndex> {
        self.0.iter()
            .position(|c| c == program_object)
            .map(|position| ConstantPoolIndex::from_usize(position))
    }
    pub fn register(&mut self, program_object: ProgramObject) -> ConstantPoolIndex {
        let index = self.find(&program_object);
        match index {
            Some(index) => index,
            None => self.push(program_object),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item=&ProgramObject> {
        self.0.iter()
    }
    pub fn size(&self) -> usize {
        self.0.len()
    }
}

impl From<Vec<ProgramObject>> for ConstantPool {
    fn from(vector: Vec<ProgramObject>) -> Self {
        ConstantPool(vector)
    }
}

impl From<Vec<i32>> for ConstantPool {
    fn from(vector: Vec<i32>) -> Self {
        ConstantPool(vector.into_iter().map(|n| ProgramObject::from_i32(n)).collect())
    }
}

impl From<Vec<&str>> for ConstantPool {
    fn from(vector: Vec<&str>) -> Self {
        ConstantPool(vector.into_iter().map(|s| ProgramObject::from_str(s)).collect())
    }
}

impl From<Vec<bool>> for ConstantPool {
    fn from(vector: Vec<bool>) -> Self {
        ConstantPool(vector.into_iter().map(|b| ProgramObject::from_bool(b)).collect())
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
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

    pub fn get_method_parameters(&self) -> anyhow::Result<&Arity> {
        match self {                                                                                // FIXME there's gotta be a way to do this cleaner. perhaps locally defined function?
            ProgramObject::Method { parameters, .. } => Ok(parameters),
            pointer => Err(anyhow::anyhow!("Expected a Method but found `{}`", pointer)),
        }
    }

    pub fn get_method_name(&self) -> anyhow::Result<&ConstantPoolIndex> {
        match self {                                                                                // FIXME there's gotta be a way to do this cleaner. perhaps locally defined function?
            ProgramObject::Method { name, .. } => Ok(name),
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

            Method {name, parameters, locals, code: range} => {
                name.serialize(sink)?;
                parameters.serialize(sink)?;
                locals.serialize(sink)?;
                OpCode::write_opcode_vector(sink, &code.materialize(range)?)
            }
        }
    }

    fn from_bytes<R: Read>(input: &mut R, code: &mut Code) -> Self {                                // TODO error handling
        let tag = serializable::read_u8(input);
        match tag {
            0x00 => ProgramObject::Integer(serializable::read_i32(input)),
            0x01 => ProgramObject::Null,
            0x02 => ProgramObject::String(serializable::read_utf8(input)),
            0x03 => ProgramObject::Method { name: ConstantPoolIndex::from_bytes(input),
                parameters: Arity::from_bytes(input),
                locals: Size::from_bytes(input),
                code: code.append(OpCode::read_opcode_vector(input))},
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


#[derive(PartialEq,Debug,Clone)]
pub struct Code(Vec<OpCode>);
impl Code {
    pub fn new() -> Self { Code(Vec::new()) }
    pub fn current_address(&self) -> Address {
        Address::from_usize(self.0.len() - 1)
    }
    pub fn upcoming_address(&self) -> Address {
        Address::from_usize(self.0.len())
    }
    pub fn emit(&mut self, opcode: OpCode) {
        self.0.push(opcode)
    }
    pub fn emit_if(&mut self, opcode: OpCode, condition: bool) {
        if condition { self.emit(opcode) }
    }
    pub fn emit_unless(&mut self, opcode: OpCode, condition: bool) {
        if !condition { self.emit(opcode) }
    }
    pub fn length(&self) -> usize {
        self.0.len()
    }
    pub fn materialize(&self, range: &AddressRange) -> Result<Vec<&OpCode>> {
        let start = range.start().value_usize();
        let end = start + range.length();

        bail_if!(end > self.0.len(),
                 "Address range exceeds code size: {} + {} >= {}.",
                 start, range.length, self.0.len());

        Ok((start..end).map(|index| &self.0[index]).collect())
    }
    pub fn append(&mut self, opcodes: Vec<OpCode>) -> AddressRange {
        let start = self.0.len();
        let length = opcodes.len();
        self.0.extend(opcodes);
        AddressRange::new(Address::from_usize(start), length)
    }
    pub fn labels(&self) -> Vec<&ConstantPoolIndex> {
        self.0.iter().flat_map(|opcode| match opcode {
            OpCode::Label { name } => Some(name),
            _ => None
        }).collect()
    }
    pub fn label_addresses(&self) -> Vec<Address> {
        self.0.iter().enumerate().flat_map(|(address, opcode)| match opcode {
            OpCode::Label { .. } => Some(Address::from_usize(address)),
            _ => None
        }).collect()
    }
    pub fn next(&self, address: Address) -> Option<Address> {
        let index = address.value_usize() + 1;
        if index < self.0.len() {
            Some(Address::from_usize(index))
        } else {
            None
        }
    }
}

impl From<Vec<OpCode>> for Code {
    fn from(vector: Vec<OpCode>) -> Self {
        Code(vector)
    }
}

// impl Code { // TODO refactor
//     pub fn new() -> Code {
//         Code(Vec::new())
//     }
//
//     #[allow(dead_code)]
//     pub fn from(opcodes: Vec<OpCode>) -> Code {
//         Code(opcodes)
//     }
//
//     #[allow(dead_code)]
//     pub fn all_opcodes(&self) -> Vec<(Address, OpCode)> {
//         self.opcodes.iter().enumerate().map(|(i, opcode)| {
//             (Address::from_usize(i), opcode.clone())
//         }).collect()
//     }
//
//     pub fn register_opcodes(&mut self, opcodes: Vec<OpCode>) -> AddressRange {
//         let start = self.opcodes.len();
//         let length = opcodes.len();
//         self.opcodes.extend(opcodes);
//         AddressRange::new(Address::from_usize(start), length)
//     }
//
//     pub fn addresses_to_code_vector(&self, range: &AddressRange) -> Vec<&OpCode> {
//         let start = range.start().value_usize();
//         let end = start + range.length();
//         let mut result: Vec<&OpCode> = Vec::new();
//         for i in start..end {
//             result.push(&self.opcodes[i]);
//         }
//         result
//     }
//
//     pub fn next_address(&self, address: Option<Address>) -> Option<Address> {
//         match address {
//             Some(address) => {
//                 let new_address = Address::from_usize(address.value_usize() + 1);
//                 if self.opcodes.len() > new_address.value_usize() {
//                     Some(new_address)
//                 } else {
//                     None
//                 }
//             }
//             None => panic!("Cannot advance a nothing address.")
//         }
//     }
//
//     pub fn get_opcode(&self, address: &Address) -> Option<&OpCode> {
//         //self.table[address.value() as usize]
//         self.opcodes.get(address.value_usize())
//     }
//
//     // pub fn dump(&self) { // TODO pretty print
//     //     for (i, opcode) in self.opcodes.iter().enumerate() {
//     //         println!("{}: {:?}", i, opcode);
//     //     }
//     // }
// }

// #[derive(PartialEq,Debug,Clone)]
// pub struct Labels {
//     labels: HashMap<String, Address>,
//     groups: usize,
// }

// impl Labels {
//     pub fn new() -> Self {
//         Labels { labels: HashMap::new(), groups: 0 }
//     }
//     pub fn from(labels: HashMap<String, Address>) -> Self {
//         let groups = labels.iter().flat_map(|(label, _)| {
//             label.split(":").last().map(|s| {
//                 s.parse::<usize>().map_or(None, |n| Some(n))
//             }).flatten()
//         }).max().map_or(0, |n| n + 1);
//         Labels { labels, groups }
//     }
//     pub fn generate_label<S>(&mut self, name: S) -> Option<String> where S: Into<String> {
//         let label = format!("{}:{}", name.into(), self.groups);
//         if self.labels.contains_key(&label) {
//             None
//         } else {
//             Some(label)
//         }
//     }
//     pub fn register_label_address<S>(&mut self, name: S, address: Address) -> Option<Address> where S: Into<String> {
//         self.labels.insert(name.into(), address)
//     }
//     pub fn new_group(&mut self) {
//         self.groups = self.groups + 1
//     }
//     pub fn get_label_address(&self, name: &str) -> Option<&Address> {
//         self.labels.get(name)
//     }
//     pub fn all(&self) -> &HashMap<String, Address> {
//         &self.labels
//     }
// }

// #[derive(PartialEq,Debug,Clone)]
// pub struct Program {
//     code: Code,
//     labels: Labels,
//     constants: Vec<ProgramObject>,
//     globals: Vec<ConstantPoolIndex>,
//     entry: ConstantPoolIndex,
// }
//


impl Serializable for Program {
    fn serialize<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        self.constant_pool.serialize(sink, &self.code)?;
        self.globals.serialize(sink)?;
        self.entry.serialize(sink)
    }

    fn from_bytes<R: Read>(input: &mut R) -> Self {
        let mut code = Code::new();

        let constant_pool = ConstantPool::from_bytes(input, &mut code);
        let globals = Globals::from_bytes(input);
        let entry = Entry::from_bytes(input);

        let label_names = code.labels();
        let label_constants = constant_pool.get_all(label_names).unwrap().into_iter();                // TODO error handling
        let label_addresses = code.label_addresses().into_iter();
        let labels = Labels::from(label_constants.zip(label_addresses)).unwrap();

        Program { constant_pool, labels, code, globals, entry }
    }
}

impl SerializableWithContext for ConstantPool {
    fn serialize<W: Write>(&self, sink: &mut W, code: &Code) -> Result<(), Error> {
        unimplemented!()
    }

    fn from_bytes<R: Read>(input: &mut R, code: &mut Code) -> Self {
        let size = serializable::read_u16_as_usize(input);
        let constants: Vec<ProgramObject> =
            (0..size).map(|_| ProgramObject::from_bytes(input, code)).collect();

        ConstantPool(constants)
    }
}

impl Serializable for Globals {
    fn serialize<W: Write>(&self, sink: &mut W) -> Result<(), Error> {
        ConstantPoolIndex::write_cpi_vector(sink, &self.0)
    }

    fn from_bytes<R: Read>(input: &mut R) -> Self {
        Globals(ConstantPoolIndex::read_cpi_vector(input))
    }
}

impl Serializable for Entry {
    fn serialize<W: Write>(&self, sink: &mut W) -> Result<(), Error> {
        self.0.expect("Cannot serialize an empty entry point.").serialize(sink)
    }

    fn from_bytes<R: Read>(input: &mut R) -> Self {
        Entry(Some(ConstantPoolIndex::from_bytes(input)))
    }
}

#[derive(PartialEq,Debug,Copy,Clone,Eq,PartialOrd,Ord,Hash)] pub struct Arity(u8);
#[derive(PartialEq,Debug,Copy,Clone,Eq,PartialOrd,Ord,Hash)] pub struct Size(u16);
#[derive(PartialEq,Debug,Copy,Clone,Eq,PartialOrd,Ord,Hash)] pub struct Address(u32);
#[derive(PartialEq,Debug,Copy,Clone,Eq,PartialOrd,Ord,Hash)] pub struct ConstantPoolIndex(u16);
#[derive(PartialEq,Debug,Copy,Clone,Eq,PartialOrd,Ord,Hash)] pub struct LocalFrameIndex(u16);
#[derive(PartialEq,Debug,Copy,Clone,Eq,PartialOrd,Ord,Hash)] pub struct AddressRange { start: Address, length: usize }

impl Arity {
    pub fn new(value: u8)  -> Arity {
        Arity(value)
    }
}
impl Size {
    #[allow(dead_code)]
    pub fn new(value: u16) -> Size {
        Size(value)
    }
}
impl LocalFrameIndex {
    #[allow(dead_code)]
    pub fn new(value: u16) -> LocalFrameIndex   {
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
    pub fn as_usize(&self) -> usize {
        self.value() as usize
    }
}

impl From<u16> for ConstantPoolIndex {
    fn from(n: u16) -> Self {
        ConstantPoolIndex(n)
    }
}

impl From<usize> for ConstantPoolIndex {
    fn from(n: usize) -> Self {
        assert!(n <= 65535usize);
        ConstantPoolIndex(n as u16)
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
        Size(serializable::read_u16(input))
    }
}

impl Serializable for Address {
    fn serialize<W: Write> (&self, sink: &mut W) -> anyhow::Result<()> {
        serializable::write_u32(sink, self.0)
    }
    fn from_bytes<R: Read>(input: &mut R) -> Self {
        Address(serializable::read_u32(input))
    }
}

impl Serializable for ConstantPoolIndex {
    fn serialize<W: Write> (&self, sink: &mut W) -> anyhow::Result<()> {
        serializable::write_u16(sink, self.0)
    }
    fn from_bytes<R: Read>(input: &mut R) -> Self {
        ConstantPoolIndex(serializable::read_u16(input))
    }
}

impl Serializable for LocalFrameIndex {
    fn serialize<W: Write> (&self, sink: &mut W) -> anyhow::Result<()> {
        serializable::write_u16(sink, self.0)
    }
    fn from_bytes<R: Read>(input: &mut R) -> Self {
        LocalFrameIndex(serializable::read_u16(input))
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{number:>0width$}", number=self.0, width=4)
    }
}

impl std::fmt::Display for ConstantPoolIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl std::fmt::Display for LocalFrameIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "::{}", self.0)
    }
}

impl std::fmt::Display for Arity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for AddressRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}-{}", self.start,
               Address::from_usize(self.start.value_usize() + self.length - 1))
    }
}

impl Size {
    pub fn make_vector<T>(&self, element: T) -> Vec<T> where T: Clone {
        repeat(element).take(self.to_usize()).collect::<Vec<T>>()
    }
}