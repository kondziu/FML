use std::collections::HashMap;

use crate::bytecode::bytecode::*;
use crate::bytecode::types::*;
use crate::bytecode::program::*;
use crate::bytecode::objects::*;
use crate::bytecode::interpreter::*;

macro_rules! hashmap {
        ($key: expr, $value: expr) => {{
            let mut map = HashMap::new();
            map.insert($key, $value);
            map
        }};
        ($key1: expr, $value1: expr, $key2: expr, $value2: expr) => {{
            let mut map = HashMap::new();
            map.insert($key1, $value1);
            map.insert($key2, $value2);
            map
        }};
    }

#[test] fn literal() {
    let code = Code::from(vec!(
        OpCode::Literal { index: ConstantPoolIndex::new(0) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::Integer(42));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(0)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn label() {
    let code = Code::from(vec!(
        OpCode::Label { name: ConstantPoolIndex::new(0) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("o.o".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, Vec::new(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn get_local() {
    let code = Code::from(vec!(
        OpCode::GetLocal { index: LocalFrameIndex::new(0) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!();
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    let pointer = Pointer::from(42i32);
    state.current_frame_mut().unwrap().push_local(pointer);

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(0)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::from(None, vec!(Pointer::from(0)))), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory")
}

#[test] fn set_local() {
    let code = Code::from(vec!(
        OpCode::SetLocal { index: LocalFrameIndex::new(0) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!();
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.push_operand(Pointer::from(42i32));
    let pointer = Pointer::from(0i32);
    state.current_frame_mut().unwrap().push_local(pointer);

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(0)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::from(None, vec!(Pointer::from(0)))), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn get_global() {
    let code = Code::from(vec!(
        OpCode::GetGlobal { name: ConstantPoolIndex::new(0) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("skippy".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!(ConstantPoolIndex::new(0));
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    let pointer = Pointer::from(666i32);
    state.register_global("skippy".to_string(), pointer).unwrap();

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(0)), "test operands");
    assert_eq!(state.globals, hashmap!("skippy".to_string(), Pointer::from(0)), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn set_global() {
    let code = Code::from(vec!(
        OpCode::SetGlobal { name: ConstantPoolIndex::new(0) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("skippy".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!(ConstantPoolIndex::new(0));
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.push_operand(Pointer::from(42i32));
    state.register_global("skippy".to_string(), Pointer::from(666i32)).unwrap();

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(0)), "test operands");
    assert_eq!(state.globals, hashmap!("skippy".to_string(), Pointer::from(0)), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn drop() {
    let code = Code::from(vec!(
        OpCode::Drop,
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!();
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.push_operand(Pointer::from(7i32));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, Vec::new(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn jump() {
    let code = Code::from(vec!(
        /*0*/ OpCode::Label { name: ConstantPoolIndex::new(0) },
        /*1*/ OpCode::Return,
        /*2*/ OpCode::Jump { label: ConstantPoolIndex::new(0) },
        /*3*/ OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("^.^".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.set_instruction_pointer(Some(Address::from_usize(2)));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, Vec::new(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(0)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory")
}

#[test] fn branch_true() {
    let code = Code::from(vec!(
        /*0*/ OpCode::Label { name: ConstantPoolIndex::new(0) },
        /*1*/ OpCode::Return,
        /*2*/ OpCode::Branch { label: ConstantPoolIndex::new(0) },
        /*3*/ OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("x.x".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.set_instruction_pointer(Some(Address::from_usize(2)));
    state.push_operand(Pointer::from(true));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, Vec::new(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(0)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn branch_false() {
    let code = Code::from(vec!(
        /*0*/ OpCode::Label { name: ConstantPoolIndex::new(0) },
        /*1*/ OpCode::Return,
        /*2*/ OpCode::Branch { label: ConstantPoolIndex::new(0) },
        /*3*/ OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("butt".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.set_instruction_pointer(Some(Address::from_usize(2)));
    state.push_operand(Pointer::from(false));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, Vec::new(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(3)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn print() {
    let code = Code::from(vec!(
        OpCode::Print { format: ConstantPoolIndex::new(0), arguments: Arity::new(0) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("Ahoj przygodo!\n".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "Ahoj przygodo!\n", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(0)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn print_one() {
    let code = Code::from(vec!(
        OpCode::Print { format: ConstantPoolIndex::new(0), arguments: Arity::new(1) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("~!\n".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.push_operand(Pointer::from(2i32));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "42!\n", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(1)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory")
}

#[test] fn print_two() {
    let code = Code::from(vec!(
        OpCode::Print { format: ConstantPoolIndex::new(0), arguments: Arity::new(2) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("~x~!\n".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.push_operand(Pointer::from(0i32));
    state.push_operand(Pointer::from(2i32));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "0x42!\n", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(2)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory")
}

#[test] fn array_zero() {
    let code = Code::from(vec!(
        OpCode::Array,
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!();
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.push_operand(Pointer::from(0i32));
    state.push_operand(Pointer::Null);

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(2)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(HeapObject::empty_array())), "test memory");
}

#[test] fn array_one() {
    let code = Code::from(vec!(
        OpCode::Array,
        OpCode::Return
    ));

    let constants: Vec<ProgramObject> = vec!();
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.push_operand(Pointer::from(1i32));
    state.push_operand(Pointer::Null);

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(3)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(HeapObject::from_pointers(vec!(Pointer::from(2))))), "test memory");
}

#[test] fn array_three() {
    let code = Code::from(vec!(
        OpCode::Array,
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!();
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.push_operand(Pointer::from(3i32));
    state.push_operand(Pointer::from(0i32));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(5)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(
                                             HeapObject::from_pointers(vec!(Pointer::from(0i32),
                                                                            Pointer::from(0i32),
                                                                            Pointer::from(0i32))))), "test memory");
}

#[test] fn call_function_zero() {
    let code = Code::from(vec!(
        /*0*/ OpCode::Return,
        /*1*/ OpCode::CallFunction { name: ConstantPoolIndex::new(0), arguments: Arity::new(0) },
        /*2*/ OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(
        ProgramObject::String("bar".to_string()),
        ProgramObject::Method { name: ConstantPoolIndex::new(0),
            arguments: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(0,1) });

    let mut state = State::minimal();
    state.functions.insert("bar".to_string(), constants.get(1).unwrap().clone());
    state.set_instruction_pointer(Some(Address::from_usize(1)));

    let globals: Vec<ConstantPoolIndex> = vec!(ConstantPoolIndex::new(1));
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut output: String = String::new();


    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(0)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty(),
                                  LocalFrame::from(Some(Address::from_usize(2)), vec!())), "test frames");
    assert_eq!(state.memory, Heap::new())
}

#[test] fn call_function_one() {
    let code = Code::from(vec!(
        /*0*/ OpCode::Return,
        /*1*/ OpCode::CallFunction { name: ConstantPoolIndex::new(0), arguments: Arity::new(1) },
        /*2*/ OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(
        ProgramObject::String("foo".to_string()),
        ProgramObject::Method { name: ConstantPoolIndex::new(0),
            arguments: Arity::new(1),
            locals: Size::new(0),
            code: AddressRange::from(0,1) });

    let mut state = State::minimal();
    state.functions.insert("foo".to_string(), constants.get(1).unwrap().clone());
    state.push_operand(Pointer::from(2i32));
    state.set_instruction_pointer(Some(Address::from_usize(1)));

    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut output: String = String::new();

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(0)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty(),
                                  LocalFrame::from(Some(Address::from_usize(2)),
                                                   vec!(Pointer::from(0)))), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn call_function_three() {
    let code = Code::from(vec!(
        /*0*/ OpCode::Return,
        /*1*/ OpCode::CallFunction { name: ConstantPoolIndex::new(0), arguments: Arity::new(3) },
        /*2*/ OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(
        ProgramObject::String("fun".to_string()),
        ProgramObject::Method { name: ConstantPoolIndex::new(0),
            arguments: Arity::new(3),
            locals: Size::new(0),
            code: AddressRange::from(0,1) });

    let mut state = State::minimal();
    state.functions.insert("fun".to_string(), constants.get(1).unwrap().clone());

    state.push_operand(Pointer::from(1i32));
    state.push_operand(Pointer::from(2i32));
    state.push_operand(Pointer::from(3i32));

    state.set_instruction_pointer(Some(Address::from_usize(1)));

    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut output: String = String::new();

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(0)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty(),
                                  LocalFrame::from(Some(Address::from_usize(2)),
                                                   vec!(Pointer::from(0),
                                                        Pointer::from(1),
                                                        Pointer::from(2),))), "test frames");
    assert_eq!(state.memory, Heap::new())
}

#[test] fn returns() {
    let code = Code::from(vec!(
        /*0*/ OpCode::Return,
        /*1*/ OpCode::CallFunction { name: ConstantPoolIndex::new(1), arguments: Arity::new(3) },
        // /*2*/ OpCode::Skip,
    ));

    let constants: Vec<ProgramObject> = vec!(
        ProgramObject::String("xxx".to_string()),
        ProgramObject::Method { name: ConstantPoolIndex::new(0),
            arguments: Arity::new(3),
            locals: Size::new(0),
            code: AddressRange::from(0,1) });
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    //state.set_instruction_pointer(Some(Address::from_usize(0)));

    let pointer1 = Pointer::from(1i32);
    let pointer2 = Pointer::from(2i32);
    let pointer3 = Pointer::from(3i32);
    state.new_frame(Some(Address::from_usize(2)),
                    vec!(pointer1, pointer2, pointer3));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(2)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::new(), "test memory");
}

#[test] fn object_zero() {
    let code = Code::from(vec!(
        /*0*/ OpCode::Return,
        /*1*/ OpCode::Object { class: ConstantPoolIndex::new(2) },
        /*2*/ OpCode::Return
    ));

    let constants: Vec<ProgramObject> = vec!(
        /*0*/ ProgramObject::String ("+".to_string()),
        /*1*/ ProgramObject::Method { name: ConstantPoolIndex::new(0),
            arguments: Arity::new(1),
            locals: Size::new(0),
            code: AddressRange::from(0, 1)},

        /*2*/ ProgramObject::Class(vec!(ConstantPoolIndex::new(1))),
    );
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.set_instruction_pointer(Some(Address::from_usize(1)));
    state.push_operand(Pointer::Null);

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(1)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(2)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(
                                             HeapObject::from(Pointer::Null,
                                                              HashMap::new(),
                                                              hashmap!("+".to_string(), ProgramObject::Method { name: ConstantPoolIndex::new(0),
                                                                                                                  arguments: Arity::new(1),
                                                                                                                  locals: Size::new(0),
                                                                                                                  code: AddressRange::from(0, 1)})))), "test memory");
}

#[test] fn object_one() {
    let code = Code::from(vec!(
        /*0*/ OpCode::Return,
        /*1*/ OpCode::Object { class: ConstantPoolIndex::new(4) },
        /*2*/ OpCode::Return
    ));

    let constants: Vec<ProgramObject> = vec!(
        /*0*/ ProgramObject::String ("x".to_string()),
        /*1*/ ProgramObject::Slot { name: ConstantPoolIndex::new(0) },

        /*2*/ ProgramObject::String ("+".to_string()),
        /*3*/ ProgramObject::Method { name: ConstantPoolIndex::new(2),
            arguments: Arity::new(1),
            locals: Size::new(0),
            code: AddressRange::from(0, 1)},

        /*4*/ ProgramObject::Class(vec!(ConstantPoolIndex::new(1),
                                        ConstantPoolIndex::new(3))),
    );
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.set_instruction_pointer(Some(Address::from_usize(1)));
    state.push_operand(Pointer::Null);          // parent
    state.push_operand(Pointer::from(0i32));     // x

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(2)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(2)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(
                                             HeapObject::from(Pointer::Null,
                                                              hashmap!("x".to_string(), Pointer::from(0i32)),
                                                              hashmap!("+".to_string(), ProgramObject::Method { name: ConstantPoolIndex::new(2),
                                                                                                                  arguments: Arity::new(1),
                                                                                                                  locals: Size::new(0),
                                                                                                                  code: AddressRange::from(0, 1)})))));
}

#[test] fn object_two() {
    let code = Code::from(vec!(
        /*0*/ OpCode::Return,
        /*1*/ OpCode::Object { class: ConstantPoolIndex::new(6) },
        /*2*/ OpCode::Return
    ));

    let constants: Vec<ProgramObject> = vec!(
        /*0*/ ProgramObject::String ("x".to_string()),
        /*1*/ ProgramObject::Slot { name: ConstantPoolIndex::new(0) },

        /*2*/ ProgramObject::String ("y".to_string()),
        /*3*/ ProgramObject::Slot { name: ConstantPoolIndex::new(2) },

        /*4*/ ProgramObject::String ("+".to_string()),
        /*5*/ ProgramObject::Method { name: ConstantPoolIndex::new(4),
            arguments: Arity::new(1),
            locals: Size::new(0),
            code: AddressRange::from(0, 1)},

        /*6*/ ProgramObject::Class(vec!(ConstantPoolIndex::new(1),
                                        ConstantPoolIndex::new(3),
                                        ConstantPoolIndex::new(5))),
    );
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.set_instruction_pointer(Some(Address::from_usize(1)));
    state.push_operand(Pointer::Null);                 // parent
    state.push_operand(Pointer::from(0i32));       // x
    state.push_operand(Pointer::from(2i32));      // y


    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(3)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(2)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(
                                             HeapObject::from(Pointer::Null,
                                                              hashmap!("x".to_string(), Pointer::from(0i32), "y".to_string(), Pointer::from(42i32)),
                                                              hashmap!("+".to_string(), ProgramObject::Method {
                                                                                            name: ConstantPoolIndex::new(4),
                                                                                            arguments: Arity::new(1),
                                                                                            locals: Size::new(0),
                                                                                            code: AddressRange::from(0, 1)})))));
}

#[test] fn get_slot() {
    let code = Code::from(vec!(
        OpCode::GetField { name: ConstantPoolIndex::new(0) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("value".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.allocate_and_push_operand(HeapObject::from(Pointer::Null,
                                                     hashmap!("value".to_string(), Pointer::from(42i32)),
                                                     HashMap::new()));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(1)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(
                                             HeapObject::from(Pointer::Null,
                                                              hashmap!("value".to_string(), Pointer::from(42i32)),
                                                              HashMap::new()))));
}

#[test] fn set_slot() {
    let code = Code::from(vec!(
        OpCode::SetField { name: ConstantPoolIndex::new(0) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("value".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    let object = HeapObject::from(Pointer::Null,
                                  hashmap!("value".to_string(), Pointer::from(1)),
                                  HashMap::new());

//        state.allocate_and_push_operand(Object::from_i32(42));
    state.allocate_and_push_operand(object.clone());
    state.push_operand(Pointer::from(6i32));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(2)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(
                                             HeapObject::from(Pointer::Null,
                                                              hashmap!("value".to_string(), Pointer::from(666i32)),
                                                              HashMap::new()))));

    assert_eq!(object, HeapObject::from(Pointer::from(0),
                                        hashmap!("value".to_string(), Pointer::from(1)),
                                        HashMap::new()));
}

#[test] fn call_method_zero() {
    let code = Code::from(vec!(
        OpCode::Return,
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(0 + 1) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("f".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    let receiver = HeapObject::from(Pointer::from(0),
                                    HashMap::new(),
                                    hashmap!("f".to_string(), ProgramObject::Method { name: ConstantPoolIndex::new(0),
                                                                                      arguments: Arity::new(0 + 1),
                                                                                      locals: Size::new(0),
                                                                                      code: AddressRange::from(0, 1) }));

    state.set_instruction_pointer(Some(Address::from_usize(1)));
    //state.allocate(HeapObject::Null);
    state.allocate_and_push_operand(receiver.clone());

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, Vec::new(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(0)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty(),
                                  LocalFrame::from(Some(Address::from_usize(2)),
                                                   vec!(Pointer::from(1)))), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(receiver.clone())))
}

#[test] fn call_method_one() {
    let code = Code::from(vec!(
        OpCode::Return,
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(1 + 1) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("+".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    let receiver = HeapObject::from(Pointer::from(0),
                                    HashMap::new(),
                                    hashmap!("+".to_string(), ProgramObject::Method { name: ConstantPoolIndex::new(0),
                                                                                      arguments: Arity::new(1 + 1),
                                                                                      locals: Size::new(0),
                                                                                      code: AddressRange::from(0, 1) }));

    state.set_instruction_pointer(Some(Address::from_usize(1)));
    //state.allocate(HeapObject::Null);
    state.allocate_and_push_operand(receiver.clone());
    state.push_operand(Pointer::from(1i32));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, Vec::new(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(0)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty(),
                                  LocalFrame::from(Some(Address::from_usize(2)),
                                                   vec!(Pointer::from(1usize),
                                                        Pointer::from(1i32)))), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(receiver.clone())))
}

#[test] fn call_method_three() {
    let code = Code::from(vec!(
        OpCode::Return,
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(3 + 1) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("g".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    let receiver = HeapObject::from(Pointer::from(0),
                                    HashMap::new(),
                                    hashmap!("g".to_string(), ProgramObject::Method { name: ConstantPoolIndex::new(0),
                                                                                      arguments: Arity::new(3 + 1),
                                                                                      locals: Size::new(0),
                                                                                      code: AddressRange::from(0, 1) }));

    state.set_instruction_pointer(Some(Address::from_usize(1)));
    //state.allocate(HeapObject::Null);
    state.allocate_and_push_operand(receiver.clone());
    state.push_operand(Pointer::from(1i32));
    state.push_operand(Pointer::from(2i32));
    state.push_operand(Pointer::from(3i32));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, Vec::new(), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(0)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty(),
                                  LocalFrame::from(Some(Address::from_usize(2)),
                                                   vec!(Pointer::from(1),
                                                        Pointer::from(1i32),
                                                        Pointer::from(2i32),
                                                        Pointer::from(3i32),))), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(receiver.clone())))
}

fn call_method(receiver: HeapObject, argument: HeapObject, operation: &str, result: HeapObject) {
    let code = Code::from(vec!(
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(1 + 1) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String(operation.to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.set_instruction_pointer(Some(Address::from_usize(0)));
    state.allocate_and_push_operand(receiver.clone());
    state.allocate_and_push_operand(argument.clone());

    interpret(&mut state, &mut output, &program);

    let mut expected_memory = Heap::new();
    expected_memory.allocate(receiver.clone());
    expected_memory.allocate(argument.clone());
    let result_pointer = Pointer::from(expected_memory.allocate(result.clone()));

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(result_pointer), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, expected_memory)
}

fn call_method_on_pointers(receiver: Pointer, argument: Pointer, operation: &str, result: Pointer) {
    // call_method(HeapObject::from_i32(receiver),
    //             HeapObject::from_i32(argument),
    //             operation,
    //             HeapObject::from_i32(result));
    let code = Code::from(vec!(
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(1 + 1) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String(operation.to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.set_instruction_pointer(Some(Address::from_usize(0)));
    state.push_operand(Pointer::from(receiver));
    state.push_operand(Pointer::from(argument));

    interpret(&mut state, &mut output, &program);

    let mut expected_memory = Heap::new();
    let result_pointer = Pointer::from(result);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(result_pointer), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, expected_memory)
}

fn call_method_integer(receiver: i32, argument: i32, operation: &str, result: i32) {
    call_method_on_pointers(Pointer::from(receiver),
                Pointer::from(argument),
                operation,
                Pointer::from(result));
}


fn call_method_integer_cmp(receiver: i32, argument: i32, operation: &str, result: bool) {
    call_method_on_pointers(Pointer::from(receiver),
                Pointer::from(argument),
                operation,
                Pointer::from(result));
}

fn call_method_boolean(receiver: bool, argument: bool, operation: &str, result: bool) {
    call_method_on_pointers(Pointer::from(receiver),
                Pointer::from(argument),
                operation,
                Pointer::from(result));
}

#[test] fn call_method_integer_add() {
    call_method_integer(2, 5, "+", 7);
    call_method_integer(2, 5, "add", 7);
}

#[test] fn call_method_integer_subtract() {
    call_method_integer(2, 5, "-", -3);
    call_method_integer(2, 5, "sub", -3);
}

#[test] fn call_method_integer_multiply() {
    call_method_integer(2, 5, "*", 10);
    call_method_integer(2, 5, "mul", 10);
}

#[test] fn call_method_integer_divide() {
    call_method_integer(2, 5, "/", 0);
    call_method_integer(2, 5, "div", 0);
}

#[test] fn call_method_integer_module() {
    call_method_integer(2, 5, "%", 2);
    call_method_integer(2, 5, "mod", 2);
}

#[test] fn call_method_integer_equality() {
    call_method_integer_cmp(2, 5, "==", false);
    call_method_integer_cmp(5, 5, "==", true);
    call_method_integer_cmp(2, 5, "eq", false);
    call_method_integer_cmp(5, 5, "eq", true);
}

#[test] fn call_method_integer_inequality() {
    call_method_integer_cmp(2, 5, "!=", true);
    call_method_integer_cmp(2, 2, "!=", false);
    call_method_integer_cmp(2, 5, "neq", true);
    call_method_integer_cmp(2, 2, "neq", false);
}

#[test] fn call_method_integer_less() {
    call_method_integer_cmp(2, 5, "<", true);
    call_method_integer_cmp(7, 5, "<", false);
    call_method_integer_cmp(5, 5, "<", false);
    call_method_integer_cmp(2, 5, "lt", true);
    call_method_integer_cmp(7, 5, "lt", false);
    call_method_integer_cmp(5, 5, "lt", false);
}

#[test] fn call_method_integer_less_equal() {
    call_method_integer_cmp(2, 5, "<=", true);
    call_method_integer_cmp(7, 5, "<=", false);
    call_method_integer_cmp(5, 5, "<=", true);
    call_method_integer_cmp(2, 5, "le", true);
    call_method_integer_cmp(7, 5, "le", false);
    call_method_integer_cmp(5, 5, "le", true);
}

#[test] fn call_method_integer_more() {
    call_method_integer_cmp(2, 5, ">", false);
    call_method_integer_cmp(7, 5, ">", true);
    call_method_integer_cmp(5, 5, ">", false);
    call_method_integer_cmp(2, 5, "gt", false);
    call_method_integer_cmp(7, 5, "gt", true);
    call_method_integer_cmp(5, 5, "gt", false);
}

#[test] fn call_method_integer_more_equal() {
    call_method_integer_cmp(2, 5, ">=", false);
    call_method_integer_cmp(7, 5, ">=", true);
    call_method_integer_cmp(5, 5, ">=", true);
    call_method_integer_cmp(2, 5, "ge", false);
    call_method_integer_cmp(7, 5, "ge", true);
    call_method_integer_cmp(5, 5, "ge", true);
}

#[test] fn call_method_boolean_conjunction() {
    call_method_boolean(true, false, "&",   false);
    call_method_boolean(true, true,  "&",   true);
    call_method_boolean(true, false, "and", false);
    call_method_boolean(true, true,  "and", true);
}

#[test] fn call_method_boolean_disjunction() {
    call_method_boolean(true,  false, "|",  true);
    call_method_boolean(false, false, "|",  false);
    call_method_boolean(true,  false, "or", true);
    call_method_boolean(false, false, "or", false);
}

#[test] fn call_method_boolean_equal() {
    call_method_boolean(true,  false, "==",  false);
    call_method_boolean(false, false, "==",  true);
    call_method_boolean(true,  true,  "==",  true);
    call_method_boolean(true,  false, "eq",  false);
    call_method_boolean(false, false, "eq",  true);
    call_method_boolean(true,  true,  "eq",  true);
}

#[test] fn call_method_boolean_unequal() {
    call_method_boolean(true,  false, "!=",  true);
    call_method_boolean(false, false, "!=",  false);
    call_method_boolean(true,  true,  "!=",  false);
    call_method_boolean(true,  false, "neq",  true);
    call_method_boolean(false, false, "neq",  false);
    call_method_boolean(true,  true,  "neq",  false);
}

#[test] fn call_method_array_get() {
    let code = Code::from(vec!(
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(1 + 1) },
        OpCode::Return,
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::from_str("get"));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    state.set_instruction_pointer(Some(Address::from_usize(0)));
    Pointer::from(1i32);
    Pointer::from(2i32);
    Pointer::from(3i32);
    state.allocate_and_push_operand(HeapObject::from_pointers(vec!(Pointer::from(0),
                                                                   Pointer::from(1),
                                                                   Pointer::from(2))));
    state.push_operand(Pointer::from(1i32));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(1)), "test operands");
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(
                                             HeapObject::from_pointers(vec!(Pointer::from(1i32),
                                                                            Pointer::from(2i32),
                                                                            Pointer::from(3i32))))), "test memory")
}

// before: array(1,2,3)
//         a.set(1, 42)
// after:  array(1,42,3)
#[test] fn call_method_array_set() {
    let code = Code::from(vec!(
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(3) },
        OpCode::Return
    ));

    let constants: Vec<ProgramObject> = vec!(ProgramObject::String("set".to_string()));
    let globals: Vec<ConstantPoolIndex> = vec!();
    let entry = ConstantPoolIndex::new(0);
    let program = Program::new(code, constants, globals, entry);

    let mut state = State::minimal();
    let mut output: String = String::new();

    let array = HeapObject::from_pointers(vec!(Pointer::from(0),
                                               Pointer::from(1),
                                               Pointer::from(2)));

    state.set_instruction_pointer(Some(Address::from_usize(0)));
    Pointer::from(1i32);
    Pointer::from(2i32);
    Pointer::from(3i32);
    state.allocate_and_push_operand(array.clone());
    state.push_operand(Pointer::from(1i32));
    state.push_operand(Pointer::from(2i32));

    interpret(&mut state, &mut output, &program);

    assert_eq!(&output, "", "test output");
    assert_eq!(state.operands, vec!(Pointer::from(6)), "test operands");    // returns null
    assert_eq!(state.globals, HashMap::new(), "test globals");
    assert_eq!(state.instruction_pointer, Some(Address::from_usize(1)), "test instruction pointer");
    assert_eq!(state.frames, vec!(LocalFrame::empty()), "test frames");
    assert_eq!(state.memory, Heap::from(vec!(
                                             HeapObject::from_pointers(vec!(Pointer::from(1i32),
                                                                            Pointer::from(42i32),
                                                                            Pointer::from(3i32))))), "test memory");

    assert_eq!(array, HeapObject::from_pointers(vec!(Pointer::from(1i32),
                                                     Pointer::from(42i32),
                                                     Pointer::from(3i32))), "test object state");
}

#[test] fn call_method_null_equals() {
    call_method_on_pointers(Pointer::Null, Pointer::Null, "==", Pointer::from(true));
    call_method_on_pointers(Pointer::Null, Pointer::from(1i32), "==", Pointer::from(false));
    call_method_on_pointers(Pointer::from(1i32), Pointer::Null, "==", Pointer::from(false));

    call_method_on_pointers(Pointer::Null, Pointer::Null, "eq", Pointer::from(true));
    call_method_on_pointers(Pointer::Null, Pointer::from(1i32), "eq", Pointer::from(false));
    call_method_on_pointers(Pointer::from(1i32), Pointer::Null, "eq", Pointer::from(false));
}

#[test] fn call_method_null_unequals() {
    call_method_on_pointers(Pointer::Null, Pointer::Null, "!=", Pointer::from(false));
    call_method_on_pointers(Pointer::Null, Pointer::from(1i32), "!=", Pointer::from(true));
    call_method_on_pointers(Pointer::from(1i32), Pointer::Null, "!=", Pointer::from(true));

    call_method_on_pointers(Pointer::Null, Pointer::Null, "neq", Pointer::from(false));
    call_method_on_pointers(Pointer::Null, Pointer::from(1i32), "neq", Pointer::from(true));
    call_method_on_pointers(Pointer::from(1i32), Pointer::Null, "neq", Pointer::from(true));
}

