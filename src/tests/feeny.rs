use std::io::Cursor;

use crate::bytecode::bytecode::*;
use crate::bytecode::debug::*;
use crate::bytecode::interpreter::*;
use crate::bytecode::program::*;
use crate::bytecode::serializable::*;
use crate::bytecode::state::*;

/*
 * defn main () :
 *     var x = object :
 *        method m (a, b, c) :
 *            printf("~", a)
 *            printf("~", b)
 *            printf("~", c)
 *
 *     x.m(1, 2, 3)
 *
 * main()
 */
fn feeny_method_argument_order_source() -> &'static str {
    r#"Constants :
    #0: Null
    #1: String("~")
    #2: String("m")
    #3: Method(#2, nargs:4, nlocals:0) :
          get local 1
          printf #1 1
          drop
          get local 2
          printf #1 1
          drop
          get local 3
          printf #1 1
          return
    #4: Class(#3)
    #5: Int(1)
    #6: Int(2)
    #7: Int(3)
    #8: String("main")
    #9: Method(#8, nargs:0, nlocals:1) :
          lit #0
          object #4
          set local 0
          drop
          get local 0
          lit #5
          lit #6
          lit #7
          call slot #2 4
          return
    #10: String("entry38")
    #11: Method(#10, nargs:0, nlocals:0) :
          call #8 0
          drop
          lit #0
          return
Globals :
    #9
Entry : #11"#
}

fn feeny_method_argument_order_program() -> Program {
    let code = Code::from(vec![
        /*  0 */
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(1) },
        /*  1 */
        OpCode::Print { format: ConstantPoolIndex::from_usize(1), arguments: Arity::from_usize(1) },
        /*  2 */ OpCode::Drop,
        /*  3 */
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(2) },
        /*  4 */
        OpCode::Print { format: ConstantPoolIndex::from_usize(1), arguments: Arity::from_usize(1) },
        /*  5 */ OpCode::Drop,
        /*  6 */
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(3) },
        /*  7 */
        OpCode::Print { format: ConstantPoolIndex::from_usize(1), arguments: Arity::from_usize(1) },
        /*  8 */ OpCode::Return,
        /*  9 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(0) },
        /* 10 */
        OpCode::Object { class: ConstantPoolIndex::from_usize(4) },
        /* 11 */
        OpCode::SetLocal { index: LocalFrameIndex::from_usize(0) },
        /* 12 */ OpCode::Drop,
        /* 13 */
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(0) },
        /* 14 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(5) },
        /* 15 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(6) },
        /* 16 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(7) },
        /* 17 */
        OpCode::CallMethod {
            name: ConstantPoolIndex::from_usize(2),
            arguments: Arity::from_usize(4),
        },
        /* 18 */ OpCode::Return,
        /* 19 */
        OpCode::CallFunction { name: ConstantPoolIndex::new(8), arguments: Arity::new(0) },
        /* 20 */ OpCode::Drop,
        /* 21 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(0) },
        /* 22 */ OpCode::Return,
    ]);

    let constant_pool = ConstantPool::from(vec![
        /*  0 */ ProgramObject::Null,
        /*  1 */ ProgramObject::from_str("~"),
        /*  2 */ ProgramObject::from_str("m"),
        /*  3 */
        ProgramObject::Method {
            name: ConstantPoolIndex::from_usize(2),
            parameters: Arity::from_usize(4),
            locals: Size::from_usize(0),
            code: AddressRange::from(0, 9),
        },
        /*  4 */ ProgramObject::Class(vec![ConstantPoolIndex::from_usize(3)]),
        /*  5 */ ProgramObject::from_i32(1),
        /*  6 */ ProgramObject::from_i32(2),
        /*  7 */ ProgramObject::from_i32(3),
        /*  8 */ ProgramObject::from_str("main"),
        /*  9 */
        ProgramObject::Method {
            name: ConstantPoolIndex::from_usize(8),
            parameters: Arity::from_usize(0),
            locals: Size::from_usize(1),
            code: AddressRange::from(9, 10),
        },
        /* 10 */ ProgramObject::from_str("entry38"),
        /* 11 */
        ProgramObject::Method {
            name: ConstantPoolIndex::from_usize(10),
            parameters: Arity::from_usize(0),
            locals: Size::from_usize(0),
            code: AddressRange::from(19, 4),
        },
    ]);

    let globals = Globals::from(vec![ConstantPoolIndex::new(9)]);

    Program::from(code, constant_pool, globals, Entry::from(11)).unwrap()
}

fn feeny_method_argument_order_bytes() -> Vec<u8> {
    vec![
        0x0C, 0x00, 0x01, 0x02, 0x01, 0x00, 0x00, 0x00, 0x7E, 0x02, 0x01, 0x00, 0x00, 0x00, 0x6D,
        0x03, 0x02, 0x00, 0x04, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x0A, 0x01, 0x00, 0x02, 0x01,
        0x00, 0x01, 0x10, 0x0A, 0x02, 0x00, 0x02, 0x01, 0x00, 0x01, 0x10, 0x0A, 0x03, 0x00, 0x02,
        0x01, 0x00, 0x01, 0x0F, 0x05, 0x01, 0x00, 0x03, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x02, 0x04, 0x00, 0x00, 0x00, 0x6D,
        0x61, 0x69, 0x6E, 0x03, 0x08, 0x00, 0x00, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x00, 0x04, 0x04, 0x00, 0x09, 0x00, 0x00, 0x10, 0x0A, 0x00, 0x00, 0x01, 0x05, 0x00, 0x01,
        0x06, 0x00, 0x01, 0x07, 0x00, 0x07, 0x02, 0x00, 0x04, 0x0F, 0x02, 0x07, 0x00, 0x00, 0x00,
        0x65, 0x6E, 0x74, 0x72, 0x79, 0x33, 0x38, 0x03, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00,
        0x00, 0x00, 0x08, 0x08, 0x00, 0x00, 0x10, 0x01, 0x00, 0x00, 0x0F, 0x01, 0x00, 0x09, 0x00,
        0x0B, 0x00,
    ]
}

#[test]
fn feeny_method_argument_order_deserialize() {
    let object = Program::from_bytes(&mut Cursor::new(feeny_method_argument_order_bytes()));
    assert_eq!(feeny_method_argument_order_program(), object);
}

#[test]
fn feeny_method_argument_order_serialize() {
    let mut output: Vec<u8> = Vec::new();
    feeny_method_argument_order_program().serialize(&mut output).unwrap();
    assert_eq!(feeny_method_argument_order_bytes(), output);
}

#[test]
fn feeny_method_argument_order_print() {
    let mut bytes: Vec<u8> = Vec::new();
    feeny_method_argument_order_program().pretty_print(&mut bytes);
    assert_eq!(&String::from_utf8(bytes).unwrap(), feeny_method_argument_order_source());
}

#[test]
fn feeny_method_argument_order_eval() {
    let program = feeny_method_argument_order_program();
    let mut state = State::from(&program).unwrap();
    let mut output = String::new();

    evaluate_with(&program, &mut state, &mut output).unwrap();

    assert_eq!(output, "123");
}

/*
 * defn main () :
 *     var x = object :
 *        var a = 1
 *        var b = 2
 *        var c = 3
 *        method m (a, b, c) :
 *            printf("~", this.a)
 *            printf("~", this.b)
 *            printf("~", this.c)
 *
 *     x.m(1, 2, 3)
 *
 * main()
 */
fn feeny_object_member_order_source() -> &'static str {
    r#"Constants :
    #0: Null
    #1: Int(1)
    #2: Int(2)
    #3: Int(3)
    #4: String("a")
    #5: Slot(#4)
    #6: String("b")
    #7: Slot(#6)
    #8: String("c")
    #9: Slot(#8)
    #10: String("~")
    #11: String("m")
    #12: Method(#11, nargs:1, nlocals:0) :
          get local 0
          get slot #4
          printf #10 1
          drop
          get local 0
          get slot #6
          printf #10 1
          drop
          get local 0
          get slot #8
          printf #10 1
          return
    #13: Class(#5, #7, #9, #12)
    #14: String("main")
    #15: Method(#14, nargs:0, nlocals:1) :
          lit #0
          lit #1
          lit #2
          lit #3
          object #13
          set local 0
          drop
          get local 0
          call slot #11 1
          return
    #16: String("entry38")
    #17: Method(#16, nargs:0, nlocals:0) :
          call #14 0
          drop
          lit #0
          return
Globals :
    #15
Entry : #17"#
}

fn feeny_object_member_order_program() -> Program {
    let code = Code::from(vec![
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(0) },
        OpCode::GetField { name: ConstantPoolIndex::from_usize(4) },
        OpCode::Print {
            format: ConstantPoolIndex::from_usize(10),
            arguments: Arity::from_usize(1),
        },
        OpCode::Drop,
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(0) },
        OpCode::GetField { name: ConstantPoolIndex::from_usize(6) },
        OpCode::Print {
            format: ConstantPoolIndex::from_usize(10),
            arguments: Arity::from_usize(1),
        },
        OpCode::Drop,
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(0) },
        OpCode::GetField { name: ConstantPoolIndex::from_usize(8) },
        OpCode::Print {
            format: ConstantPoolIndex::from_usize(10),
            arguments: Arity::from_usize(1),
        },
        OpCode::Return,
        OpCode::Literal { index: ConstantPoolIndex::from_usize(0) },
        OpCode::Literal { index: ConstantPoolIndex::from_usize(1) },
        OpCode::Literal { index: ConstantPoolIndex::from_usize(2) },
        OpCode::Literal { index: ConstantPoolIndex::from_usize(3) },
        OpCode::Object { class: ConstantPoolIndex::from_usize(13) },
        OpCode::SetLocal { index: LocalFrameIndex::from_usize(0) },
        OpCode::Drop,
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(0) },
        OpCode::CallMethod {
            name: ConstantPoolIndex::from_usize(11),
            arguments: Arity::from_usize(1),
        },
        OpCode::Return,
        OpCode::CallFunction {
            name: ConstantPoolIndex::from_usize(14),
            arguments: Arity::from_usize(0),
        },
        OpCode::Drop,
        OpCode::Literal { index: ConstantPoolIndex::from_usize(0) },
        OpCode::Return,
    ]);

    let constant_pool = ConstantPool::from(vec![
        ProgramObject::Null,
        ProgramObject::from_i32(1),
        ProgramObject::from_i32(2),
        ProgramObject::from_i32(3),
        ProgramObject::from_str("a"),
        ProgramObject::Slot { name: ConstantPoolIndex::from_usize(4) },
        ProgramObject::from_str("b"),
        ProgramObject::Slot { name: ConstantPoolIndex::from_usize(6) },
        ProgramObject::from_str("c"),
        ProgramObject::Slot { name: ConstantPoolIndex::from_usize(8) },
        ProgramObject::from_str("~"),
        ProgramObject::from_str("m"),
        ProgramObject::Method {
            name: ConstantPoolIndex::from_usize(11),
            parameters: Arity::from_usize(1),
            locals: Size::from_usize(0),
            code: AddressRange::from(0, 12),
        },
        ProgramObject::Class(vec![
            ConstantPoolIndex::from_usize(5),
            ConstantPoolIndex::from_usize(7),
            ConstantPoolIndex::from_usize(9),
            ConstantPoolIndex::from_usize(12),
        ]),
        ProgramObject::from_str("main"),
        ProgramObject::Method {
            name: ConstantPoolIndex::from_usize(14),
            parameters: Arity::from_usize(0),
            locals: Size::from_usize(1),
            code: AddressRange::from(12, 10),
        },
        ProgramObject::from_str("entry38"),
        ProgramObject::Method {
            name: ConstantPoolIndex::from_usize(16),
            parameters: Arity::from_usize(0),
            locals: Size::from_usize(0),
            code: AddressRange::from(22, 4),
        },
    ]);

    let globals = Globals::from(vec![ConstantPoolIndex::new(15)]);

    Program::from(code, constant_pool, globals, Entry::from(17)).unwrap()
}

fn feeny_object_member_order_bytes() -> Vec<u8> {
    vec![
        0x12, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x61, 0x04, 0x04, 0x00, 0x02, 0x01, 0x00,
        0x00, 0x00, 0x62, 0x04, 0x06, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x63, 0x04, 0x08, 0x00,
        0x02, 0x01, 0x00, 0x00, 0x00, 0x7E, 0x02, 0x01, 0x00, 0x00, 0x00, 0x6D, 0x03, 0x0B, 0x00,
        0x01, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x05, 0x04, 0x00, 0x02, 0x0A,
        0x00, 0x01, 0x10, 0x0A, 0x00, 0x00, 0x05, 0x06, 0x00, 0x02, 0x0A, 0x00, 0x01, 0x10, 0x0A,
        0x00, 0x00, 0x05, 0x08, 0x00, 0x02, 0x0A, 0x00, 0x01, 0x0F, 0x05, 0x04, 0x00, 0x05, 0x00,
        0x07, 0x00, 0x09, 0x00, 0x0C, 0x00, 0x02, 0x04, 0x00, 0x00, 0x00, 0x6D, 0x61, 0x69, 0x6E,
        0x03, 0x0E, 0x00, 0x00, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x01,
        0x00, 0x01, 0x02, 0x00, 0x01, 0x03, 0x00, 0x04, 0x0D, 0x00, 0x09, 0x00, 0x00, 0x10, 0x0A,
        0x00, 0x00, 0x07, 0x0B, 0x00, 0x01, 0x0F, 0x02, 0x07, 0x00, 0x00, 0x00, 0x65, 0x6E, 0x74,
        0x72, 0x79, 0x33, 0x38, 0x03, 0x10, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08,
        0x0E, 0x00, 0x00, 0x10, 0x01, 0x00, 0x00, 0x0F, 0x01, 0x00, 0x0F, 0x00, 0x11, 0x00,
    ]
}

#[test]
fn feeny_object_member_order_deserialize() {
    let object = Program::from_bytes(&mut Cursor::new(feeny_object_member_order_bytes()));
    assert_eq!(feeny_object_member_order_program(), object);
}

#[test]
fn feeny_object_member_order_serialize() {
    let mut output: Vec<u8> = Vec::new();
    feeny_object_member_order_program().serialize(&mut output).unwrap();
    assert_eq!(feeny_object_member_order_bytes(), output);
}

#[test]
fn feeny_object_member_order_print() {
    let mut bytes: Vec<u8> = Vec::new();
    feeny_object_member_order_program().pretty_print(&mut bytes);
    assert_eq!(&String::from_utf8(bytes).unwrap(), feeny_object_member_order_source());
}

#[test]
fn feeny_object_member_order_eval() {
    let program = feeny_object_member_order_program();
    let mut state = State::from(&program).unwrap();
    let mut output = String::new();

    evaluate_with(&program, &mut state, &mut output).unwrap();

    assert_eq!(output, "123");
}

/*
 *   defn main () :
 *       printf("~~~",1,2,3)
 *
 *   main()
 */
fn feeny_print_argument_order_source() -> &'static str {
    r#"Constants :
    #0: Int(1)
    #1: Int(2)
    #2: Int(3)
    #3: String("~~~")
    #4: String("main")
    #5: Method(#4, nargs:0, nlocals:0) :
          lit #0
          lit #1
          lit #2
          printf #3 3
          return
    #6: Null
    #7: String("entry35")
    #8: Method(#7, nargs:0, nlocals:0) :
          call #4 0
          drop
          lit #6
          return
Globals :
    #5
Entry : #8"#
}

fn feeny_print_argument_order_program() -> Program {
    let code = Code::from(vec![
        /* 0 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(0) },
        /* 1 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(1) },
        /* 2 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(2) },
        /* 3 */
        OpCode::Print { format: ConstantPoolIndex::new(3), arguments: Arity::new(3) },
        /* 4 */ OpCode::Return,
        /* 5 */
        OpCode::CallFunction { name: ConstantPoolIndex::new(4), arguments: Arity::new(0) },
        /* 6 */ OpCode::Drop,
        /* 7 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(6) },
        /* 8 */ OpCode::Return,
    ]);

    let constant_pool = ConstantPool::from(vec![
        /* 0 */ ProgramObject::from_i32(1),
        /* 1 */ ProgramObject::from_i32(2),
        /* 2 */ ProgramObject::from_i32(3),
        /* 3 */ ProgramObject::from_str("~~~"),
        /* 4 */ ProgramObject::from_str("main"),
        /* 5 */
        ProgramObject::Method {
            name: ConstantPoolIndex::new(4),
            parameters: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(0, 5),
        },
        /* 6 */ ProgramObject::Null,
        /* 7 */ ProgramObject::from_str("entry35"),
        /* 8 */
        ProgramObject::Method {
            name: ConstantPoolIndex::new(7),
            parameters: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(5, 4),
        },
    ]);

    let globals = Globals::from(vec![ConstantPoolIndex::new(5)]);

    Program::from(code, constant_pool, globals, Entry::from(8)).unwrap()
}

fn feeny_print_argument_order_bytes() -> Vec<u8> {
    vec![
        0x09, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00,
        0x00, 0x00, 0x02, 0x03, 0x00, 0x00, 0x00, 0x7E, 0x7E, 0x7E, 0x02, 0x04, 0x00, 0x00, 0x00,
        0x6D, 0x61, 0x69, 0x6E, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x01, 0x01, 0x00, 0x01, 0x02, 0x00, 0x02, 0x03, 0x00, 0x03, 0x0F, 0x01, 0x02,
        0x07, 0x00, 0x00, 0x00, 0x65, 0x6E, 0x74, 0x72, 0x79, 0x33, 0x35, 0x03, 0x07, 0x00, 0x00,
        0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08, 0x04, 0x00, 0x00, 0x10, 0x01, 0x06, 0x00, 0x0F,
        0x01, 0x00, 0x05, 0x00, 0x08, 0x00,
    ]
}

#[test]
fn feeny_print_argument_order_deserialize() {
    let object = Program::from_bytes(&mut Cursor::new(feeny_print_argument_order_bytes()));
    assert_eq!(feeny_print_argument_order_program(), object);
}

#[test]
fn feeny_print_argument_order_serialize() {
    let mut output: Vec<u8> = Vec::new();
    feeny_print_argument_order_program().serialize(&mut output).unwrap();
    assert_eq!(feeny_print_argument_order_bytes(), output);
}

#[test]
fn feeny_print_argument_order_print() {
    let mut bytes: Vec<u8> = Vec::new();
    feeny_print_argument_order_program().pretty_print(&mut bytes);
    assert_eq!(&String::from_utf8(bytes).unwrap(), feeny_print_argument_order_source());
}

#[test]
fn feeny_print_argument_order_eval() {
    let program = feeny_print_argument_order_program();
    let mut state = State::from(&program).unwrap();
    let mut output = String::new();

    evaluate_with(&program, &mut state, &mut output).unwrap();

    assert_eq!(output, "123");
}

/*
 *   defn f (a, b, c,) :
 *       printf("~", a);
 *       printf("~", b);
 *       printf("~", c);
 *
 *   defn main () :
 *       f(1,2,3)
 *
 *   main()
 */
fn feeny_function_argument_order_source() -> &'static str {
    r#"Constants :
    #0: String("~")
    #1: String("f")
    #2: Method(#1, nargs:3, nlocals:0) :
          get local 0
          printf #0 1
          drop
          get local 1
          printf #0 1
          drop
          get local 2
          printf #0 1
          return
    #3: Int(1)
    #4: Int(2)
    #5: Int(3)
    #6: String("main")
    #7: Method(#6, nargs:0, nlocals:0) :
          lit #3
          lit #4
          lit #5
          call #1 3
          return
    #8: Null
    #9: String("entry36")
    #10: Method(#9, nargs:0, nlocals:0) :
          call #6 0
          drop
          lit #8
          return
Globals :
    #2
    #7
Entry : #10"#
}

fn feeny_function_argument_order_program() -> Program {
    let code = Code::from(vec![
        /*  0 */
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(0) },
        /*  1 */
        OpCode::Print { format: ConstantPoolIndex::from_usize(0), arguments: Arity::from_usize(1) },
        /*  2 */ OpCode::Drop,
        /*  3 */
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(1) },
        /*  4 */
        OpCode::Print { format: ConstantPoolIndex::from_usize(0), arguments: Arity::from_usize(1) },
        /*  5 */ OpCode::Drop,
        /*  6 */
        OpCode::GetLocal { index: LocalFrameIndex::from_usize(2) },
        /*  7 */
        OpCode::Print { format: ConstantPoolIndex::from_usize(0), arguments: Arity::from_usize(1) },
        /*  8 */ OpCode::Return,
        /*  9 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(3) },
        /* 10 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(4) },
        /* 11 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(5) },
        /* 12 */
        OpCode::CallFunction { name: ConstantPoolIndex::new(1), arguments: Arity::new(3) },
        /* 13 */ OpCode::Return,
        /* 14 */
        OpCode::CallFunction { name: ConstantPoolIndex::new(6), arguments: Arity::new(0) },
        /* 15 */ OpCode::Drop,
        /* 16 */
        OpCode::Literal { index: ConstantPoolIndex::from_usize(8) },
        /* 17 */ OpCode::Return,
    ]);

    let constant_pool = ConstantPool::from(vec![
        /*  0 */ ProgramObject::from_str("~"),
        /*  1 */ ProgramObject::from_str("f"),
        /*  2 */
        ProgramObject::Method {
            name: ConstantPoolIndex::new(1),
            parameters: Arity::new(3),
            locals: Size::new(0),
            code: AddressRange::from(0, 9),
        },
        /*  3 */ ProgramObject::from_i32(1),
        /*  4 */ ProgramObject::from_i32(2),
        /*  5 */ ProgramObject::from_i32(3),
        /*  6 */ ProgramObject::from_str("main"),
        /*  7 */
        ProgramObject::Method {
            name: ConstantPoolIndex::new(6),
            parameters: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(9, 5),
        },
        /*  8 */ ProgramObject::Null,
        /*  9 */ ProgramObject::from_str("entry36"),
        /* 10 */
        ProgramObject::Method {
            name: ConstantPoolIndex::new(9),
            parameters: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(14, 4),
        },
    ]);

    let globals = Globals::from(vec![ConstantPoolIndex::new(2), ConstantPoolIndex::new(7)]);

    Program::from(code, constant_pool, globals, Entry::from(10)).unwrap()
}

fn feeny_function_argument_order_bytes() -> Vec<u8> {
    vec![
        0x0B, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x7E, 0x02, 0x01, 0x00, 0x00, 0x00, 0x66, 0x03,
        0x01, 0x00, 0x03, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x02, 0x00, 0x00,
        0x01, 0x10, 0x0A, 0x01, 0x00, 0x02, 0x00, 0x00, 0x01, 0x10, 0x0A, 0x02, 0x00, 0x02, 0x00,
        0x00, 0x01, 0x0F, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x00, 0x02, 0x04, 0x00, 0x00, 0x00, 0x6D, 0x61, 0x69, 0x6E, 0x03, 0x06, 0x00,
        0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x01, 0x03, 0x00, 0x01, 0x04, 0x00, 0x01, 0x05,
        0x00, 0x08, 0x01, 0x00, 0x03, 0x0F, 0x01, 0x02, 0x07, 0x00, 0x00, 0x00, 0x65, 0x6E, 0x74,
        0x72, 0x79, 0x33, 0x36, 0x03, 0x09, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08,
        0x06, 0x00, 0x00, 0x10, 0x01, 0x08, 0x00, 0x0F, 0x02, 0x00, 0x02, 0x00, 0x07, 0x00, 0x0A,
        0x00,
    ]
}

#[test]
fn feeny_function_argument_order_deserialize() {
    let object = Program::from_bytes(&mut Cursor::new(feeny_function_argument_order_bytes()));
    assert_eq!(feeny_function_argument_order_program(), object);
}

#[test]
fn feeny_function_argument_order_serialize() {
    let mut output: Vec<u8> = Vec::new();
    feeny_function_argument_order_program().serialize(&mut output).unwrap();
    assert_eq!(feeny_function_argument_order_bytes(), output);
}

#[test]
fn feeny_function_argument_order_print() {
    let mut bytes: Vec<u8> = Vec::new();
    feeny_function_argument_order_program().pretty_print(&mut bytes);
    assert_eq!(&String::from_utf8(bytes).unwrap(), feeny_function_argument_order_source());
}

#[test]
fn feeny_function_argument_order_eval() {
    let program = feeny_function_argument_order_program();
    let mut state = State::from(&program).unwrap();
    let mut output = String::new();

    evaluate_with(&program, &mut state, &mut output).unwrap();

    assert_eq!(output, "123");
}

fn feeny_hello_world_source() -> &'static str {
    r#"Constants :
    #0: String("Hello World\n")
    #1: String("main")
    #2: Method(#1, nargs:0, nlocals:0) :
          printf #0 0
          return
    #3: Null
    #4: String("entry35")
    #5: Method(#4, nargs:0, nlocals:0) :
          call #1 0
          drop
          lit #3
          return
Globals :
    #2
Entry : #5"#
}

fn feeny_hello_world_program() -> Program {
    let code = Code::from(vec![
        /* 0 */
        OpCode::Print { format: ConstantPoolIndex::new(0), arguments: Arity::new(0) },
        /* 1 */ OpCode::Return,
        /* 2 */
        OpCode::CallFunction { name: ConstantPoolIndex::new(1), arguments: Arity::new(0) },
        /* 3 */ OpCode::Drop,
        /* 4 */ OpCode::Literal { index: ConstantPoolIndex::new(3) },
        /* 5 */ OpCode::Return,
    ]);

    let constant_pool = ConstantPool::from(vec![
        /* #0 */ ProgramObject::String("Hello World\n".to_string()),
        /* #1 */ ProgramObject::String("main".to_string()),
        /* #2 */
        ProgramObject::Method {
            name: ConstantPoolIndex::new(1),
            parameters: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(0, 2),
        },
        /* #3 */ ProgramObject::Null,
        /* #4 */ ProgramObject::String("entry35".to_string()),
        /* #5 */
        ProgramObject::Method {
            name: ConstantPoolIndex::new(4),
            parameters: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(2, 4),
        },
    ]);

    let globals = Globals::from(vec![ConstantPoolIndex::new(2)]);

    Program::from(code, constant_pool, globals, Entry::from(5)).unwrap()
}

fn feeny_hello_world_bytes() -> Vec<u8> {
    vec![
        0x06, 0x00, 0x02, 0x0C, 0x00, 0x00, 0x00, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F,
        0x72, 0x6C, 0x64, 0x0A, 0x02, 0x04, 0x00, 0x00, 0x00, 0x6D, 0x61, 0x69, 0x6E, 0x03, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0F, 0x01, 0x02,
        0x07, 0x00, 0x00, 0x00, 0x65, 0x6E, 0x74, 0x72, 0x79, 0x33, 0x35, 0x03, 0x04, 0x00, 0x00,
        0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08, 0x01, 0x00, 0x00, 0x10, 0x01, 0x03, 0x00, 0x0F,
        0x01, 0x00, 0x02, 0x00, 0x05, 0x00,
    ]
}

#[test]
fn feeny_hello_world_deserialize() {
    let object = Program::from_bytes(&mut Cursor::new(feeny_hello_world_bytes()));
    assert_eq!(feeny_hello_world_program(), object);
}

#[test]
fn feeny_hello_world_serialize() {
    let mut output: Vec<u8> = Vec::new();
    feeny_hello_world_program().serialize(&mut output).unwrap();
    assert_eq!(feeny_hello_world_bytes(), output);
}

#[test]
fn feeny_hello_world_print() {
    let mut bytes: Vec<u8> = Vec::new();
    feeny_hello_world_program().pretty_print(&mut bytes);
    assert_eq!(&String::from_utf8(bytes).unwrap(), feeny_hello_world_source());
}

#[test]
fn feeny_hello_world_eval() {
    let program = feeny_hello_world_program();
    let mut state = State::from(&program).unwrap();
    let mut output = String::new();

    evaluate_with(&program, &mut state, &mut output).unwrap();

    assert_eq!(output, "Hello World\n");
}

fn feeny_fibonacci_source() -> &'static str {
    r#"Constants :
    #0: String("conseq39")
    #1: String("end40")
    #2: Int(0)
    #3: String("eq")
    #4: String("conseq41")
    #5: String("end42")
    #6: Int(1)
    #7: String("test43")
    #8: String("loop44")
    #9: String("add")
    #10: String("sub")
    #11: Int(2)
    #12: String("ge")
    #13: Null
    #14: String("fib")
    #15: Method(#14, nargs:1, nlocals:3) :
          get local 0
          lit #2
          call slot #3 2
          branch #0
          get local 0
          lit #6
          call slot #3 2
          branch #4
          lit #6
          set local 1
          drop
          lit #6
          set local 2
          drop
          goto #7
       label #8
          get local 1
          get local 2
          call slot #9 2
          set local 3
          drop
          get local 2
          set local 1
          drop
          get local 3
          set local 2
          drop
          get local 0
          lit #6
          call slot #10 2
          set local 0
          drop
       label #7
          get local 0
          lit #11
          call slot #12 2
          branch #8
          lit #13
          drop
          get local 2
          goto #5
       label #4
          lit #6
       label #5
          goto #1
       label #0
          lit #6
       label #1
          return
    #16: String("test45")
    #17: String("loop46")
    #18: String("Fib(~) = ~\n")
    #19: Int(20)
    #20: String("lt")
    #21: String("main")
    #22: Method(#21, nargs:0, nlocals:1) :
          lit #2
          set local 0
          drop
          goto #16
       label #17
          get local 0
          get local 0
          call #14 1
          printf #18 2
          drop
          get local 0
          lit #6
          call slot #9 2
          set local 0
          drop
       label #16
          get local 0
          lit #19
          call slot #20 2
          branch #17
          lit #13
          return
    #23: String("entry47")
    #24: Method(#23, nargs:0, nlocals:0) :
          call #21 0
          drop
          lit #13
          return
Globals :
    #15
    #22
Entry : #24"#
}

fn feeny_fibonacci_expected_output() -> &'static str {
    r#"Fib(0) = 1
Fib(1) = 1
Fib(2) = 2
Fib(3) = 3
Fib(4) = 5
Fib(5) = 8
Fib(6) = 13
Fib(7) = 21
Fib(8) = 34
Fib(9) = 55
Fib(10) = 89
Fib(11) = 144
Fib(12) = 233
Fib(13) = 377
Fib(14) = 610
Fib(15) = 987
Fib(16) = 1597
Fib(17) = 2584
Fib(18) = 4181
Fib(19) = 6765
"#
}

fn feeny_fibonacci_bytes() -> Vec<u8> {
    vec![
        0x19, 0x00, 0x02, 0x08, 0x00, 0x00, 0x00, 0x63, 0x6F, 0x6E, 0x73, 0x65, 0x71, 0x33, 0x39,
        0x02, 0x05, 0x00, 0x00, 0x00, 0x65, 0x6E, 0x64, 0x34, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02, 0x02, 0x00, 0x00, 0x00, 0x65, 0x71, 0x02, 0x08, 0x00, 0x00, 0x00, 0x63, 0x6F, 0x6E,
        0x73, 0x65, 0x71, 0x34, 0x31, 0x02, 0x05, 0x00, 0x00, 0x00, 0x65, 0x6E, 0x64, 0x34, 0x32,
        0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x06, 0x00, 0x00, 0x00, 0x74, 0x65, 0x73, 0x74, 0x34,
        0x33, 0x02, 0x06, 0x00, 0x00, 0x00, 0x6C, 0x6F, 0x6F, 0x70, 0x34, 0x34, 0x02, 0x03, 0x00,
        0x00, 0x00, 0x61, 0x64, 0x64, 0x02, 0x03, 0x00, 0x00, 0x00, 0x73, 0x75, 0x62, 0x00, 0x02,
        0x00, 0x00, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00, 0x67, 0x65, 0x01, 0x02, 0x03, 0x00, 0x00,
        0x00, 0x66, 0x69, 0x62, 0x03, 0x0E, 0x00, 0x01, 0x03, 0x00, 0x31, 0x00, 0x00, 0x00, 0x0A,
        0x00, 0x00, 0x01, 0x02, 0x00, 0x07, 0x03, 0x00, 0x02, 0x0D, 0x00, 0x00, 0x0A, 0x00, 0x00,
        0x01, 0x06, 0x00, 0x07, 0x03, 0x00, 0x02, 0x0D, 0x04, 0x00, 0x01, 0x06, 0x00, 0x09, 0x01,
        0x00, 0x10, 0x01, 0x06, 0x00, 0x09, 0x02, 0x00, 0x10, 0x0E, 0x07, 0x00, 0x00, 0x08, 0x00,
        0x0A, 0x01, 0x00, 0x0A, 0x02, 0x00, 0x07, 0x09, 0x00, 0x02, 0x09, 0x03, 0x00, 0x10, 0x0A,
        0x02, 0x00, 0x09, 0x01, 0x00, 0x10, 0x0A, 0x03, 0x00, 0x09, 0x02, 0x00, 0x10, 0x0A, 0x00,
        0x00, 0x01, 0x06, 0x00, 0x07, 0x0A, 0x00, 0x02, 0x09, 0x00, 0x00, 0x10, 0x00, 0x07, 0x00,
        0x0A, 0x00, 0x00, 0x01, 0x0B, 0x00, 0x07, 0x0C, 0x00, 0x02, 0x0D, 0x08, 0x00, 0x01, 0x0D,
        0x00, 0x10, 0x0A, 0x02, 0x00, 0x0E, 0x05, 0x00, 0x00, 0x04, 0x00, 0x01, 0x06, 0x00, 0x00,
        0x05, 0x00, 0x0E, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x06, 0x00, 0x00, 0x01, 0x00, 0x0F,
        0x02, 0x06, 0x00, 0x00, 0x00, 0x74, 0x65, 0x73, 0x74, 0x34, 0x35, 0x02, 0x06, 0x00, 0x00,
        0x00, 0x6C, 0x6F, 0x6F, 0x70, 0x34, 0x36, 0x02, 0x0B, 0x00, 0x00, 0x00, 0x46, 0x69, 0x62,
        0x28, 0x7E, 0x29, 0x20, 0x3D, 0x20, 0x7E, 0x0A, 0x00, 0x14, 0x00, 0x00, 0x00, 0x02, 0x02,
        0x00, 0x00, 0x00, 0x6C, 0x74, 0x02, 0x04, 0x00, 0x00, 0x00, 0x6D, 0x61, 0x69, 0x6E, 0x03,
        0x15, 0x00, 0x00, 0x01, 0x00, 0x16, 0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x09, 0x00, 0x00,
        0x10, 0x0E, 0x10, 0x00, 0x00, 0x11, 0x00, 0x0A, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x08, 0x0E,
        0x00, 0x01, 0x02, 0x12, 0x00, 0x02, 0x10, 0x0A, 0x00, 0x00, 0x01, 0x06, 0x00, 0x07, 0x09,
        0x00, 0x02, 0x09, 0x00, 0x00, 0x10, 0x00, 0x10, 0x00, 0x0A, 0x00, 0x00, 0x01, 0x13, 0x00,
        0x07, 0x14, 0x00, 0x02, 0x0D, 0x11, 0x00, 0x01, 0x0D, 0x00, 0x0F, 0x02, 0x07, 0x00, 0x00,
        0x00, 0x65, 0x6E, 0x74, 0x72, 0x79, 0x34, 0x37, 0x03, 0x17, 0x00, 0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x08, 0x15, 0x00, 0x00, 0x10, 0x01, 0x0D, 0x00, 0x0F, 0x02, 0x00, 0x0F,
        0x00, 0x16, 0x00, 0x18, 0x00,
    ]
}

fn feeny_fibonacci_program() -> Program {
    let code = Code::from(vec![
        /* method fib: start: 0, length: 39 */
        /* 00 */
        OpCode::GetLocal { index: LocalFrameIndex::new(0) }, // arg0
        /* 01 */
        OpCode::Literal { index: ConstantPoolIndex::new(2) }, // 0
        /* 02 */
        OpCode::CallMethod {
            // 0.eq(arg0)
            name: ConstantPoolIndex::new(3),
            arguments: Arity::new(2),
        },
        /* 03 */
        OpCode::Branch { label: ConstantPoolIndex::new(0) }, // branch conseq39
        /* 04 */
        OpCode::GetLocal { index: LocalFrameIndex::new(0) }, // also x
        /* 05 */
        OpCode::Literal { index: ConstantPoolIndex::new(6) }, // 1
        /* 06 */
        OpCode::CallMethod {
            // arg0.eq(1)
            name: ConstantPoolIndex::new(3),
            arguments: Arity::new(2),
        },
        /* 07 */
        OpCode::Branch { label: ConstantPoolIndex::new(4) }, // branch conseq41
        /* 08 */
        OpCode::Literal { index: ConstantPoolIndex::new(6) }, // 1
        /* 09 */
        OpCode::SetLocal { index: LocalFrameIndex::new(1) }, // var1 = 1
        /* 10 */ OpCode::Drop,
        /* 11 */
        OpCode::Literal { index: ConstantPoolIndex::new(6) }, // 1
        /* 12 */
        OpCode::SetLocal { index: LocalFrameIndex::new(2) }, // var2 = 1
        /* 13 */ OpCode::Drop,
        /* 14 */
        OpCode::Jump { label: ConstantPoolIndex::new(7) }, // goto test43
        /* 15 */
        OpCode::Label { name: ConstantPoolIndex::new(8) }, // label loop44
        /* 16 */
        OpCode::GetLocal { index: LocalFrameIndex::new(1) }, // var1
        /* 17 */
        OpCode::GetLocal { index: LocalFrameIndex::new(2) }, // var2
        /* 18 */
        OpCode::CallMethod {
            // var1.add(var2) -> result1
            name: ConstantPoolIndex::new(9),
            arguments: Arity::new(2),
        },
        /* 19 */
        OpCode::SetLocal { index: LocalFrameIndex::new(3) }, // var3 = result1
        /* 20 */ OpCode::Drop,
        /* 21 */
        OpCode::GetLocal { index: LocalFrameIndex::new(2) }, // var2
        /* 22 */
        OpCode::SetLocal { index: LocalFrameIndex::new(1) }, // var1 = var2
        /* 23 */ OpCode::Drop,
        /* 24 */
        OpCode::GetLocal { index: LocalFrameIndex::new(3) }, // var3
        /* 25 */
        OpCode::SetLocal { index: LocalFrameIndex::new(2) }, // var2 = var3
        /* 26 */ OpCode::Drop,
        /* 27 */
        OpCode::GetLocal { index: LocalFrameIndex::new(0) }, // arg0
        /* 28 */
        OpCode::Literal { index: ConstantPoolIndex::new(6) }, // 1
        /* 29 */
        OpCode::CallMethod {
            // arg0.sub(1) -> result2
            name: ConstantPoolIndex::new(10),
            arguments: Arity::new(2),
        },
        /* 30 */
        OpCode::SetLocal { index: LocalFrameIndex::new(0) }, // arg0 = result2
        /* 31 */ OpCode::Drop,
        /* 32 */
        OpCode::Label { name: ConstantPoolIndex::new(7) }, // label test43
        /* 33 */
        OpCode::GetLocal { index: LocalFrameIndex::new(0) }, // arg0
        /* 34 */
        OpCode::Literal { index: ConstantPoolIndex::new(11) }, // 2
        /* 35 */
        OpCode::CallMethod {
            // arg0.ge(2) -> result3
            name: ConstantPoolIndex::new(12),
            arguments: Arity::new(2),
        },
        /* 36 */
        OpCode::Branch { label: ConstantPoolIndex::new(8) }, // loop44
        /* 37 */
        OpCode::Literal { index: ConstantPoolIndex::new(13) }, // null
        /* 38 */ OpCode::Drop,
        /* 39 */
        OpCode::GetLocal { index: LocalFrameIndex::new(2) }, // arg2 (return arg2)
        /* 40 */
        OpCode::Jump { label: ConstantPoolIndex::new(5) }, // goto end42
        /* 41 */
        OpCode::Label { name: ConstantPoolIndex::new(4) }, // label conseq41
        /* 42 */
        OpCode::Literal { index: ConstantPoolIndex::new(6) }, // 1 (return 1)
        /* 43 */
        OpCode::Label { name: ConstantPoolIndex::new(5) }, // label end42
        /* 44 */
        OpCode::Jump { label: ConstantPoolIndex::new(1) }, // goto end40
        /* 45 */
        OpCode::Label { name: ConstantPoolIndex::new(0) }, // label conseq39
        /* 46 */
        OpCode::Literal { index: ConstantPoolIndex::new(6) }, // 1 (return 1)
        /* 47 */
        OpCode::Label { name: ConstantPoolIndex::new(1) }, // label end40
        /* 48 */ OpCode::Return,
        /* method main: start: 49, length: 22 */
        /* 49 */
        OpCode::Literal { index: ConstantPoolIndex::new(2) }, // 0
        /* 50 */
        OpCode::SetLocal { index: LocalFrameIndex::new(0) }, // var0 = 0
        /* 51 */ OpCode::Drop,
        /* 52 */
        OpCode::Jump { label: ConstantPoolIndex::new(16) }, // goto loop45
        /* 53 */
        OpCode::Label { name: ConstantPoolIndex::new(17) }, // label loop46
        /* 54 */
        OpCode::GetLocal { index: LocalFrameIndex::new(0) }, // var0
        /* 55 */
        OpCode::GetLocal { index: LocalFrameIndex::new(0) }, // var0 ... again?
        /* 56 */
        OpCode::CallFunction {
            // fib(var0) -> result1
            name: ConstantPoolIndex::new(14),
            arguments: Arity::new(1),
        },
        /* 57 */
        OpCode::Print {
            // printf "Fib(~) = ~\n" var0 result1
            format: ConstantPoolIndex::new(18),
            arguments: Arity::new(2),
        },
        /* 58 */ OpCode::Drop,
        /* 59 */
        OpCode::GetLocal { index: LocalFrameIndex::new(0) }, // var0
        /* 60 */
        OpCode::Literal { index: ConstantPoolIndex::new(6) }, // 1
        /* 61 */
        OpCode::CallMethod {
            // var0.add(1) -> result2
            name: ConstantPoolIndex::new(9),
            arguments: Arity::new(2),
        },
        /* 62 */
        OpCode::SetLocal { index: LocalFrameIndex::new(0) }, // var0 = result2
        /* 63 */ OpCode::Drop,
        /* 64 */
        OpCode::Label { name: ConstantPoolIndex::new(16) }, // label test45
        /* 65 */
        OpCode::GetLocal { index: LocalFrameIndex::new(0) }, // var0
        /* 66 */
        OpCode::Literal { index: ConstantPoolIndex::new(19) }, // 20
        /* 67 */
        OpCode::CallMethod {
            // var0.lt(20) -> result3
            name: ConstantPoolIndex::new(20),
            arguments: Arity::new(2),
        },
        /* 68 */
        OpCode::Branch { label: ConstantPoolIndex::new(17) }, // branch loop46
        /* 69 */
        OpCode::Literal { index: ConstantPoolIndex::new(13) }, // null
        /* 70 */ OpCode::Return,
        /* method entry: start: 71, length: 4 */
        /* 71 */
        OpCode::CallFunction {
            // main() -> result0
            name: ConstantPoolIndex::new(21),
            arguments: Arity::new(0),
        },
        /* 72 */ OpCode::Drop,
        /* 73 */
        OpCode::Literal { index: ConstantPoolIndex::new(13) }, // null
        /* 74 */ OpCode::Return,
    ]);

    let constants = ConstantPool::from(vec![
        /* #0  0x00 */ ProgramObject::String("conseq39".to_string()),
        /* #1  0x01 */ ProgramObject::String("end40".to_string()),
        /* #2  0x02 */ ProgramObject::Integer(0),
        /* #3  0x03 */ ProgramObject::String("eq".to_string()),
        /* #4  0x04 */ ProgramObject::String("conseq41".to_string()),
        /* #5  0x05 */ ProgramObject::String("end42".to_string()),
        /* #6  0x06 */ ProgramObject::Integer(1),
        /* #7  0x07 */ ProgramObject::String("test43".to_string()),
        /* #8  0x08 */ ProgramObject::String("loop44".to_string()),
        /* #9  0x09 */ ProgramObject::String("add".to_string()),
        /* #10 0x0A */ ProgramObject::String("sub".to_string()),
        /* #11 0x0B */ ProgramObject::Integer(2),
        /* #12 0x0C */ ProgramObject::String("ge".to_string()),
        /* #13 0x0D */ ProgramObject::Null,
        /* #14 0x0E */ ProgramObject::String("fib".to_string()),
        /* #15 0x0F */
        ProgramObject::Method {
            // fib
            name: ConstantPoolIndex::new(14),
            parameters: Arity::new(1),
            locals: Size::new(3),
            code: AddressRange::from(0, 49),
        },
        /* #16 0x10 */ ProgramObject::String("test45".to_string()),
        /* #17 0x11 */ ProgramObject::String("loop46".to_string()),
        /* #18 0x11 */ ProgramObject::String("Fib(~) = ~\n".to_string()),
        /* #19 0x12 */ ProgramObject::Integer(20),
        /* #20 0x13 */ ProgramObject::String("lt".to_string()),
        /* #21 0x14 */ ProgramObject::String("main".to_string()),
        /* #22 0x15 */
        ProgramObject::Method {
            // main
            name: ConstantPoolIndex::new(21),
            parameters: Arity::new(0),
            locals: Size::new(1),
            code: AddressRange::from(49, 22),
        },
        /* #23 0x15 */ ProgramObject::String("entry47".to_string()),
        /* #24 0x16 */
        ProgramObject::Method {
            // entry47
            name: ConstantPoolIndex::new(23),
            parameters: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(71, 4),
        },
    ]);

    let globals = Globals::from(vec![ConstantPoolIndex::new(15), ConstantPoolIndex::new(22)]);

    let entry = Entry::from(24);

    Program::from(code, constants, globals, entry).unwrap()
}

#[test]
fn feeny_fibonacci_deserialize() {
    let object = Program::from_bytes(&mut Cursor::new(feeny_fibonacci_bytes()));
    assert_eq!(feeny_fibonacci_program(), object);
}

#[test]
fn feeny_fibonacci_serialize() {
    let mut output: Vec<u8> = Vec::new();
    feeny_fibonacci_program().serialize(&mut output).unwrap();
    assert_eq!(feeny_fibonacci_bytes(), output);
}

#[test]
fn feeny_fibonacci_print() {
    let mut bytes: Vec<u8> = Vec::new();
    feeny_fibonacci_program().pretty_print(&mut bytes);
    assert_eq!(&String::from_utf8(bytes).unwrap(), feeny_fibonacci_source());
}

#[test]
fn feeny_fibonacci_eval() {
    let program = feeny_fibonacci_program();
    let mut state = State::from(&program).unwrap();
    let mut output = String::new();

    let mut source: Vec<u8> = Vec::new();
    program.pretty_print(&mut source);
    println!("{}", String::from_utf8(source).unwrap());

    evaluate_with(&program, &mut state, &mut output).unwrap();

    assert_eq!(output, feeny_fibonacci_expected_output());
}
