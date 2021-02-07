use std::io::Cursor;

use crate::bytecode::bytecode::*;
use crate::bytecode::serializable::*;
use crate::bytecode::types::*;
use crate::bytecode::program::*;
use crate::bytecode::objects::*;

fn deserialize_test(expected: OpCode, input: Vec<u8>) {
    assert_eq!(OpCode::from_bytes(&mut Cursor::new(input)), expected);
}

fn deserialize_with_context_test(expected_object: ProgramObject, expected_code: Code, input: Vec<u8>) {
    let mut code = Code::new();
    let object = ProgramObject::from_bytes(&mut Cursor::new(input), &mut code);
    assert_eq!(object, expected_object);
    assert_eq!(code, expected_code);
}

fn serialize_test<S>(expected: Vec<u8>, object: S) where S: Serializable {
    let mut actual: Vec<u8> = Vec::new();
    object.serialize(&mut actual).unwrap();
    assert_eq!(actual, expected);
}

fn serialize_with_context_test<S>(expected: Vec<u8>, object: S, code: Code) where S: SerializableWithContext {
    let mut actual: Vec<u8> = Vec::new();
    object.serialize(&mut actual, &code).unwrap();
    assert_eq!(actual, expected);
}

#[test] fn deserialize_label () {
    let expected = OpCode::Label { name: ConstantPoolIndex::new(1) };
    let bytes = vec!(0x00, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_literal () {
    let expected = OpCode::Literal { index: ConstantPoolIndex::new(1) };
    let bytes = vec!(0x01, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_get_local () {
    let expected = OpCode::GetLocal { index: LocalFrameIndex::new(1) };
    let bytes = vec!(0x0A, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_set_local () {
    let expected = OpCode::SetLocal { index: LocalFrameIndex::new(1) };
    let bytes = vec!(0x09, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_get_global () {
    let expected = OpCode::GetGlobal { name: ConstantPoolIndex::new(1) };
    let bytes = vec!(0x0C, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_set_global () {
    let expected = OpCode::SetGlobal { name: ConstantPoolIndex::new(1) };
    let bytes = vec!(0x0B, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_object () {
    let expected = OpCode::Object { class: ConstantPoolIndex::new(1) };
    let bytes = vec!(0x04, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_array () {
    let expected = OpCode::Array;
    let bytes = vec!(0x03);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_get_slot () {
    let expected = OpCode::GetSlot { name: ConstantPoolIndex::new(1) };
    let bytes = vec!(0x05, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_set_slot () {
    let expected = OpCode::SetSlot { name: ConstantPoolIndex::new(1) };
    let bytes = vec!(0x06, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_call_method () {
    let expected = OpCode::CallMethod { name: ConstantPoolIndex::new(1), arguments: Arity::new(1) };
    let bytes = vec!(0x07, 0x01, 0x00, 0x01);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_call_function () {
    let expected = OpCode::CallFunction { name: ConstantPoolIndex::new(1), arguments: Arity::new(2) };
    let bytes = vec!(0x08, 0x01, 0x00, 0x02);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_print () {
    let expected = OpCode::Print { format: ConstantPoolIndex::new(1), arguments: Arity::new(2) };
    let bytes = vec!(0x02, 0x01, 0x00, 0x02);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_jump () {
    let expected = OpCode::Jump { label: ConstantPoolIndex::new(1) };
    let bytes = vec!(0x0E, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_branch () {
    let expected = OpCode::Branch { label: ConstantPoolIndex::new(1) };
    let bytes = vec!(0x0D, 0x01, 0x00, 0x00, 0x00);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_return_op () {
    let expected = OpCode::Return;
    let bytes = vec!(0x0F);
    deserialize_test(expected, bytes);
}

#[test] fn deserialize_drop () {
    let expected = OpCode::Drop;
    let bytes = vec!(0x10);
    deserialize_test(expected, bytes);
}

#[test] fn serialize_label () {
    let expected = vec!(0x00, 0x01, 0x00);
    let object = OpCode::Label { name: ConstantPoolIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_literal () {
    let expected = vec!(0x01, 0x01, 0x00, );
    let object = OpCode::Literal { index: ConstantPoolIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_get_local () {
    let expected = vec!(0x0A, 0x01, 0x00, );
    let object = OpCode::GetLocal { index: LocalFrameIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_set_local () {
    let expected = vec!(0x09, 0x01, 0x00,);
    let object = OpCode::SetLocal { index: LocalFrameIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_get_global () {
    let expected = vec!(0x0C, 0x01, 0x00, );
    let object = OpCode::GetGlobal { name: ConstantPoolIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_set_global () {
    let expected = vec!(0x0B, 0x01, 0x00, );
    let object = OpCode::SetGlobal { name: ConstantPoolIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_object () {
    let expected = vec!(0x04, 0x01, 0x00, );
    let object = OpCode::Object { class: ConstantPoolIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_array () {
    let expected = vec!(0x03);
    let object = OpCode::Array;
    serialize_test(expected, object);
}

#[test] fn serialize_get_slot () {
    let expected = vec!(0x05, 0x01, 0x00, );
    let object = OpCode::GetSlot { name: ConstantPoolIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_set_slot () {
    let expected = vec!(0x06, 0x01, 0x00, );
    let object = OpCode::SetSlot { name: ConstantPoolIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_call_method () {
    let expected = vec!(0x07, 0x01, 0x00, 0x01);
    let object = OpCode::CallMethod { name: ConstantPoolIndex::new(1), arguments: Arity::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_call_function () {
    let expected = vec!(0x08, 0x01, 0x00, 0x02);
    let object = OpCode::CallFunction { name: ConstantPoolIndex::new(1), arguments: Arity::new(2) };
    serialize_test(expected, object);
}

#[test] fn serialize_print () {
    let expected = vec!(0x02, 0x01, 0x00, 0x02);
    let object = OpCode::Print { format: ConstantPoolIndex::new(1), arguments: Arity::new(2) };
    serialize_test(expected, object);
}

#[test] fn serialize_jump () {
    let expected = vec!(0x0E, 0x01, 0x00, );
    let object = OpCode::Jump { label: ConstantPoolIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_branch () {
    let expected = vec!(0x0D, 0x01, 0x00, );
    let object = OpCode::Branch { label: ConstantPoolIndex::new(1) };
    serialize_test(expected, object);
}

#[test] fn serialize_return_op () {
    let expected = vec!(0x0F);
    let object = OpCode::Return;
    serialize_test(expected, object);
}

#[test] fn serialize_drop () {
    let expected = vec!(0x10);
    let object = OpCode::Drop;
    serialize_test(expected, object);
}

#[test] fn serialize_null () {
    let expected = vec!(0x01);
    let object = ProgramObject::Null;
    serialize_with_context_test(expected, object, Code::new());
}

#[test] fn serialize_integer () {
    let expected = vec!(0x00, 0x2A, 0x00, 0x00, 0x00);
    let object = ProgramObject::Integer(42);
    serialize_with_context_test(expected, object, Code::new());
}

#[test] fn serialize_boolean () {
    let expected = vec!(0x06, 0x01);
    let object = ProgramObject::Boolean(true);
    serialize_with_context_test(expected, object, Code::new());
}

#[test] fn serialize_string () {
    let expected = vec!(0x02,
                        0x0C, 0x00, 0x00, 0x00,
                        0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x0A);
    let object = ProgramObject::String("Hello World\n".to_string());
    serialize_with_context_test(expected, object, Code::new());
}

#[test] fn serialize_slot () {
    let expected = vec!(0x04, 0x2A, 0x00);
    let object = ProgramObject::Slot { name: ConstantPoolIndex::new(42) };
    serialize_with_context_test(expected, object, Code::new());
}

#[test] fn serialize_class () {
    let expected = vec!(0x05,
                        0x02, 0x00,
                        0x2A, 0x00,
                        0x9A, 0x02, );
    let object = ProgramObject::Class(vec!(ConstantPoolIndex::new(42),
                                           ConstantPoolIndex::new(666)));
    serialize_with_context_test(expected, object, Code::new());
}

#[test] fn serialize_method () {
    let expected = vec!(0x03,
                        0xFF, 0x00,
                        0x03,
                        0x0F, 0x00,
                        0x02, 0x00, 0x00, 0x00,
                        0x01,
                        0x2A, 0x00,
                        0x0F);

    let object = ProgramObject::Method {
        name: ConstantPoolIndex::new(255),
        arguments: Arity::new(3),
        locals: Size::new(15),
        code: AddressRange::from(0, 2),
    };

    let code = Code::from(
        vec!(
            /* 0 */ OpCode::Literal { index: ConstantPoolIndex::new(42) },
            /* 1 */ OpCode::Return));

    serialize_with_context_test(expected, object, code);
}

#[test] fn null () {
    let expected = ProgramObject::Null;
    let bytes = vec!(0x01);
    deserialize_with_context_test(expected, Code::new(), bytes);
}

#[test] fn integer () {
    let expected = ProgramObject::Integer(42);
    let bytes = vec!(0x00, 0x2A, 0x00, 0x00, 0x00);
    deserialize_with_context_test(expected, Code::new(),bytes);
}

#[test] fn boolean () {
    let expected = ProgramObject::Boolean(true);
    let bytes = vec!(0x06, 0x01);
    deserialize_with_context_test(expected, Code::new(),bytes);
}

#[test] fn string () {
    let expected = ProgramObject::String("Hello World\0".to_string());
    let bytes = vec!(0x02,
                     0x0C, 0x00, 0x00, 0x00,
                     0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x00);
    deserialize_with_context_test(expected, Code::new(),bytes);
}

#[test] fn slot () {
    let expected = ProgramObject::Slot { name: ConstantPoolIndex::new(42) };
    let bytes = vec!(0x04, 0x2A, 0x00, );
    deserialize_with_context_test(expected, Code::new(),bytes);
}

#[test] fn class () {
    let expected = ProgramObject::Class(vec!(ConstantPoolIndex::new(42),
                                             ConstantPoolIndex::new(666)));
    let bytes = vec!(0x05,
                     0x02, 0x00,
                     0x2A, 0x00,
                     0x9A, 0x02, );
    deserialize_with_context_test(expected, Code::new(),bytes);
}


#[test] fn method () {
    let object = ProgramObject::Method { name: ConstantPoolIndex::new(255),
                                         arguments: Arity::new(3),
                                         locals: Size::new(15),
                                         code: AddressRange::from(0, 2)};

    let code = Code::from(vec!(OpCode::Literal { index: ConstantPoolIndex::new(42) },
                               OpCode::Return));

    let bytes = vec!(0x03,
                     0xFF, 0x00,
                     0x03,
                     0x0F, 0x00,
                     0x02, 0x00, 0x00, 0x00,
                     0x01,
                     0x2A, 0x00,
                     0x0F);

    deserialize_with_context_test(object, code, bytes);
}