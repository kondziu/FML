use std::collections::{HashMap, VecDeque};
use std::fmt::{Write, Error};
use std::io::Write as IOWrite;

use super::types::*;
use super::bytecode::*;
use super::program::Program;

use anyhow;
use crate::bytecode::heap::{Pointer, HeapObject, ObjectInstance, Heap, HeapIndex};
use crate::bytecode::program::ProgramObject;
use crate::bytecode::state::{State, LocalFrame};

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
        Some(ProgramObject::Method { name:_, locals, parameters:_, code }) => (*code.start(), locals),
        None => panic!("No entry method at index {:?}", program.entry()),
        Some(constant) => panic!("Constant at index {:?} is not a method {:?}",
                                  program.entry(), constant),
    };

    let mut slots = Vec::new();
    for _ in 0..locals.to_usize() {
        slots.push(Pointer::Null);
    }

    state.new_frame(None, slots);
    state.set_instruction_pointer(Some(start_address));
    while state.has_next_instruction_pointer() {
        interpret(&mut state, &mut output, program);
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

            state.push_operand(Pointer::from(constant));
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

            state.update_global(name.to_owned(), operand);
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
                    ProgramObject::Method { code:_, locals:_, parameters:_, name:_ } => false,
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
                        ProgramObject::Method { name: index, parameters:_, locals:_, code:_ } => {
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

            state.allocate_and_push_operand(HeapObject::from(parent, fields, method_map));
            state.bump_instruction_pointer(program);
        }

        OpCode::Array => {
            let initializer = state.pop_operand()
                .expect(&format!("Array error: cannot pop initializer from empty operand stack"));

            let size_pointer = state.pop_operand()
                .expect(&format!("Array error: cannot pop size from empty operand stack"));

            let size: usize = match size_pointer {
                Pointer::Integer(n) => {
                    if n < 0 {
                        panic!("Array error: negative value cannot be used to specify the size of \
                                an array {:?}", size_pointer);
                    } else {
                        n as usize
                    }
                }
                Pointer::Null => panic!("Array error: null cannot be used to specify the size of an array"),
                Pointer::Boolean(_) => panic!("Array error: boolean cannot be used to specify the size of an array"),
                _ => panic!("Array error: object at pointer {} cannot be used to specify the size of an array", size_pointer),
            };

            let mut elements: Vec<Pointer> = Vec::new();
            for _ in 0..size {
                let pointer = state.copy_memory(&initializer)
                    .expect(&format!("Array error: no initializer to copy from at {:?}",
                                      initializer));
                elements.push(pointer);
            }

            state.allocate_and_push_operand(HeapObject::from_pointers(elements));
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

            let heap_reference =
                operand_pointer.into_heap_reference().expect("Cast error");

            let operand = state.dereference(&heap_reference)
                .expect(&format!("Get slot error: no operand object at {:?}", operand_pointer));

            match operand {
                HeapObject::Object(ObjectInstance{parent:_, fields, methods:_ }) => {
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

            let heap_reference = host_pointer.into_heap_reference().expect("Cast error");

            let host = state.dereference_mut(&heap_reference)
                .expect(&format!("Set slot error: no operand object at {:?}", host_pointer));

            match host {
                HeapObject::Object(ObjectInstance { parent:_, fields, methods:_ }) => {
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

            let heap_reference = object_pointer.into_heap_reference().expect("Cast error");

            let object: &mut HeapObject = state.dereference_mut(&heap_reference)
                .expect(&format!("Call method error: no operand object at {:?}", object_pointer));


            unimplemented!();
            match object {
                // HeapObject::Null =>
                //     interpret_null_method(object_pointer, name, &Vec::from(arguments), state, program),
                // HeapObject::Integer(_) =>
                //     interpret_integer_method(object_pointer, name, &Vec::from(arguments), state, program),
                // HeapObject::Boolean(_) =>
                //     interpret_boolean_method(object_pointer, name, &Vec::from(arguments), state, program),
                HeapObject::Array(_) =>
                    interpret_array_method(object_pointer, name, &Vec::from(arguments), *parameters, state, program),
                HeapObject::Object(ObjectInstance { parent:_, fields:_, methods:_ }) =>
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
                ProgramObject::Method { name:_, parameters, locals, code: range } => {
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
                        slots.push_back(Pointer::Null)
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

            state.push_operand(Pointer::Null);
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

            if !operand.evaluate_as_condition() {
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
        // if $arguments.len() != 1 {
        //     panic!("Call method error: method {} takes 1 argument, but {} were supplied",
        //             $name, $arguments.len())
        // }
        //
        // let argument_pointer: &Pointer = &$arguments[0];
        // let argument = $state.dereference(argument_pointer)
        //     .expect(&format!("Call method error: no operand object at {:?}", argument_pointer));
        //
        // let object = $state.dereference(&$pointer).unwrap(); /*checked earlier*/
        // (object, argument)
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

    //let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
    unimplemented!();
    // let result = match (name, operand) {
    //     // ("==", HeapObject::Null)  => HeapObject::from_bool(true),
    //     // ("==", _)             => HeapObject::from_bool(false),
    //     // ("!=", HeapObject::Null)  => HeapObject::from_bool(false),
    //     // ("!=", _)             => HeapObject::from_bool(true),
    //     // ("eq", HeapObject::Null)  => HeapObject::from_bool(true),
    //     // ("eq", _)             => HeapObject::from_bool(false),
    //     // ("neq", HeapObject::Null) => HeapObject::from_bool(false),
    //     // ("neq", _)            => HeapObject::from_bool(true),
    //
    //     _ => panic!("Call method error: object {:?} has no method {} for operand {:?}",
    //                  object, name, operand),
    // };
    // push_result_and_finish!(result, state, program);
}

pub fn interpret_integer_method(pointer: Pointer, name: &str, arguments: &Vec<Pointer>,
                                state: &mut State, program: &Program) {

    //let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
    unimplemented!();
    // let result = match (object, name, operand) {
    //     // (HeapObject::Integer(i), "+",   HeapObject::Integer(j)) => HeapObject::from_i32 (*i +  *j),
    //     // (HeapObject::Integer(i), "-",   HeapObject::Integer(j)) => HeapObject::from_i32 (*i -  *j),
    //     // (HeapObject::Integer(i), "*",   HeapObject::Integer(j)) => HeapObject::from_i32 (*i *  *j),
    //     // (HeapObject::Integer(i), "/",   HeapObject::Integer(j)) => HeapObject::from_i32 (*i /  *j),
    //     // (HeapObject::Integer(i), "%",   HeapObject::Integer(j)) => HeapObject::from_i32 (*i %  *j),
    //     // (HeapObject::Integer(i), "<=",  HeapObject::Integer(j)) => HeapObject::from_bool(*i <= *j),
    //     // (HeapObject::Integer(i), ">=",  HeapObject::Integer(j)) => HeapObject::from_bool(*i >= *j),
    //     // (HeapObject::Integer(i), "<",   HeapObject::Integer(j)) => HeapObject::from_bool(*i <  *j),
    //     // (HeapObject::Integer(i), ">",   HeapObject::Integer(j)) => HeapObject::from_bool(*i >  *j),
    //     // (HeapObject::Integer(i), "==",  HeapObject::Integer(j)) => HeapObject::from_bool(*i == *j),
    //     // (HeapObject::Integer(i), "!=",  HeapObject::Integer(j)) => HeapObject::from_bool(*i != *j),
    //     // (HeapObject::Integer(_), "==",  _)                  => HeapObject::from_bool(false),
    //     // (HeapObject::Integer(_), "!=",  _)                  => HeapObject::from_bool(true),
    //     //
    //     // (HeapObject::Integer(i), "add", HeapObject::Integer(j)) => HeapObject::from_i32 (*i +  *j),
    //     // (HeapObject::Integer(i), "sub", HeapObject::Integer(j)) => HeapObject::from_i32 (*i -  *j),
    //     // (HeapObject::Integer(i), "mul", HeapObject::Integer(j)) => HeapObject::from_i32 (*i *  *j),
    //     // (HeapObject::Integer(i), "div", HeapObject::Integer(j)) => HeapObject::from_i32 (*i /  *j),
    //     // (HeapObject::Integer(i), "mod", HeapObject::Integer(j)) => HeapObject::from_i32 (*i %  *j),
    //     // (HeapObject::Integer(i), "le",  HeapObject::Integer(j)) => HeapObject::from_bool(*i <= *j),
    //     // (HeapObject::Integer(i), "ge",  HeapObject::Integer(j)) => HeapObject::from_bool(*i >= *j),
    //     // (HeapObject::Integer(i), "lt",  HeapObject::Integer(j)) => HeapObject::from_bool(*i <  *j),
    //     // (HeapObject::Integer(i), "gt",  HeapObject::Integer(j)) => HeapObject::from_bool(*i >  *j),
    //     // (HeapObject::Integer(i), "eq",  HeapObject::Integer(j)) => HeapObject::from_bool(*i == *j),
    //     // (HeapObject::Integer(i), "neq", HeapObject::Integer(j)) => HeapObject::from_bool(*i != *j),
    //     // (HeapObject::Integer(_), "eq",  _)                  => HeapObject::from_bool(false),
    //     // (HeapObject::Integer(_), "neq", _)                  => HeapObject::from_bool(true),
    //
    //     _ => panic!("Call method error: object {:?} has no method {} for operand {:?}",
    //                  object, name, operand),
    // };
    // push_result_and_finish!(result, state, program);
}

pub fn interpret_boolean_method(pointer: Pointer, name: &str, arguments: &Vec<Pointer>,
                                state: &mut State, program: &Program) {

    //let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
    unimplemented!();
    //let result = match (object, name, operand) {
        // (HeapObject::Boolean(p), "and", HeapObject::Boolean(q)) => HeapObject::from_bool(*p && *q),
        // (HeapObject::Boolean(p), "or",  HeapObject::Boolean(q)) => HeapObject::from_bool(*p || *q),
        // (HeapObject::Boolean(p), "eq",  HeapObject::Boolean(q)) => HeapObject::from_bool(*p == *q),
        // (HeapObject::Boolean(p), "neq", HeapObject::Boolean(q)) => HeapObject::from_bool(*p != *q),
        // (HeapObject::Boolean(_), "eq",  _)                  => HeapObject::from_bool(false),
        // (HeapObject::Boolean(_), "neq", _)                  => HeapObject::from_bool(true),
        //
        // (HeapObject::Boolean(p), "&",   HeapObject::Boolean(q)) => HeapObject::from_bool(*p && *q),
        // (HeapObject::Boolean(p), "|",   HeapObject::Boolean(q)) => HeapObject::from_bool(*p || *q),
        // (HeapObject::Boolean(p), "==",  HeapObject::Boolean(q)) => HeapObject::from_bool(*p == *q),
        // (HeapObject::Boolean(p), "!=",  HeapObject::Boolean(q)) => HeapObject::from_bool(*p != *q),
        // (HeapObject::Boolean(_), "==",  _)                  => HeapObject::from_bool(false),
        // (HeapObject::Boolean(_), "!=",  _)                  => HeapObject::from_bool(true),
    //
    //     _ => panic!("Call method error: object {:?} has no method {} for operand {:?}",
    //                 object, name, operand),
    // };
    // push_result_and_finish!(result, state, program);
}


pub fn interpret_array_method(pointer: Pointer, name: &str, arguments: &Vec<Pointer>,
                              arity: Arity, state: &mut State, program: &Program) {

    if arguments.len() != arity.to_usize() - 1 {
        panic!("Call method error: Array method {} takes {} argument, but {} were supplied",
                name, arity.value() - 1, arguments.len())
    }

    if name == "get" {
        //let (object, operand) = check_arguments_one!(pointer, arguments, name, state);
        unimplemented!();
        //let result = match (object, operand) {
            // (HeapObject::Array(element_pointers), HeapObject::Integer(index)) => {
            //     if (*index as usize) >= element_pointers.len() {
            //         panic!("Call method error: array index {} is out of bounds (should be < {})",
            //                 index, element_pointers.len())
            //     }
            //     element_pointers.get(*index as usize)
            //         .expect("Call method error: no array element object at {:?}")
            // },
            //_ => panic!("Call method error: array {:?} has no method {} for operand {:?}",
            //             object, name, operand),
        //}.clone();
        //let result = Pointer::Null;

        //push_pointer_and_finish!(result, state, program);
    }

    if name == "set" {
        // let operand_1_pointer: &Pointer = &arguments[0];
        // let operand_2_pointer: &Pointer = &arguments[1];
        //
        // unimplemented!();
        // let index: usize = match state.dereference(operand_1_pointer) {
        //     //Some(HeapObject::Integer(index)) => *index as usize,
        //     Some(object) => panic!("Call method error: cannot index array with {:?}", object),
        //     None => panic!("Call method error: no operand (1) object at {:?}", operand_1_pointer),
        // };
        //
        // let object : &mut HeapObject = state.dereference_mut(&pointer).unwrap(); /* pre-checked elsewhere */
        // unimplemented!();
        // let result = Pointer::Null;
        // let result = match object {
        //     HeapObject::Array(element_pointers) => {
        //         if index >= element_pointers.len() {
        //             panic!("Call method error: array index {} is out of bounds (should be < {})",
        //                    index, element_pointers.len())
        //         }
        //         element_pointers[index] = *operand_2_pointer;
        //         HeapObject::Null
        //     },
        //     _ => panic!("Call method error: object {:?} has no method {}", object, name),
        // };

        //push_result_and_finish!(result, state, program)
    }
}


fn dispatch_object_method(pointer: Pointer, name: &str, arguments: &Vec<Pointer>, arity: Arity,
                          state: &mut State, program: &Program) {
    //
    // let mut cursor: Pointer = pointer;
    // loop {
    //     let object = state.dereference(&cursor)
    //         .expect("Call method error: no object at {:?}");
    //
    //     unimplemented!();
    //     let method: ProgramObject = match object {
    //         HeapObject::Object { parent, fields: _, methods } => {
    //             if let Some(method) = methods.get(name) {
    //                 method.clone()
    //             } else {
    //                 cursor = *parent;
    //                 continue
    //             }
    //         },
    //         // HeapObject::Null => {
    //         //     interpret_null_method(cursor, name, arguments, state, program);
    //         //     break
    //         // },
    //         // HeapObject::Boolean(_) => {
    //         //     interpret_boolean_method(cursor, name, arguments, state, program);
    //         //     break
    //         // },
    //         // HeapObject::Integer(_) => {
    //         //     interpret_integer_method(cursor, name, arguments, state, program);
    //         //     break
    //         // },
    //         HeapObject::Array(_) => {
    //             interpret_array_method(cursor, name, arguments, arity, state, program);
    //             break
    //         },
    //     };
    //
    //     interpret_object_method(method, cursor, name, arguments, state, program);
    //     break
    // }
}


fn interpret_object_method(method: ProgramObject, pointer: Pointer, name: &str,
                           arguments: &Vec<Pointer>, state: &mut State, program: &Program) {

    match method {
        ProgramObject::Method { name: _, locals, parameters: arity, code } => {
            if arguments.len() != arity.to_usize() - 1 {
                panic!("Call method error: method {} takes {} arguments, but {} were supplied",
                        name, arity.value() - 1, arguments.len())
            }

            let mut slots: Vec<Pointer> =
                Vec::with_capacity(1 + arity.to_usize() + locals.to_usize());

            slots.push(pointer);

            slots.extend(arguments); // TODO passes by reference... correct?

            for _ in 0..locals.to_usize() {
                slots.push(Pointer::Null)
            }

            state.bump_instruction_pointer(program);
            state.new_frame(*state.instruction_pointer(), slots);
            state.set_instruction_pointer(Some(*code.start()));
        },

        thing => panic!("Call method error: member {} in object definition should have type \
                         Method, but it is {:?}", name, thing),
    }
}