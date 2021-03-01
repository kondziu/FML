use crate::bytecode::program::*;
use crate::bytecode::heap::*;
use std::collections::HashMap;

use anyhow::*;

// TODO anyhow has ensure which will replace bailf_if

pub struct InstructionPointer(Option<Address>);
impl InstructionPointer {
    pub fn new() -> Self { InstructionPointer(None) }
    pub fn bump(&mut self, program: &Program) {
        if let Some(address) = self.0 {
            self.0 = program.code.next(address)
        }
    }
    pub fn set(&mut self, address: Option<Address>) {
        self.0 = address
    }
    pub fn get(&self) -> Option<Address> {
        self.0
    }
}
impl From<Address> for InstructionPointer {
    fn from(address: Address) -> Self { InstructionPointer(Some(address)) }
}
impl From<&Address> for InstructionPointer {
    fn from(address: &Address) -> Self { InstructionPointer(Some(address.clone())) }
}

pub struct OperandStack(Vec<Pointer>);
impl OperandStack {
    pub fn new() -> Self { OperandStack(Vec::new()) }
    pub fn push(&mut self, pointer: Pointer) {
        self.0.push(pointer)
    }
    pub fn pop(&mut self) -> Result<Pointer> {
        self.0.pop().with_context(|| format!("Cannot pop from an empty operand stack."))
    }
    pub fn peek(&self) -> Result<&Pointer> {
        self.0.last().with_context(|| format!("Cannot peek from an empty operand stack."))
    }
    pub fn pop_sequence(&mut self, n: usize) -> Result<Vec<Pointer>> {
        (0..n).map(|_| self.pop()).collect::<Result<Vec<Pointer>>>()
    }
    pub fn pop_reverse_sequence(&mut self, n: usize) -> Result<Vec<Pointer>> {
        (0..n).map(|_| self.pop()).rev().collect::<Result<Vec<Pointer>>>()
    }
}

pub struct Frame { pub(crate) return_address: Option<Address>, locals: Vec<Pointer> }
impl Frame {
    pub fn new(return_address: Option<Address>, locals: Vec<Pointer>) -> Self {
        Frame { locals: Vec::new(), return_address: None }
    }
    pub fn get(&self, index: &LocalFrameIndex) -> Result<&Pointer> { unimplemented!() }
    pub fn set(&mut self, index: &LocalFrameIndex, pointer: Pointer) -> Result<()> { unimplemented!() }
}

pub struct FrameStack { pub globals: GlobalFrame, pub functions: GlobalFunctions, frames: Vec<Frame> }
impl FrameStack {
    pub fn new() -> Self { unimplemented!() }
    pub fn pop(&mut self) -> Result<Frame> { unimplemented!() }
    pub fn push(&mut self, frame: Frame) { unimplemented!() }
    pub fn get_locals(&self) -> Result<&Frame> { unimplemented!() }
    pub fn get_locals_mut(&mut self) -> Result<&mut Frame> { unimplemented!() }
}

pub struct GlobalFunctions(HashMap<String, ConstantPoolIndex>);
impl GlobalFunctions {
    pub fn new() -> Self { GlobalFunctions(HashMap::new()) }
    pub fn get(&self, name: &str) -> Result<&ConstantPoolIndex> {
        self.0.get(name)
            .with_context(|| format!("No such function `{}`.", name))
    }
    pub fn update(&mut self, name: String, index: ConstantPoolIndex) -> Result<()> {
        let result = self.0.insert(name.clone(), index);
        bail_if!(result.is_none(), "No such function `{}`.", name);
        Ok(())
    }
    pub fn define(&mut self, name: String, index: ConstantPoolIndex) -> Result<()> {
        let result = self.0.insert(name.clone(), index);
        bail_if!(result.is_some(), "Cannot define function `{}`: already defined.", name);
        Ok(())
    }
}

pub struct GlobalFrame(HashMap<String, Pointer>);
impl GlobalFrame {
    pub fn new() -> Self { GlobalFrame(HashMap::new()) }
    pub fn get(&self, name: &str) -> Result<&Pointer> {
        self.0.get(name)
            .with_context(|| format!("No such global `{}`.", name))
    }
    pub fn update(&mut self, name: String, pointer: Pointer) -> Result<()> {
        let result = self.0.insert(name.clone(), pointer);
        bail_if!(result.is_none(), "No such global `{}`.", name);
        Ok(())
    }
    pub fn define(&mut self, name: String, pointer: Pointer) -> Result<()> {
        let result = self.0.insert(name.clone(), pointer);
        bail_if!(result.is_some(), "Cannot define global `{}`: already defined.", name);
        Ok(())
    }
}

pub struct State {
    pub operand_stack: OperandStack,
    pub frame_stack: FrameStack,
    pub instruction_pointer: InstructionPointer,
    pub heap: Heap
}

// pub struct State {
//     pub instruction_pointer: Option<Address>,
//     pub frames: Vec<LocalFrame>,
//     pub operands: Vec<Pointer>,
//     pub globals: HashMap<String, Pointer>,
//     pub functions: HashMap<String, ProgramObject>,
//     pub memory: Heap,
// }

impl State {
    pub fn from(program: &Program) -> Result<Self> {                                                // TODO error handling is a right mess here.

        let entry_index = program.entry.get()?;
        let entry_method = program.constant_pool.get(&entry_index)
            .expect(&format!("State init error: entry method is not in the constant pool \
                              at index {:?}", entry_index));

        let instruction_pointer = InstructionPointer::from(match entry_method {
            ProgramObject::Method { name: _, parameters: _, locals: _, code } => code.start(),
            _ => panic!("State init error: entry method is not a Method {:?}", entry_method),
        });

        let mut globals: HashMap<String, Pointer> = HashMap::new();
        let mut functions: HashMap<String, ProgramObject> = HashMap::new();

        for index in program.globals.iter() {
            let thing = program.constant_pool.get(&index)
                .expect(&format!("State init error: no such entry at index pool: {:?}", index));

            match thing {
                ProgramObject::Slot { name: index } => {
                    let constant = program.constant_pool.get(index)
                        .expect(&format!("State init error: no such entry at index pool: {:?} \
                                 (expected by slot: {:?})", index, thing));
                    let name = match constant {
                        ProgramObject::String(string) => string,
                        constant => panic!("State init error: name of global at index {:?} is \
                                            not a String {:?}", index, constant),
                    };
                    if globals.contains_key(name) {
                        panic!("State init error: duplicate name for global {:?}", name)
                    }

                    globals.insert(name.to_string(), Pointer::Null);
                }

                ProgramObject::Method { name: index, parameters: _, locals: _, code: _ } => {
                    let constant = program.constant_pool.get(index)
                        .expect(&format!("State init error: no such entry at index pool: {:?} \
                                 (expected by method: {:?})", index, thing));
                    let name = match constant {
                        ProgramObject::String(string) => string,
                        constant => panic!("State init error: name of function at index {:?} \
                                            is not a String {:?}", index, constant),
                    };
                    if functions.contains_key(name) {
                        panic!("State init error: duplicate name for function {:?}", name)
                    }
                    functions.insert(name.to_string(), thing.clone());
                }
                _ => panic!("State init error: name of global at index {:?} is not a String {:?}",
                            index, thing),
            };
        }

        let frame_stack = FrameStack::new();
        let operand_stack = OperandStack::new();
        let mut heap: Heap = Heap::new();

        Ok(State { operand_stack, frame_stack, instruction_pointer, heap })
    }

    #[allow(dead_code)]
    pub fn new() -> Self {
        State {
            operand_stack: OperandStack::new(),
            frame_stack: FrameStack::new(),
            instruction_pointer: InstructionPointer::new(),
            heap: Heap::new()
        }
    }

    #[allow(dead_code)]
    pub fn minimal() -> Self {
        State {
            operand_stack: OperandStack::new(),
            frame_stack: FrameStack::new(),
            instruction_pointer: InstructionPointer::from(Address::from_usize(0)),
            heap: Heap::new()
        }
    }

    // pub fn bump_instruction_pointer(&mut self, program: &Program) -> &Option<Address> {
    //     let address = program.code().next_address(self.instruction_pointer);
    //     self.instruction_pointer = address;
    //     &self.instruction_pointer
    // }

    // pub fn set_instruction_pointer(&mut self, address: Option<Address>) -> () {
    //     self.instruction_pointer = address;
    // }
    //
    // pub fn has_next_instruction_pointer(&mut self) -> bool {
    //     self.instruction_pointer != None
    // }

    // pub fn current_frame(&self) -> Option<&LocalFrame> {
    //     self.frames.last()
    // }
    //
    // pub fn current_frame_mut(&mut self, ) -> Option<&mut LocalFrame> {
    //     self.frames.last_mut()
    // }
    //
    // pub fn pop_frame(&mut self) -> Option<LocalFrame> {
    //     self.frames.pop()
    // }

    // pub fn new_frame(&mut self, return_address: Option<Address>, slots: Vec<Pointer>, ) {
    //     self.frames.push(LocalFrame { locals: slots, return_address });
    // }
    //
    // pub fn peek_operand(&mut self) -> Option<&Pointer> {
    //     self.operands.last()
    // }

    // pub fn pop_operand(&mut self) -> Option<Pointer> {
    //     self.operands.pop()
    // }
    //
    // pub fn push_operand(&mut self, object: Pointer) {
    //     self.operands.push(object)
    // }
    //
    // pub fn allocate_and_push_operand(&mut self, object: HeapObject) {
    //     self.operands.push(Pointer::from(self.memory.allocate(object)))
    // }
    //
    // pub fn get_function(&self, name: &str) -> Option<&ProgramObject> {
    //     self.functions.get(name)
    // }
    //
    // pub fn get_global(&self, name: &str) -> Option<&Pointer> {
    //     self.globals.get(name)
    // }

    // #[allow(dead_code)]
    // pub fn register_global(&mut self, name: String, object: Pointer) -> Result<(), String> {
    //     if self.globals.contains_key(&name) {
    //         Err(format!("Global {} already registered (with value {:?})",
    //                     &name, self.globals.get(&name).unwrap()))
    //     } else {
    //         self.globals.insert(name, object);
    //         Ok(())
    //     }
    // }

    // #[allow(dead_code)]
    // pub fn allocate_and_register_global(&mut self, name: String, object: HeapObject) -> Result<(), String> {
    //     let pointer = self.allocate(object);
    //     self.register_global(name, pointer)
    // }

    // pub fn update_global(&mut self, name: String, object: Pointer) {
    //     self.globals.insert(name, object);
    // }

    // pub fn set_instruction_pointer_from_label(&mut self, program: &Program, name: &str) -> Result<(), String> {
    //     match program.get_label(name) {
    //         None => Err(format!("Label {} does not exist", name)),
    //         Some(address) => {
    //             self.instruction_pointer = Some(*address);
    //             Ok(())
    //         }
    //     }
    // }

    // #[allow(dead_code)]
    // pub fn push_global_to_operand_stack(&mut self, name: &str) -> Result<(), String> {
    //     let global = self.get_global(name).map(|e| e.clone());
    //     match global {
    //         Some(global) => {
    //             self.push_operand(global);
    //             Ok(())
    //         },
    //         None => {
    //             Err(format!("No such global {}", name))
    //         }
    //     }
    // }

    // pub fn dereference_to_string(&self, pointer: &Pointer) -> String {
    //     self.memory.dereference_to_string(pointer)
    // }
    //
    // pub fn dereference_mut(&mut self, pointer: &HeapIndex) -> Option<&mut HeapObject> {
    //     self.memory.dereference_mut(pointer)
    // }
    //
    // pub fn dereference(&self, pointer: &HeapIndex) -> Option<&HeapObject> {
    //     self.memory.dereference(pointer)
    // }
    //
    // pub fn allocate(&mut self, object: HeapObject) -> Pointer {
    //     Pointer::from(self.memory.allocate(object))
    // }

    // pub fn copy_memory(&mut self, pointer: &Pointer) -> Option<Pointer> {
    //     match pointer {
    //         Pointer::Reference(p) =>
    //             self.memory.copy(p).map(|p| Pointer::from(p)),
    //         tagged_pointer =>
    //             Some(tagged_pointer.clone())
    //     }
    // }
}


#[derive(PartialEq,Debug)]
pub struct LocalFrame {
    locals: Vec<Pointer>, /* ProgramObject::Slot */
    return_address: Option<Address>, /* address */
}

impl LocalFrame {
    pub fn empty() -> Self {
        LocalFrame {
            locals: vec!(),
            return_address: None,
        }
    }

    #[allow(dead_code)]
    pub fn from(return_address: Option<Address>, slots: Vec<Pointer>) -> Self {
        LocalFrame {
            return_address,
            locals: slots,
        }
    }

    pub fn return_address(&self) -> &Option<Address> {
        &self.return_address
    }

    pub fn get_local(&self, index: &LocalFrameIndex) -> Option<Pointer> {
        match index.value() {
            index if index as usize >= self.locals.len() => None,
            index => Some(self.locals[index as usize].clone()), // new ref
        }
    }

    pub fn update_local(&mut self, index: &LocalFrameIndex, local: Pointer) -> Result<(), String> {
        match index.value() {
            index if index as usize >= self.locals.len() =>
                Err(format!("No local at index {} in frame", index)),
            index => {
                self.locals[index as usize] = local;
                Ok(())
            },
        }
    }

    #[allow(dead_code)]
    pub fn push_local(&mut self, local: Pointer) -> LocalFrameIndex {
        self.locals.push(local);
        assert!(self.locals.len() <= 65_535usize);
        LocalFrameIndex::new(self.locals.len() as u16 - 1u16)
    }
}
