use std::io::{Write, Read};
use std::collections::HashMap;

use super::bytecode::OpCode;
use super::objects::ProgramObject;
use super::types::*;

use super::serializable;
use super::serializable::*;

/**
 * The instruction pointer contains the address of the instruction that will be executed next.
 */
//pub struct InstructionPointer(u64);

/**
 * A listing of program objects. They can be referred to by their numerical index.
 */
//#[derive(PartialEq,Debug,Clone)]
//pub struct ConstantPool {
//    constants: Vec<ProgramObject>,
//}

//impl ConstantPool {
//    pub fn new(constants: Vec<ProgramObject>) -> ConstantPool {
//        ConstantPool {constants}
//    }
//}

//impl Serializable for ConstantPool {
//    fn serialize<W: Write>(&self, sink: &mut W) -> () {
//        serializable::write_usize_as_u16(sink, self.constants.len());
//        for constant in self.constants.iter() {
//            constant.serialize(sink)
//        }
//    }
//
//    fn from_bytes<R: Read>(input: &mut R) -> Self {
//        println!("ConstantPool::from_bytes");
//        let size = serializable::read_u16_as_usize(input);
//        let mut constants: Vec<ProgramObject> = Vec::new();
//        for _ in 0..size {
//            constants.push(ProgramObject::from_bytes(input))
//        }
//        ConstantPool { constants }
//    }
//}

/**
 * A listing of global variables and functions in the program. Each listing points to an object in
 * the constant pool and is **guaranteed** to refer to either:
 *   - a `ProgramObject::Slot` object, or
 *   - a `ProgramObject:Method` object.
 */
//#[derive(PartialEq,Debug,Clone)]
//pub struct GlobalSlots {
//    slots: Vec<ConstantPoolIndex>,
//}

//impl GlobalSlots {
//    pub fn new(slots: Vec<ConstantPoolIndex>) -> GlobalSlots {
//        GlobalSlots {slots}
//    }
//}
//
//impl Serializable for GlobalSlots {
//    fn serialize<W: Write>(&self, sink: &mut W) -> () {
//        ConstantPoolIndex::write_cpi_vector(sink, &self.slots)
//    }
//
//    fn from_bytes<R: Read>(input: &mut R) -> Self {
//        println!("GlobalSlots::from_bytes");
//        GlobalSlots { slots: ConstantPoolIndex::read_cpi_vector(input) }
//    }
//}

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
            label.split("_").last().map(|s| {
                s.parse::<usize>().map_or(None, |n| Some(n))
            }).flatten()
        }).max().map_or(0, |n| n + 1);
        Labels { labels, groups }
    }
    pub fn generate_label<S>(&mut self, name: S) -> Option<String> where S: Into<String> {
        let label = format!("{}_{}", name.into(), self.groups);
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
