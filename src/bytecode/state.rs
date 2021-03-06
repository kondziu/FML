use crate::bytecode::program::*;
use crate::bytecode::heap::*;
use std::collections::{HashMap, HashSet};

use anyhow::*;
use std::io::Write as IOWrite;

// TODO anyhow has ensure which will replace bailf_if

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy)]
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
impl From<u32> for InstructionPointer {
    fn from(n: u32) -> Self { InstructionPointer(Some(Address::from_u32(n))) }
}
impl From<usize> for InstructionPointer {
    fn from(n: usize) -> Self { InstructionPointer(Some(Address::from_usize(n))) }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
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
    #[allow(dead_code)]
    pub fn pop_sequence(&mut self, n: usize) -> Result<Vec<Pointer>> {
        (0..n).map(|_| self.pop()).collect::<Result<Vec<Pointer>>>()
    }
    pub fn pop_reverse_sequence(&mut self, n: usize) -> Result<Vec<Pointer>> {
        (0..n).map(|_| self.pop()).rev().collect::<Result<Vec<Pointer>>>()
    }
}

impl From<Vec<Pointer>> for OperandStack {
    fn from(vector: Vec<Pointer>) -> Self {
        OperandStack(vector)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct Frame { pub(crate) return_address: Option<Address>, locals: Vec<Pointer> }
impl Frame {
    pub fn new() -> Self {
        Frame { locals: Vec::new(), return_address: None }
    }
    pub fn with_capacity(return_address: Option<Address>, size: usize, initial: Pointer) -> Self {
        Frame { locals: (0..size).map(|_| initial.clone()).collect(), return_address }
    }
    pub fn from(return_address: Option<Address>, locals: Vec<Pointer>) -> Self {
        Frame { locals, return_address }
    }
    pub fn get(&self, index: &LocalFrameIndex) -> Result<&Pointer> {
        let index = index.value() as usize;
        if index >= self.locals.len() {
            bail!("Local frame index {} out of range (0..{})", index, self.locals.len());
        }
        Ok(&self.locals[index])
    }
    pub fn set(&mut self, index: &LocalFrameIndex, pointer: Pointer) -> Result<()> {
        let index = index.value() as usize;
        if index >= self.locals.len() {
            bail!("Local frame index {} out of range (0..{})", index, self.locals.len());
        }
        //println!("set {} {} <- {}", index, self.locals[index], pointer);
        self.locals[index] = pointer;
        //println!("= set {} {}", index, self.locals[index]);
        Ok(())
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct FrameStack { pub globals: GlobalFrame, pub functions: GlobalFunctions, frames: Vec<Frame> }
impl FrameStack {
    pub fn new() -> Self {
        FrameStack {
            globals: GlobalFrame::new(),
            functions: GlobalFunctions::new(),
            frames: Vec::new()}
    }
    pub fn pop(&mut self) -> Result<Frame> {
        self.frames.pop().with_context(|| format!("Attempting to pop frame from empty stack."))
    }
    pub fn push(&mut self, frame: Frame) {
        self.frames.push(frame)
    }
    pub fn get_locals(&self) -> Result<&Frame> {
        self.frames.last()
            .with_context(|| format!("Attempting to access frame from empty stack."))
    }
    pub fn get_locals_mut(&mut self) -> Result<&mut Frame> {
        self.frames.last_mut()
            .with_context(|| format!("Attempting to access frame from empty stack."))
    }
}

impl From<(GlobalFrame, GlobalFunctions)> for FrameStack {
    fn from((globals, functions): (GlobalFrame, GlobalFunctions)) -> Self {
        FrameStack { globals, functions, frames: Vec::new() }
    }
}

impl From<Frame> for FrameStack {
    fn from(frame: Frame) -> Self {
        FrameStack {
            globals: GlobalFrame::new(),
            functions: GlobalFunctions::new(),
            frames: vec![frame]
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct GlobalFunctions(HashMap<String, ConstantPoolIndex>);
impl GlobalFunctions {
    pub fn new() -> Self { GlobalFunctions(HashMap::new()) }
    pub fn get(&self, name: &str) -> Result<&ConstantPoolIndex> {
        self.0.get(name)
            .with_context(|| format!("No such function `{}`.", name))
    }
    #[allow(dead_code)]
    pub fn update(&mut self, name: String, index: ConstantPoolIndex) -> Result<()> {
        let result = self.0.insert(name.clone(), index);
        bail_if!(result.is_none(), "No such function `{}`.", name);
        Ok(())
    }
    #[allow(dead_code)]
    pub fn define(&mut self, name: String, index: ConstantPoolIndex) -> Result<()> {
        let result = self.0.insert(name.clone(), index);
        bail_if!(result.is_some(), "Cannot define function `{}`: already defined.", name);
        Ok(())
    }
    pub fn from(methods: Vec<(String, ConstantPoolIndex)>) -> Result<Self> {
        let mut unique = HashSet::new();
        let functions = methods.into_iter()
            .map(|(name, index)| {
                if unique.insert(name.clone()) {
                    Ok((name, index))
                } else {
                    Err(anyhow!("Function is a duplicate: {}", name))
                }
            })
            .collect::<Result<HashMap<String, ConstantPoolIndex>>>()?;
        Ok(GlobalFunctions(functions))
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct GlobalFrame(HashMap<String, Pointer>);
impl GlobalFrame {
    pub fn new() -> Self { GlobalFrame(HashMap::new()) }
    pub fn get(&self, name: &str) -> Result<&Pointer> {
        self.0.get(name)
            .with_context(|| format!("No such global `{}`.", name))
    }
    #[allow(dead_code)]
    pub fn update(&mut self, name: String, pointer: Pointer) -> Result<()> {
        let result = self.0.insert(name.clone(), pointer);
        bail_if!(result.is_none(), "No such global `{}`.", name);
        Ok(())
    }
    #[allow(dead_code)]
    pub fn define(&mut self, name: String, pointer: Pointer) -> Result<()> {
        let result = self.0.insert(name.clone(), pointer);
        bail_if!(result.is_some(), "Cannot define global `{}`: already defined.", name);
        Ok(())
    }
    pub fn from(names: Vec<String>, initial: Pointer) -> Result<Self> {
        let mut unique = HashSet::new();
        let globals = names.into_iter()
            .map(|name| {
                if unique.insert(name.clone()) {
                    Ok((name, initial.clone()))
                } else {
                    Err(anyhow!("Global is a duplicate: {}", name))
                }
            })
            .collect::<Result<HashMap<String, Pointer>>>()?;
        Ok(GlobalFrame(globals))
    }
}

#[derive(Eq, PartialEq, Debug)]
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

        let entry_index = program.entry.get()
            .with_context(|| format!("Cannot find entry method."))?;
        let entry_method = program.constant_pool.get(&entry_index)
            .with_context(|| format!("Cannot find entry method."))?;

        let instruction_pointer =
            InstructionPointer::from(entry_method.get_method_start_address()?);

        let global_objects = program.globals.iter()
            .map(|index| {
                program.constant_pool.get(&index).map(|object| (index, object))
            })
            .collect::<Result<Vec<(ConstantPoolIndex, &ProgramObject)>>>()?;

        ensure!(global_objects.iter().all(|(_, object)| object.is_slot() || object.is_method()),
                "Illegal global constant: expecting Method or Slot.");

        fn extract_slot(program: &Program, slot: &ProgramObject) -> Result<String> {
            let name_index = slot.as_slot_index()?;
            let name_object = program.constant_pool.get(name_index)?;
            let name = name_object.as_str()?;
            Ok(name.to_owned())
        }

        let globals = global_objects.iter()
            .filter(|(_, program_object)| program_object.is_slot())
            .map(|(_, slot)| extract_slot(program, slot))
            .collect::<Result<Vec<String>>>()?;

        fn extract_function(program: &Program, index: &ConstantPoolIndex, method: &ProgramObject) -> Result<(String, ConstantPoolIndex)> {
            let name_index = method.get_method_name()?;
            let name_object = program.constant_pool.get(name_index)?;
            let name = name_object.as_str()?;
            Ok((name.to_owned(), index.clone()))
        }

        let functions = global_objects.iter()
            .filter(|(_, program_object)| program_object.is_method())
            .map(|(index, method)| extract_function(program, index, method))
            .collect::<Result<Vec<(String, ConstantPoolIndex)>>>()?;

        let global_frame = GlobalFrame::from(globals, Pointer::Null)?;
        let global_functions = GlobalFunctions::from(functions)?;
        let frame_stack = FrameStack::from((global_frame, global_functions));

        let operand_stack = OperandStack::new();
        let heap: Heap = Heap::new();

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
            frame_stack: FrameStack::from(Frame::new()),
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


// #[derive(PartialEq,Debug)]
// pub struct LocalFrame {
//     locals: Vec<Pointer>, /* ProgramObject::Slot */
//     return_address: Option<Address>, /* address */
// }
//
// impl LocalFrame {
//     pub fn empty() -> Self {
//         LocalFrame {
//             locals: vec!(),
//             return_address: None,
//         }
//     }
//
//     #[allow(dead_code)]
//     pub fn from(return_address: Option<Address>, slots: Vec<Pointer>) -> Self {
//         LocalFrame {
//             return_address,
//             locals: slots,
//         }
//     }
//
//     pub fn return_address(&self) -> &Option<Address> {
//         &self.return_address
//     }
//
//     pub fn get_local(&self, index: &LocalFrameIndex) -> Option<Pointer> {
//         match index.value() {
//             index if index as usize >= self.locals.len() => None,
//             index => Some(self.locals[index as usize].clone()), // new ref
//         }
//     }
//
//     pub fn update_local(&mut self, index: &LocalFrameIndex, local: Pointer) -> Result<(), String> {
//         match index.value() {
//             index if index as usize >= self.locals.len() =>
//                 Err(format!("No local at index {} in frame", index)),
//             index => {
//                 self.locals[index as usize] = local;
//                 Ok(())
//             },
//         }
//     }
//
//     #[allow(dead_code)]
//     pub fn push_local(&mut self, local: Pointer) -> LocalFrameIndex {
//         self.locals.push(local);
//         assert!(self.locals.len() <= 65_535usize);
//         LocalFrameIndex::new(self.locals.len() as u16 - 1u16)
//     }
// }

pub struct Output();

impl Output {
    pub fn new() -> Self { Output() }
}

impl std::fmt::Write for Output {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match std::io::stdout().write_all(s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(std::fmt::Error),
        }
    }
}