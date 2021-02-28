use std::io::{Write, Read};
use std::collections::HashMap;

use super::bytecode::OpCode;
use super::types::*;

use super::serializable;
use super::serializable::*;

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


#[derive(PartialEq,Debug,Clone)]
pub struct Code {
    opcodes: Vec<OpCode>,
}

impl Code {
    pub fn new() -> Code {
        Code { opcodes: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn from(opcodes: Vec<OpCode>) -> Code {
        Code { opcodes }
    }

    #[allow(dead_code)]
    pub fn all_opcodes(&self) -> Vec<(Address, OpCode)> {
        self.opcodes.iter().enumerate().map(|(i, opcode)| {
            (Address::from_usize(i), opcode.clone())
        }).collect()
    }

    pub fn register_opcodes(&mut self, opcodes: Vec<OpCode>) -> AddressRange {
        let start = self.opcodes.len();
        let length = opcodes.len();
        self.opcodes.extend(opcodes);
        AddressRange::new(Address::from_usize(start), length)
    }

    pub fn addresses_to_code_vector(&self, range: &AddressRange) -> Vec<&OpCode> {
        let start = range.start().value_usize();
        let end = start + range.length();
        let mut result: Vec<&OpCode> = Vec::new();
        for i in start..end {
            result.push(&self.opcodes[i]);
        }
        result
    }

    pub fn next_address(&self, address: Option<Address>) -> Option<Address> {
        match address {
            Some(address) => {
                let new_address = Address::from_usize(address.value_usize() + 1);
                if self.opcodes.len() > new_address.value_usize() {
                    Some(new_address)
                } else {
                    None
                }
            }
            None => panic!("Cannot advance a nothing address.")
        }
    }

    pub fn get_opcode(&self, address: &Address) -> Option<&OpCode> {
        //self.table[address.value() as usize]
        self.opcodes.get(address.value_usize())
    }

    // pub fn dump(&self) { // TODO pretty print
    //     for (i, opcode) in self.opcodes.iter().enumerate() {
    //         println!("{}: {:?}", i, opcode);
    //     }
    // }
}

#[derive(PartialEq,Debug,Clone)]
pub struct Labels {
    labels: HashMap<String, Address>,
    groups: usize,
}

impl Labels {
    pub fn new() -> Self {
        Labels { labels: HashMap::new(), groups: 0 }
    }
    pub fn from(labels: HashMap<String, Address>) -> Self {
        let groups = labels.iter().flat_map(|(label, _)| {
            label.split(":").last().map(|s| {
                s.parse::<usize>().map_or(None, |n| Some(n))
            }).flatten()
        }).max().map_or(0, |n| n + 1);
        Labels { labels, groups }
    }
    pub fn generate_label<S>(&mut self, name: S) -> Option<String> where S: Into<String> {
        let label = format!("{}:{}", name.into(), self.groups);
        if self.labels.contains_key(&label) {
            None
        } else {
            Some(label)
        }
    }
    pub fn register_label_address<S>(&mut self, name: S, address: Address) -> Option<Address> where S: Into<String> {
        self.labels.insert(name.into(), address)
    }
    pub fn new_group(&mut self) {
        self.groups = self.groups + 1
    }
    pub fn get_label_address(&self, name: &str) -> Option<&Address> {
        self.labels.get(name)
    }
    pub fn all(&self) -> &HashMap<String, Address> {
        &self.labels
    }
}

#[derive(PartialEq,Debug,Clone)]
pub struct Program {
    code: Code,
    labels: Labels,
    constants: Vec<ProgramObject>,
    globals: Vec<ConstantPoolIndex>,
    entry: ConstantPoolIndex,
}

impl Program {
    #[allow(dead_code)]
    pub fn new(code: Code,
               constants: Vec<ProgramObject>,
               globals: Vec<ConstantPoolIndex>,
               entry: ConstantPoolIndex) -> Program {

        let labels = Program::labels_from_code(&code, &constants);

        Program { code, labels, constants, globals, entry }
    }

    pub fn empty() -> Program {
        Program {
            code: Code::new(),
            labels: Labels::new(),
            constants: Vec::new(),
            globals: Vec::new(),
            entry: ConstantPoolIndex::new(0) // FIXME
        }
    }

    fn labels_from_code(code: &Code, constants: &Vec<ProgramObject>) -> Labels {
        let mut labels: HashMap<String, Address> = HashMap::new();
        for (i, opcode) in code.opcodes.iter().enumerate() {
            if let OpCode::Label { name: index } = opcode {
                let constant = constants.get(index.value() as usize)
                    .expect(&format!("Program initialization: label {:?} expects a constant in the \
                                      constant pool at index {:?} but none was found",
                                     opcode, index));

                let name = match constant {
                    ProgramObject::String(string) => string,
                    _ => panic!("Program initialization: label {:?} expects a String in the \
                                 constant pool at index {:?} but {:?} was found",
                                opcode, index, constant),
                };

                if labels.contains_key(name) {
                    panic!("Program initialization: attempt to define label {:?} with a non-unique \
                            name: {}", opcode, name)
                }

                labels.insert(name.to_string(), Address::from_usize(i));
            };
        }
        Labels::from(labels)
    }

    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn constants(&self) -> &Vec<ProgramObject> {
        &self.constants
    }

    pub fn labels(&self) -> &HashMap<String, Address> {
        &self.labels.all()
    }

    pub fn globals(&self) -> &Vec<ConstantPoolIndex> {
        &self.globals
    }

    pub fn entry(&self) -> &ConstantPoolIndex {
        &self.entry
    }

    pub fn get_constant(&self, index: &ConstantPoolIndex) -> Option<&ProgramObject> {
        self.constants.get(index.value() as usize)
    }

    pub fn get_opcode(&self, address: &Address) -> Option<&OpCode> {
        self.code.get_opcode(address)
    }

    pub fn get_label(&self, name: &str) -> Option<&Address> {
        self.labels.get_label_address(name)
    }

    //-----------

    pub fn register_constant(&mut self, constant: ProgramObject) -> ConstantPoolIndex {
        match self.constants.iter().position(|c| *c == constant) {
            Some(position) => ConstantPoolIndex::from_usize(position),
            None => {
                let index = ConstantPoolIndex::from_usize(self.constants.len());
                self.constants.push(constant);
                index
            }
        }
    }

    pub fn register_global(&mut self, constant: ConstantPoolIndex) {
        if self.globals.contains(&constant) {
            panic!("Cannot register global {:?}, this index is already registered.", constant)
        }

        self.globals.push(constant)
    }

//    fn register_label(&mut self, label: String) -> ConstantPoolIndex {
//        if let Some(index) = self.labels.get(&label) {
//            return *index;
//        }
//        let index = ConstantPoolIndex::from_usize(self.labels.len());
//        self.labels.insert(label, index);
//        index
//    }

    // pub fn generate_new_label_name(&mut self, name: &str) -> ConstantPoolIndex {
    //     let label = self.labels.generate_label(name).unwrap();
    //     self.labels.new_group();
    //     let constant = ProgramObject::String(label);
    //     let index = self.register_constant(constant);
    //
    //     index
    // }

    pub fn generate_new_label_names(&mut self, names: Vec<&str>) -> Vec<ConstantPoolIndex> {
        let labels: Vec<String> = names.into_iter()
            .map(|name| self.labels.generate_label(name))
            .map(|label| label.unwrap())
            .collect();

        self.labels.new_group();

        labels.into_iter()
            .map(|label| {
                self.register_constant(ProgramObject::String(label.clone()))
            })
            .collect()
    }

    pub fn get_current_address(&self) -> Address {
        let size = self.code.opcodes.len();
        Address::from_usize(size - 1)
    }

    pub fn get_upcoming_address(&self) -> Address {
        let size = self.code.opcodes.len();
        Address::from_usize(size)
    }

    pub fn set_entry(&mut self, function_index: ConstantPoolIndex) {
        self.entry = function_index;
    }

    pub fn emit_conditionally(&mut self, opcode: OpCode, emit: bool) {
        if emit { self.emit_code(opcode) }
    }

    pub fn emit_code(&mut self, opcode: OpCode) {
        // println!("Emitting code: {:?}", opcode);
        match opcode {
            OpCode::Label {name: index} => {
                let address = Address::from_usize(self.code.opcodes.len());
                self.code.opcodes.push(opcode);
                let constant = self.get_constant(&index);
                match constant {
                    Some(ProgramObject::String(name)) => {
                        let name = name.to_owned();
                        let result = self.labels.register_label_address(name, address);

                        if result.is_some() {
                             panic!("Emit code error: cannot create label {:?}, \
                                               name {:?} already used by another label.",
                                                   opcode, self.get_constant(&index))
                        }
                    },
                    Some(object) => panic!("Emit code error: cannot create label, \
                                            constant at index {:?} should be a String, but is {:?}",
                                            index, object),

                    None => panic!("Emit code error: cannot create label, \
                                    there is no constant at index {:?}", index),
                }

            }
            _ => self.code.opcodes.push(opcode),
        }
    }
}

impl Serializable for Program {
    fn serialize<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {

        serializable::write_usize_as_u16(sink, self.constants.len())?;
        for constant in self.constants.iter() {
            constant.serialize(sink, self.code())?;
        }

        ConstantPoolIndex::write_cpi_vector(sink, &self.globals)?;

        self.entry.serialize(sink)
    }

    fn from_bytes<R: Read>(input: &mut R) -> Self {
        let mut code = Code::new();

        let size = serializable::read_u16_as_usize(input);
        let mut constants: Vec<ProgramObject> = Vec::new();
        for _ in 0..size {
            constants.push(ProgramObject::from_bytes(input, &mut code))
        }

        let globals = ConstantPoolIndex::read_cpi_vector(input);
        let entry = ConstantPoolIndex::from_bytes(input);
        let labels = Program::labels_from_code(&code, &constants);

        Program { code, constants, globals, entry, labels }
    }
}
