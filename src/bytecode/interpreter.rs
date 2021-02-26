use std::collections::{HashMap, VecDeque};
use std::fmt::{Write, Error};
use std::io::Write as IOWrite;

use super::types::*;
use super::objects::*;
use super::bytecode::*;
use super::program::Program;

use anyhow;
use crate::bytecode::objects::RuntimeObject;

pub struct Output {}

impl Output {
    fn new() -> Output {
        Output {}
    }
}

impl Write for Output {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        match std::io::stdout().write_all(s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error),
        }
    }
}

pub fn evaluate(program: &Program) {
    let mut state = State::from(program);
    let mut output = Output::new();

    let (start_address, locals) = match program.get_constant(program.entry()) {
        Some(ProgramObject::Method { name:_, locals, arguments:_, code }) => (*code.start(), locals),
        None => panic!("No entry method at index {:?}", program.entry()),
        Some(constant) => panic!("Constant at index {:?} is not a method {:?}",
                                  program.entry(), constant),
    };

    let mut slots = Vec::new();
    for _ in 0..locals.to_usize() {
        slots.push(state.allocate(RuntimeObject::Null));
    }

    state.new_frame(None, slots);
    state.set_instruction_pointer(Some(start_address));
    while state.has_next_instruction_pointer() {
        interpret(&mut state, &mut output, program);
    }
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

#[derive(PartialEq,Debug)]
pub struct Heap(Vec<RuntimeObject>);

impl Heap {
    pub fn new() -> Self {
        Heap(Vec::new())
    }

    #[allow(dead_code)]
    pub fn from(objects: Vec<RuntimeObject>) -> Self {
        Heap(objects)
    }

    pub fn allocate(&mut self, object: RuntimeObject) -> Pointer {
        let pointer = Pointer::from(self.0.len());
        self.0.push(object);
        pointer
    }

    pub fn dereference(&self, pointer: &Pointer) -> Option<&RuntimeObject> {
        let index = pointer.as_usize();
        if self.0.len() > index {
            Some(&self.0[index])
        } else {
            None
        }
    }

    pub fn dereference_mut(&mut self, pointer: &Pointer) -> Option<&mut RuntimeObject> {
        let index = pointer.as_usize();
        if self.0.len() > index {
            Some(&mut self.0[index])
        } else {
            None
        }
    }

    pub fn copy(&mut self, pointer: &Pointer) -> Option<Pointer> {
        self.dereference(pointer)
            .map(|object| object.clone())
            .map(|object| self.allocate(object))
    }

    #[allow(dead_code)]
    pub fn write_over(&mut self, pointer: Pointer, object: RuntimeObject) -> anyhow::Result<()> {
        let index = pointer.as_usize();
        if index < self.0.len() {
            anyhow::bail!("Expected an object at {:?} to write over, but none was found", pointer)
        }
        self.0.push(object.clone());
        Ok(())
    }

    pub fn dereference_to_string(&self, pointer: &Pointer) -> String {
        let object = self.dereference(&pointer)
            .expect(&format!("Expected object at {:?} to convert to string, but none was found",
                              pointer));

        match object {
            RuntimeObject::Null => "null".to_string(),
            RuntimeObject::Integer(n) => n.to_string(),
            RuntimeObject::Boolean(b) => b.to_string(),
            RuntimeObject::Array(elements) => {
                let element_string = elements.iter()
                    .map(|p| self.dereference_to_string(p))
                    .collect::<Vec<String>>()
                    .join(", ");

                format!("[{}]", element_string)
            },
            RuntimeObject::Object { parent, fields, methods:_ } => {
                let parent_string = self.dereference_to_string(parent);
                let parent_string = if parent_string == "null" {
                    String::new()
                } else {
                    format!("..={}{}", parent_string, if fields.len() == 0 { "" } else { ", " })
                };
                let fields_string = fields.iter()
                    .map(|(name, field)| {
                        format!("{}={}", name, self.dereference_to_string(field))
                    })
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("object({}{})", parent_string, fields_string)
            }
        }
    }
}

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
            ProgramObject::Method { name: _, arguments: _, locals: _, code } => code.start(),
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

                    let pointer = memory.allocate(RuntimeObject::Null);
                    globals.insert(name.to_string(), pointer);
                }

                ProgramObject::Method { name: index, arguments: _, locals: _, code: _ } => {
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

    pub fn allocate_and_push_operand(&mut self, object: RuntimeObject) {
        self.operands.push(self.memory.allocate(object))
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
    pub fn allocate_and_register_global(&mut self, name: String, object: RuntimeObject) -> Result<(), String> {
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

    pub fn dereference_mut(&mut self, pointer: &Pointer) -> Option<&mut RuntimeObject> {
        self.memory.dereference_mut(pointer)
    }

    pub fn dereference(&self, pointer: &Pointer) -> Option<&RuntimeObject> {
        self.memory.dereference(pointer)
    }

    pub fn allocate(&mut self, object: RuntimeObject) -> Pointer {
        self.memory.allocate(object)
    }

    pub fn copy_memory(&mut self, pointer: &Pointer) -> Option<Pointer> {
        self.memory.copy(pointer)
    }

    #[allow(dead_code)]
    pub fn pass_by_value_or_reference(&mut self, pointer: &Pointer) -> Option<Pointer> {
        let object = self.dereference(pointer).map(|e| e.clone());

        if object.is_none() {
            return None
        }

        let pass_by_value = object.as_ref().map_or(false, |e| match e {
            RuntimeObject::Object { parent:_, methods:_, fields:_ } => false,
            RuntimeObject::Array(_) => false,
            RuntimeObject::Integer(_) => true,
            RuntimeObject::Boolean(_) => true,
            RuntimeObject::Null => true,
        });

        if pass_by_value {
            Some(self.allocate(object.unwrap()))
        } else {
            Some(*pointer)
        }
    }
}

pub fn interpret<Output>(state: &mut State, output: &mut Output, /*memory: &mut Memory,*/ program: &Program)
    where /*Input : Read,*/ Output : Write {

    // println!("Stack:");
    // for pointer in state.operands.iter() {
    //     println!("  {:?}: {:?}", pointer, state.memory.objects.get(&pointer));
    // }
    // println!("Memory:");
    // for (pointer, object) in state.memory.objects.iter() {
    //     println!("  {:?}: {:?}", pointer, object);
    // }
    // println!("Interpreting {:?}: {:?}", state.instruction_pointer(), state.instruction_pointer().map(|opcode| program.code().get_opcode(&opcode)));



    let opcode: &OpCode = {
        let address = state.instruction_pointer()
            .expect("Interpreter error: cannot reference opcode at instruction pointer: nothing");

        let opcode = program.get_opcode(&address)
            .expect(&format!("Interpreter error: cannot reference opcode at instruction pointer: \
                              {:?}", address));

        opcode
    };

    //eprintln!("{ :<width$}", "-", width=80);
/*
    eprintln!("| {: <code$} |", "CODE", code=30);
    for (address, opcode) in program.code().all_opcodes() {
        let here = if state.instruction_pointer().unwrap() == address { "*" } else { " " };
        eprintln!("| {}{} {: <width$} |", here, address, opcode.to_string(), width=24);
    }
    eprintln!();

    eprintln!("| {: <constant_pool$} |", "CONSTANT POOL", constant_pool=50);
    for (i, constant) in program.constants().iter().enumerate() {
        let index = ConstantPoolIndex::from_usize(i);
        eprintln!("|  {: >4} {: <width$} |", index.to_string(), constant.to_string(), width=44);
    }
    eprintln!();

    eprintln!("| {: <globals$} |", "GLOBALS", globals=7);
    for (i, index) in program.globals().iter().enumerate() {
        eprintln!("| {: >2} {: >width$} |", i, index.to_string(), width=4);
    }
    eprintln!();

    eprintln!("| {: <stack$} |", "STACK", stack=16);
    for (i, pointer) in state.operands.iter().enumerate() {
        eprintln!("| {: >4} {} |", i, pointer);
    }
    eprintln!();*/

    match opcode {
        OpCode::Literal { index } => {
            let constant: &ProgramObject = program.get_constant(index)
                 .expect(&format!("Literal error: no constant at index {:?}", index.value()));

            match constant {
                ProgramObject::Null => (),
                ProgramObject::Boolean(_) => (),
                ProgramObject::Integer(_) => (),
                _ => panic!("Literal error: constant at index {:?} must be either Null, Integer, \
                             or Boolean, but is {:?}", index, constant),
            }

            state.allocate_and_push_operand(RuntimeObject::from_constant(constant));
            state.bump_instruction_pointer(program);
        }

        OpCode::GetLocal { index } => {
            let frame: &LocalFrame = state.current_frame()
                .expect("Get local error: no frame on stack.");

            let local: Pointer = frame.get_local(&index)
                .expect(&format!("Get local error: there is no local at index {:?} in the current \
                                  frame", index));

            state.push_operand(local);
            state.bump_instruction_pointer(program);
        }

        OpCode::SetLocal { index } => {
            let operand: Pointer = *state.peek_operand()
                .expect("Set local error: cannot pop from empty operand stack");

            let frame: &mut LocalFrame = state.current_frame_mut()
                .expect("Set local error: no frame on stack.");

            frame.update_local(index, operand)
                .expect(&format!("Set local error: there is no local at index {:?} in the current \
                                  frame", index));

            state.bump_instruction_pointer(program);
        }

        OpCode::GetGlobal { name: index } => {
            let constant: &ProgramObject = program.get_constant(index)
                .expect(&format!("Get global error: no constant at index {:?}", index.value()));

            let name: &str = match constant {
                ProgramObject::String(s) => s,
                _ => panic!("Get global error: constant at index {:?} must be a String, \
                             but it is {:?}", index.value(), constant),
            };

            let global = state.get_global(name).map(|g| g.clone())
                .expect(&format!("Get global error: no such global: {}", name));

            state.push_operand(global);
            state.bump_instruction_pointer(program);
        }

        OpCode::SetGlobal { name: index } => {
            let constant: &ProgramObject = program.get_constant(index)
                .expect(&format!("Set global error: no constant at index {:?}", index.value()));

            let name: &str = match constant {
                ProgramObject::String(s) => s,
                _ => panic!("Set global error: constant at index {:?} must be a String, \
                             but it is {:?}", index.value(), constant),
            };

            let operand: Pointer = state.peek_operand().map(|o| o.clone())
                .expect("Set global: cannot pop operand from empty operand stack");

            state.update_global(name.to_string(), operand);

            state.bump_instruction_pointer(program);
        }

        OpCode::Object { class: index } => {
            let constant: &ProgramObject = program.get_constant(index)
                .expect(&format!("Object error: no constant at index {:?}", index.value()));

            let member_definitions: Vec<&ProgramObject> = match constant {
                ProgramObject::Class(v) => v,
                _ => panic!("Object error: constant at index {:?} must be a String, \
                             but it is {:?}", index.value(), constant),
            }.iter().map(| index | program.get_constant(index)
                .expect(&format!("Object error: no constant at index {:?} for member_ object",
                                 index.value()))).collect();

            let (slots, methods): (Vec<&ProgramObject>, Vec<&ProgramObject>) =
                member_definitions.iter().partition(|c| match c {
                    ProgramObject::Method { code:_, locals:_, arguments:_, name:_ } => false,
                    ProgramObject::Slot { name:_ } => true,
                    member =>
                        panic!("Object error: class members may be either Methods or Slots, \
                                 but this member is {:?}", member),
            }); // XXX this will work even if the member definitions are not sorted, which is
                // contrary to the spec

            let fields: HashMap<String, Pointer> = {
                let mut map: HashMap<String, Pointer> = HashMap::new();
                for slot in slots.into_iter().rev() {
                    if let ProgramObject::Slot {name: index} = slot {
                        let object = state.pop_operand()
                            .expect("Object error: cannot pop operand (member) from empty operand \
                                     stack");

                        let constant: &ProgramObject = program.get_constant(index)
                            .expect(&format!("Object error: no constant at index {:?}",
                                             index.value()));

                        let name: &str = match constant {
                            ProgramObject::String(s) => s,
                            _ => panic!("Object error: constant at index {:?} must be a String, \
                                         but it is {:?}", index.value(), constant),
                        };

                        let result = map.insert(name.to_string(), object);
                        if let Some(_) = result {
                            panic!("Object error: member fields must have unique names, but \
                                    {} is used by to name more than one field", name)
                        }
                    } else {
                        unreachable!()
                    }
                }
                map
            };

            let method_map: HashMap<String, ProgramObject> = {
                let mut map: HashMap<String, ProgramObject> = HashMap::new();
                for method in methods {
                    match method {
                        ProgramObject::Method { name: index, arguments:_, locals:_, code:_ } => {
                            let constant: &ProgramObject = program.get_constant(index)
                                .expect(&format!("Object error: no constant at index {:?}",
                                                 index.value()));

                            let name: &str = match constant {
                                ProgramObject::String(s) => s,
                                _ => panic!("Object error: constant at index {:?} must be a String, \
                                             but it is {:?}", index.value(), constant),
                            };
                            let result = map.insert(name.to_string(), method.clone());

                            match result {
                                Some (other_method) =>
                                    panic!("Object error: method {} has a non-unique name in \
                                            object: {:?} v {:?}", name, method, other_method),
                                None => ()
                            }
                        },
                        _ => unreachable!(),
                    }
                }
                map
            };

            let parent = state.pop_operand()
                .expect("Object error: cannot pop operand (parent) from empty operand stack");

            state.allocate_and_push_operand(RuntimeObject::from(parent, fields, method_map));
            state.bump_instruction_pointer(program);
        }

        OpCode::Array => {
            let initializer = state.pop_operand()
                .expect(&format!("Array error: cannot pop initializer from empty operand stack"));

            let size_pointer = state.pop_operand()
                .expect(&format!("Array error: cannot pop size from empty operand stack"));

            let size_object: &RuntimeObject = state.dereference(&size_pointer)
                .expect(&format!("Array error: pointer does not reference an object in memory {:?}",
                                 size_pointer));

            let size: usize = match size_object {
                RuntimeObject::Integer(n) => {
                    if *n < 0 {
                        panic!("Array error: negative value cannot be used to specify the size of \
                                an array {:?}", size_object);
                    } else {
                        *n as usize
                    }
                }
                _ => panic!("Array error: object cannot be used to specify the size of an array \
                             {:?}", size_object),
            };

            let mut elements: Vec<Pointer> = Vec::new();
            for _ in 0..size {
                let pointer = state.copy_memory(&initializer)
                    .expect(&format!("Array error: no initializer to copy from at {:?}",
                                      initializer));
                elements.push(pointer);
            }

            state.allocate_and_push_operand(RuntimeObject::from_pointers(elements));
            state.bump_instruction_pointer(program);
        }

        OpCode::GetField { name: index } => {
            let constant: &ProgramObject = program.get_constant(index)
                .expect(&format!("Get slot error: no constant to serve as label name at index {:?}",
                                 index.value()));

            let name: &str = match constant {
                ProgramObject::String(s) => s,
                _ => panic!("Get slot error: constant at index {:?} must be a String, but it is {:?}",
                            index, constant),
            };

            let operand_pointer: Pointer = state.pop_operand()
                .expect(&format!("Get slot error: cannot pop operand from empty operand stack"));

            let operand = state.dereference(&operand_pointer)
                .expect(&format!("Get slot error: no operand object at {:?}", operand_pointer));

            match operand {
                RuntimeObject::Object { parent:_, fields, methods:_ } => {
                    let slot: Pointer = fields.get(name)
                        .expect(&format!("Get slot error: no field {} in object", name))
                        .clone();

                    state.push_operand(slot)
                }
                _ => panic!("Get slot error: attempt to access field of a non-object {:?}", operand)
            }; // this semicolon turns the expression into a statement and is *important* because of
               // how temporaries work https://github.com/rust-lang/rust/issues/22449

            state.bump_instruction_pointer(program);
        }

        OpCode::SetField { name: index } => {
            let constant: &ProgramObject = program.get_constant(index)
                .expect(&format!("Set slot error: no constant to serve as label name at index {:?}",
                                 index.value()));

            let name: &str = match constant {
                ProgramObject::String(s) => s,
                _ => panic!("Set slot error: constant at index {:?} must be a String, but it is {:?}",
                            index, constant),
            };

            let value: Pointer = state.pop_operand()
                .expect(&format!("Set slot error: cannot pop operand (value) from empty operand \
                                  stack"));

            let host_pointer: Pointer = state.pop_operand().clone()
                .expect(&format!("Set slot error: cannot pop operand (host) from empty operand \
                                  stack"));

            let host = state.dereference_mut(&host_pointer)
                .expect(&format!("Set slot error: no operand object at {:?}", host_pointer));

            match host {
                RuntimeObject::Object { parent:_, fields, methods:_ } => {
                    if !(fields.contains_key(name)) {
                        panic!("Set slot error: no field {} in object {:?}", name, host)
                    }

                    fields.insert(name.to_string(), value.clone());
                    state.push_operand(value)
                }
                _ => panic!("Get slot error: attempt to access field of a non-object {:?}", host)
            }; // this semicolon turns the expression into a statement and is *important* because of
               // how temporaries work https://github.com/rust-lang/rust/issues/22449

            state.bump_instruction_pointer(program);
        }

        OpCode::CallMethod { name: index, arguments: parameters } => {
            if parameters.value() == 0 {
                panic!("Call method error: method must have at least one parameter (receiver)");
            }

            let mut arguments: VecDeque<Pointer> = VecDeque::with_capacity(parameters.value() as usize);
            for index in 0..(parameters.to_usize() - 1) {
                let element = state.pop_operand()
                    .expect(&format!("Call method error: cannot pop argument {} from empty operand \
                                      stack", index));
                arguments.push_front(element);
            }

            let object_pointer: Pointer = state.pop_operand()
                .expect(&format!("Call method error: cannot pop host object from empty operand \
                                  stack"));

            let constant: &ProgramObject = program.get_constant(index)
                .expect(&format!("Call method error: no constant to serve as format index {:?}",
                                 index));

            let name: &str = match constant {
                ProgramObject::String(s) => s,
                _ => panic!("Call method error: constant at index {:?} must be a String, but it is \
                             {:?}", index, constant),
            };

            let object: &mut RuntimeObject = state.dereference_mut(&object_pointer)
                .expect(&format!("Call method error: no operand object at {:?}", object_pointer));


            match object {
                RuntimeObject::Null =>
                    interpret_null_method(object_pointer, name, &Vec::from(arguments), state, program),
                RuntimeObject::Integer(_) =>
                    interpret_integer_method(object_pointer, name, &Vec::from(arguments), state, program),
                RuntimeObject::Boolean(_) =>
                    interpret_boolean_method(object_pointer, name, &Vec::from(arguments), state, program),
                RuntimeObject::Array(_) =>
                    interpret_array_method(object_pointer, name, &Vec::from(arguments), *parameters, state, program),
                RuntimeObject::Object { parent:_, fields:_, methods:_ } =>
                    dispatch_object_method(object_pointer, name, &Vec::from(arguments), *parameters, state, program),
            };
        }

        OpCode::CallFunction { name: index, arguments } => {

            let constant: &ProgramObject = program.get_constant(index)
                .expect(&format!("Call function error: no constant to serve as function name at \
                                  index {:?}", index));

            let name = match constant {
                ProgramObject::String(string) => string,
                _ => panic!("Call function error: function name must be specified by a String \
                             object, but instead it is: {:?}", constant),
            };

            let function: ProgramObject = {
                state.get_function(name)
                    .expect(&format!("Call function error: no such function {}", name))
                    .clone()
            };

            match function {
                ProgramObject::Method { name:_, arguments: parameters, locals, code: range } => {
                    if arguments.value() != parameters.value() {
                        panic!("Call function error: function definition requires {} arguments, \
                               but {} were supplied", parameters.value(), arguments.value())
                    }

                    let mut slots: VecDeque<Pointer> =
                        VecDeque::with_capacity(parameters.value() as usize + locals.value() as usize);

                    for index in 0..arguments.to_usize() {
                        let element = state.pop_operand()
                            .expect(&format!("Call function error: cannot pop argument {} from \
                                              empty operand stack", index));
                        slots.push_front(element);
                    }

                    for _ in 0..locals.value() {
                        slots.push_back(state.allocate(RuntimeObject::Null))
                    }

                    state.bump_instruction_pointer(program);
                    state.new_frame(*state.instruction_pointer(), Vec::from(slots));
                    state.set_instruction_pointer(Some(*range.start()));
                },
                _ => panic!("Call function error: constant at index {:?} must be a Method, but it \
                             is {:?}", index, constant),
            }
        }

        OpCode::Print { format: index, arguments } => {
            let mut argument_values = {
                let mut argument_values: Vec<Pointer> = Vec::new();
                for index in 0..arguments.value() {
                    let element = state.pop_operand()
                        .expect(&format!("Print error: cannot pop argument {} from empty operand \
                                          stack", index));
                    argument_values.push(element);
                }
                argument_values
            };

            let constant: &ProgramObject = program.get_constant(index)
                .expect(&format!("Print error: no constant to serve as format index {:?}", index));

            let format: &str = match constant {
                ProgramObject::String(s) => s,
                _ => panic!("Print error: constant at index {:?} must be a String, but it is {:?}",
                            index, constant),
            };

            let mut escape = false;
            for character in format.chars() {
                match (character, escape) {
                    ('~', _) => {
                        let string = &argument_values.pop()
                            .map(|e| state.dereference_to_string(&e))
                            .expect(&format!("Print error: Not enough arguments for format {}",
                                             format));

                        output.write_str(string)
                            .expect("Print error: Could not write to output stream.")
                    },
                    ('\\', false) => {
                        escape = true;
                    }
                    ('\\', true)  => {
                        output.write_char('\\')
                            .expect("Print error: Could not write to output stream.");
                        escape = false;
                    }
                    ('n', true)  => {
                        output.write_char('\n')
                            .expect("Print error: Could not write to output stream.");
                        escape = false;
                    }
                    ('"', true)  => {
                        output.write_char('"')
                            .expect("Print error: Could not write to output stream.");
                        escape = false;
                    }
                    ('t', true)  => {
                        output.write_char('\t')
                            .expect("Print error: Could not write to output stream.");
                        escape = false;
                    }
                    (character, true)  => {
                        panic!("Print error: Unknown escape sequence: \\{}", character)
                    }
                    (character, false) => {
                        output.write_char(character)
                            .expect("Print error: Could not write to output stream.")
                    }
                }
            }

            if !argument_values.is_empty() {
                panic!("Print error: Unused arguments for format {}", format)
            }

            state.allocate_and_push_operand(RuntimeObject::Null);
            state.bump_instruction_pointer(program);
        }

        OpCode::Label { name: _ } => {
            state.bump_instruction_pointer(program);
        }

        OpCode::Jump { label } => {
            let constant: &ProgramObject = program.get_constant(label)
                .expect(&format!("Jump error: no label name at index {:?}", label.value()));

            let name: &str = match constant {
                ProgramObject::String(s) => s,
                _ => panic!("Jump error: constant at index {:?} must be a String, but it is {:?}",
                            label, constant),
            };

            state.set_instruction_pointer_from_label(program, name)
                .expect(&format!("Jump error: no such label {:?} (labels: {:?})", name, program.labels()));
        }

        OpCode::Branch { label } => {
            let operand = state.pop_operand()
                .expect("Branch error: cannot pop operand from empty operand stack");

            let jump_condition_object = state.dereference(&operand)
                .expect(&format!("Branch error: cannot find condition at {:?}", operand));

            let jump_condition = {
                match jump_condition_object {
                    RuntimeObject::Boolean(value) => *value,
                    RuntimeObject::Null => false,
                    _ => true,
                }
            };

            if !jump_condition {
                state.bump_instruction_pointer(program);
                return;
            }

            let constant: &ProgramObject = program.get_constant(label)
                .expect(&format!("Branch error: no label name at index {:?}",
                                 label.value()));

            let name: &str = match constant {
                ProgramObject::String(s) => s,
                _ => panic!("Branch error: constant at index {:?} must be a String, but it is {:?}",
                            label, constant),
            };

            state.set_instruction_pointer_from_label(program, name)
                .expect(&format!("Branch error: no such label {:?}", name));
        }

        OpCode::Return => {
            let current_frame: LocalFrame = state.pop_frame()
                .expect("Return error: cannot pop local frame from empty frame stack");
            let address: &Option<Address> = current_frame.return_address();
            state.set_instruction_pointer(*address);
            // current_frame is reclaimed here
        }

        OpCode::Drop => { // FIXME balance stack
            state.pop_operand()
                .expect("Drop error: cannot pop operand from empty operand stack");
            state.bump_instruction_pointer(program);
        },
    }
}

macro_rules! check_arguments_one {
    ($pointer: expr, $arguments: expr, $name: expr, $state: expr) => {{
        if $arguments.len() != 1 {
            panic!("Call method error: method {} takes 1 argument, but {} were supplied",
                    $name, $arguments.len())
        }

        let argument_pointer: &Pointer = &$arguments[0];
        let argument = $state.dereference(argument_pointer)
            .expect(&format!("Call method error: no operand object at {:?}", argument_pointer));

        let object = $state.dereference(&$pointer).unwrap(); /*checked earlier*/
        (object, argument)
    }}
}

macro_rules! push_result_and_finish {
    ($result: expr, $state: expr, $program: expr) => {{
        $state.allocate_and_push_operand($result);
        $state.bump_instruction_pointer($program);
    }}
}

macro_rules! push_pointer_and_finish {
    ($result: expr, $state: expr, $program: expr) => {{
        $state.push_operand($result);
        $state.bump_instruction_pointer($program);
    }}
}

pub fn interpret_null_method(pointer: Pointer, name: &str, arguments: &Vec<Pointer>,
                             state: &mut State, program: &Program) {

    let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
    let result = match (name, operand) {
        ("==", RuntimeObject::Null)  => RuntimeObject::from_bool(true),
        ("==", _)             => RuntimeObject::from_bool(false),
        ("!=", RuntimeObject::Null)  => RuntimeObject::from_bool(false),
        ("!=", _)             => RuntimeObject::from_bool(true),
        ("eq", RuntimeObject::Null)  => RuntimeObject::from_bool(true),
        ("eq", _)             => RuntimeObject::from_bool(false),
        ("neq", RuntimeObject::Null) => RuntimeObject::from_bool(false),
        ("neq", _)            => RuntimeObject::from_bool(true),

        _ => panic!("Call method error: object {:?} has no method {} for operand {:?}",
                     object, name, operand),
    };
    push_result_and_finish!(result, state, program);
}

pub fn interpret_integer_method(pointer: Pointer, name: &str, arguments: &Vec<Pointer>,
                                state: &mut State, program: &Program) {

    let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
    let result = match (object, name, operand) {
        (RuntimeObject::Integer(i), "+",   RuntimeObject::Integer(j)) => RuntimeObject::from_i32 (*i +  *j),
        (RuntimeObject::Integer(i), "-",   RuntimeObject::Integer(j)) => RuntimeObject::from_i32 (*i -  *j),
        (RuntimeObject::Integer(i), "*",   RuntimeObject::Integer(j)) => RuntimeObject::from_i32 (*i *  *j),
        (RuntimeObject::Integer(i), "/",   RuntimeObject::Integer(j)) => RuntimeObject::from_i32 (*i /  *j),
        (RuntimeObject::Integer(i), "%",   RuntimeObject::Integer(j)) => RuntimeObject::from_i32 (*i %  *j),
        (RuntimeObject::Integer(i), "<=",  RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i <= *j),
        (RuntimeObject::Integer(i), ">=",  RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i >= *j),
        (RuntimeObject::Integer(i), "<",   RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i <  *j),
        (RuntimeObject::Integer(i), ">",   RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i >  *j),
        (RuntimeObject::Integer(i), "==",  RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i == *j),
        (RuntimeObject::Integer(i), "!=",  RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i != *j),
        (RuntimeObject::Integer(_), "==",  _)                  => RuntimeObject::from_bool(false),
        (RuntimeObject::Integer(_), "!=",  _)                  => RuntimeObject::from_bool(true),

        (RuntimeObject::Integer(i), "add", RuntimeObject::Integer(j)) => RuntimeObject::from_i32 (*i +  *j),
        (RuntimeObject::Integer(i), "sub", RuntimeObject::Integer(j)) => RuntimeObject::from_i32 (*i -  *j),
        (RuntimeObject::Integer(i), "mul", RuntimeObject::Integer(j)) => RuntimeObject::from_i32 (*i *  *j),
        (RuntimeObject::Integer(i), "div", RuntimeObject::Integer(j)) => RuntimeObject::from_i32 (*i /  *j),
        (RuntimeObject::Integer(i), "mod", RuntimeObject::Integer(j)) => RuntimeObject::from_i32 (*i %  *j),
        (RuntimeObject::Integer(i), "le",  RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i <= *j),
        (RuntimeObject::Integer(i), "ge",  RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i >= *j),
        (RuntimeObject::Integer(i), "lt",  RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i <  *j),
        (RuntimeObject::Integer(i), "gt",  RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i >  *j),
        (RuntimeObject::Integer(i), "eq",  RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i == *j),
        (RuntimeObject::Integer(i), "neq", RuntimeObject::Integer(j)) => RuntimeObject::from_bool(*i != *j),
        (RuntimeObject::Integer(_), "eq",  _)                  => RuntimeObject::from_bool(false),
        (RuntimeObject::Integer(_), "neq", _)                  => RuntimeObject::from_bool(true),

        _ => panic!("Call method error: object {:?} has no method {} for operand {:?}",
                     object, name, operand),
    };
    push_result_and_finish!(result, state, program);
}

pub fn interpret_boolean_method(pointer: Pointer, name: &str, arguments: &Vec<Pointer>,
                                state: &mut State, program: &Program) {

    let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
    let result = match (object, name, operand) {
        (RuntimeObject::Boolean(p), "and", RuntimeObject::Boolean(q)) => RuntimeObject::from_bool(*p && *q),
        (RuntimeObject::Boolean(p), "or",  RuntimeObject::Boolean(q)) => RuntimeObject::from_bool(*p || *q),
        (RuntimeObject::Boolean(p), "eq",  RuntimeObject::Boolean(q)) => RuntimeObject::from_bool(*p == *q),
        (RuntimeObject::Boolean(p), "neq", RuntimeObject::Boolean(q)) => RuntimeObject::from_bool(*p != *q),
        (RuntimeObject::Boolean(_), "eq",  _)                  => RuntimeObject::from_bool(false),
        (RuntimeObject::Boolean(_), "neq", _)                  => RuntimeObject::from_bool(true),

        (RuntimeObject::Boolean(p), "&",   RuntimeObject::Boolean(q)) => RuntimeObject::from_bool(*p && *q),
        (RuntimeObject::Boolean(p), "|",   RuntimeObject::Boolean(q)) => RuntimeObject::from_bool(*p || *q),
        (RuntimeObject::Boolean(p), "==",  RuntimeObject::Boolean(q)) => RuntimeObject::from_bool(*p == *q),
        (RuntimeObject::Boolean(p), "!=",  RuntimeObject::Boolean(q)) => RuntimeObject::from_bool(*p != *q),
        (RuntimeObject::Boolean(_), "==",  _)                  => RuntimeObject::from_bool(false),
        (RuntimeObject::Boolean(_), "!=",  _)                  => RuntimeObject::from_bool(true),

        _ => panic!("Call method error: object {:?} has no method {} for operand {:?}",
                    object, name, operand),
    };
    push_result_and_finish!(result, state, program);
}


pub fn interpret_array_method(pointer: Pointer, name: &str, arguments: &Vec<Pointer>,
                              arity: Arity, state: &mut State, program: &Program) {

    if arguments.len() != arity.to_usize() - 1 {
        panic!("Call method error: Array method {} takes {} argument, but {} were supplied",
                name, arity.value() - 1, arguments.len())
    }

    if name == "==" || name == "eq" {
        let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
        let result = RuntimeObject::from_bool(compare(state, object, operand));
        push_result_and_finish!(result, state, program);
    }

    if name == "!=" || name == "neq" {
        let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
        let result = RuntimeObject::from_bool(!compare(state, object, operand));
        push_result_and_finish!(result, state, program);
    }

    if name == "get" {
        let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
        let result = match (object, operand) {
            (RuntimeObject::Array(element_pointers), RuntimeObject::Integer(index)) => {
                if (*index as usize) >= element_pointers.len() {
                    panic!("Call method error: array index {} is out of bounds (should be < {})",
                            index, element_pointers.len())
                }
                element_pointers.get(*index as usize)
                    .expect("Call method error: no array element object at {:?}")
            },
            _ => panic!("Call method error: array {:?} has no method {} for operand {:?}",
                         object, name, operand),
        }.clone();

        push_pointer_and_finish!(result, state, program);
    }

    if name == "set" {
        let operand_1_pointer: &Pointer = &arguments[0];
        let operand_2_pointer: &Pointer = &arguments[1];

        let index: usize = match state.dereference(operand_1_pointer) {
            Some(RuntimeObject::Integer(index)) => *index as usize,
            Some(object) => panic!("Call method error: cannot index array with {:?}", object),
            None => panic!("Call method error: no operand (1) object at {:?}", operand_1_pointer),
        };

        let object : &mut RuntimeObject = state.dereference_mut(&pointer).unwrap(); /* pre-checked elsewhere */
        let result = match object {
            RuntimeObject::Array(element_pointers) => {
                if index >= element_pointers.len() {
                    panic!("Call method error: array index {} is out of bounds (should be < {})",
                           index, element_pointers.len())
                }
                element_pointers[index] = *operand_2_pointer;
                RuntimeObject::Null
            },
            _ => panic!("Call method error: object {:?} has no method {}", object, name),
        };

        push_result_and_finish!(result, state, program)
    }
}


fn dispatch_object_method(pointer: Pointer, name: &str, arguments: &Vec<Pointer>, arity: Arity,
                          state: &mut State, program: &Program) {

    let mut cursor: Pointer = pointer;
    loop {
        let object = state.dereference(&cursor)
            .expect("Call method error: no object at {:?}");

        let method: ProgramObject = match object {
            RuntimeObject::Object { parent, fields: _, methods } => {
                if let Some(method) = methods.get(name) {
                    method.clone()
                } else {
                    if name == "==" || name == "eq" {
                        let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
                        let result = RuntimeObject::from_bool(compare(state, object, operand));
                        push_result_and_finish!(result, state, program);
                        break;
                    }

                    if name == "!=" || name == "neq" {
                        let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
                        let result = RuntimeObject::from_bool(!compare(state, object, operand));
                        push_result_and_finish!(result, state, program);
                        break;
                    }

                    cursor = *parent;
                    continue
                }
            },
            RuntimeObject::Null => {
                interpret_null_method(cursor, name, arguments, state, program);
                break
            },
            RuntimeObject::Boolean(_) => {
                interpret_boolean_method(cursor, name, arguments, state, program);
                break
            },
            RuntimeObject::Integer(_) => {
                interpret_integer_method(cursor, name, arguments, state, program);
                break
            },
            RuntimeObject::Array(_) => {
                interpret_array_method(cursor, name, arguments, arity, state, program);
                break
            },
        };

        interpret_object_method(method, cursor, name, arguments, state, program);
        break
    }
}


fn interpret_object_method(method: ProgramObject, pointer: Pointer, name: &str,
                           arguments: &Vec<Pointer>, state: &mut State, program: &Program) {

    match method {
        ProgramObject::Method { name: _, locals, arguments: arity, code } => {
            if arguments.len() != arity.to_usize() - 1 {
                panic!("Call method error: method {} takes {} arguments, but {} were supplied",
                        name, arity.value() - 1, arguments.len())
            }

            let mut slots: Vec<Pointer> =
                Vec::with_capacity(1 + arity.to_usize() + locals.to_usize());

            slots.push(pointer);

            slots.extend(arguments); // TODO passes by reference... correct?

            for _ in 0..locals.to_usize() {
                slots.push(state.allocate(RuntimeObject::Null))
            }

            state.bump_instruction_pointer(program);
            state.new_frame(*state.instruction_pointer(), slots);
            state.set_instruction_pointer(Some(*code.start()));
        },

        thing => panic!("Call method error: member {} in object definition should have type \
                         Method, but it is {:?}", name, thing),
    }
}

pub fn compare(state: &State, a: &RuntimeObject, b: &RuntimeObject) -> bool {
    match (a, b) {
        (RuntimeObject::Null, RuntimeObject::Null) => true,
        (RuntimeObject::Integer(a), RuntimeObject::Integer(b)) => a == b,
        (RuntimeObject::Boolean(a), RuntimeObject::Boolean(b)) => a == b,
        (RuntimeObject::Array(a), RuntimeObject::Array(b)) if a.len() == b.len() =>
            a.iter().zip(b.iter()).all(|(a, b)| {
                let a = state.memory.dereference(a).unwrap();
                let b = state.memory.dereference(b).unwrap();
                compare(state, a, b)
            }),
        (RuntimeObject::Object {
            parent: a_parent,
            fields: a_fields,
            methods: a_methods
        },
        RuntimeObject::Object {
            parent: b_parent,
            fields: b_fields,
            methods: b_methods
        }) => {
            let a_parent = state.memory.dereference(a_parent).unwrap();
            let b_parent = state.memory.dereference(b_parent).unwrap();
            if !compare(state, a_parent, b_parent) {
                false
            } else {
                let same_fields =
                    a_fields.iter().zip(b_fields.iter())
                        .all(|((a_name, a), (b_name, b))| {
                            let a = state.memory.dereference(a).unwrap();
                            let b = state.memory.dereference(b).unwrap();
                            if a_name == b_name {
                                compare(state, a, b)
                            } else {
                                false
                            }
                        });
                if same_fields {
                    a_methods.iter().zip(b_methods.iter())
                        .all(|((a_name, a), (b_name, b))| {
                            a_name == b_name && a == b
                        })
                } else {
                    false
                }
            }
        },
        _ => false
    }
}