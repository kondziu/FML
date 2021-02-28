use crate::bytecode::program::{ProgramObject, Program};
use crate::bytecode::heap::{Pointer, Heap, HeapObject, HeapIndex};
use std::collections::HashMap;
use crate::bytecode::types::{Address, LocalFrameIndex};

pub struct State {
    pub instruction_pointer: Option<Address>,
    pub frames: Vec<LocalFrame>,
    pub operands: Vec<Pointer>,
    pub globals: HashMap<String, Pointer>,
    pub functions: HashMap<String, ProgramObject>,
    pub memory: Heap,
}

impl State {
    pub fn from(program: &Program) -> Self {

        let entry_index = program.entry();
        let entry_method = program.get_constant(entry_index)
            .expect(&format!("State init error: entry method is not in the constant pool \
                              at index {:?}", entry_index));

        let instruction_pointer = *match entry_method {
            ProgramObject::Method { name: _, parameters: _, locals: _, code } => code.start(),
            _ => panic!("State init error: entry method is not a Method {:?}", entry_method),
        };

        let mut globals: HashMap<String, Pointer> = HashMap::new();
        let mut functions: HashMap<String, ProgramObject> = HashMap::new();
        let mut memory: Heap = Heap::new();

        for index in program.globals() {
            let thing = program.get_constant(index)
                .expect(&format!("State init error: no such entry at index pool: {:?}", index));

            match thing {
                ProgramObject::Slot { name: index } => {
                    let constant = program.get_constant(index)
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
                    let constant = program.get_constant(index)
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

        let frames = vec!(LocalFrame::empty());

        State {
            instruction_pointer: Some(instruction_pointer),
            frames,
            operands: Vec::new(),
            globals,
            functions,
            memory,
        }
    }

    #[allow(dead_code)]
    pub fn empty() -> Self {
        State {
            instruction_pointer: None,
            frames: Vec::new(),
            operands: Vec::new(),
            globals: HashMap::new(),
            functions: HashMap::new(),
            memory: Heap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn minimal() -> Self {
        State {
            instruction_pointer: Some(Address::from_usize(0)),
            frames: vec!(LocalFrame::empty()),
            operands: Vec::new(),
            globals: HashMap::new(),
            functions: HashMap::new(),
            memory: Heap::new(),
        }
    }

    pub fn instruction_pointer(&self) -> &Option<Address> {
        &self.instruction_pointer
    }

    pub fn bump_instruction_pointer(&mut self, program: &Program) -> &Option<Address> {
        let address = program.code().next_address(self.instruction_pointer);
        self.instruction_pointer = address;
        &self.instruction_pointer
    }

    pub fn set_instruction_pointer(&mut self, address: Option<Address>) -> () {
        self.instruction_pointer = address;
    }

    pub fn has_next_instruction_pointer(&mut self) -> bool {
        self.instruction_pointer != None
    }

    pub fn current_frame(&self) -> Option<&LocalFrame> {
        self.frames.last()
    }

    pub fn current_frame_mut(&mut self, ) -> Option<&mut LocalFrame> {
        self.frames.last_mut()
    }

    pub fn pop_frame(&mut self) -> Option<LocalFrame> {
        self.frames.pop()
    }

    pub fn new_frame(&mut self, return_address: Option<Address>, slots: Vec<Pointer>, ) {
        self.frames.push(LocalFrame { locals: slots, return_address });
    }

    pub fn peek_operand(&mut self) -> Option<&Pointer> {
        self.operands.last()
    }

    pub fn pop_operand(&mut self) -> Option<Pointer> {
        self.operands.pop()
    }

    pub fn push_operand(&mut self, object: Pointer) {
        self.operands.push(object)
    }

    pub fn allocate_and_push_operand(&mut self, object: HeapObject) {
        self.operands.push(Pointer::from(self.memory.allocate(object)))
    }

    pub fn get_function(&self, name: &str) -> Option<&ProgramObject> {
        self.functions.get(name)
    }

    pub fn get_global(&self, name: &str) -> Option<&Pointer> {
        self.globals.get(name)
    }

    #[allow(dead_code)]
    pub fn register_global(&mut self, name: String, object: Pointer) -> Result<(), String> {
        if self.globals.contains_key(&name) {
            Err(format!("Global {} already registered (with value {:?})",
                        &name, self.globals.get(&name).unwrap()))
        } else {
            self.globals.insert(name, object);
            Ok(())
        }
    }

    #[allow(dead_code)]
    pub fn allocate_and_register_global(&mut self, name: String, object: HeapObject) -> Result<(), String> {
        let pointer = self.allocate(object);
        self.register_global(name, pointer)
    }

    pub fn update_global(&mut self, name: String, object: Pointer) {
        self.globals.insert(name, object);
    }

    pub fn set_instruction_pointer_from_label(&mut self, program: &Program, name: &str) -> Result<(), String> {
        match program.get_label(name) {
            None => Err(format!("Label {} does not exist", name)),
            Some(address) => {
                self.instruction_pointer = Some(*address);
                Ok(())
            }
        }
    }

    #[allow(dead_code)]
    pub fn push_global_to_operand_stack(&mut self, name: &str) -> Result<(), String> {
        let global = self.get_global(name).map(|e| e.clone());
        match global {
            Some(global) => {
                self.push_operand(global);
                Ok(())
            },
            None => {
                Err(format!("No such global {}", name))
            }
        }
    }

    pub fn dereference_to_string(&self, pointer: &Pointer) -> String {
        self.memory.dereference_to_string(pointer)
    }



    pub fn dereference_mut(&mut self, pointer: &HeapIndex) -> Option<&mut HeapObject> {
        self.memory.dereference_mut(pointer)
    }

    pub fn dereference(&self, pointer: &HeapIndex) -> Option<&HeapObject> {
        self.memory.dereference(pointer)
    }

    pub fn allocate(&mut self, object: HeapObject) -> Pointer {
        Pointer::from(self.memory.allocate(object))
    }

    pub fn copy_memory(&mut self, pointer: &Pointer) -> Option<Pointer> {
        match pointer {
            Pointer::Reference(p) =>
                self.memory.copy(p).map(|p| Pointer::from(p)),
            tagged_pointer =>
                Some(tagged_pointer.clone())
        }
    }

    // #[allow(dead_code)]
    // pub fn pass_by_value_or_reference(&mut self, pointer: &Pointer) -> Option<Pointer> {
    //     let object = self.dereference(pointer).map(|e| e.clone());
    //
    //     if object.is_none() {
    //         return None
    //     }
    //
    //     let pass_by_value = object.as_ref().map_or(false, |e| match e {
    //         HeapObject::Object { parent:_, methods:_, fields:_ } => false,
    //         HeapObject::Array(_) => false,
    //         HeapObject::Integer(_) => true,
    //         HeapObject::Boolean(_) => true,
    //         HeapObject::Null => true,
    //     });
    //
    //     if pass_by_value {
    //         Some(self.allocate(object.unwrap()))
    //     } else {
    //         Some(*pointer)
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
