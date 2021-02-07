use std::io::Cursor;

use crate::bytecode::bytecode::*;
use crate::bytecode::types::*;
use crate::bytecode::program::*;
use crate::bytecode::objects::*;
use crate::bytecode::interpreter::*;
use crate::bytecode::serializable::*;
use crate::bytecode::debug::*;

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
Entry : #5"#}

fn feeny_hello_world_program() -> Program {
    let code = Code::from(vec!(
        /* 0 */ OpCode::Print { format: ConstantPoolIndex::new(0), arguments: Arity::new(0) },
        /* 1 */ OpCode::Return,
        /* 2 */ OpCode::CallFunction { name: ConstantPoolIndex::new(1), arguments: Arity::new(0) },
        /* 3 */ OpCode::Drop,
        /* 4 */ OpCode::Literal { index: ConstantPoolIndex::new(3) },
        /* 5 */ OpCode::Return,
    ));

    let constants = vec!(
        /* #0 */ ProgramObject::String("Hello World\n".to_string()),
        /* #1 */ ProgramObject::String("main".to_string()),
        /* #2 */ ProgramObject::Method {
            name: ConstantPoolIndex::new(1),
            arguments: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(0, 2),
        },
        /* #3 */ ProgramObject::Null,
        /* #4 */ ProgramObject::String("entry35".to_string()),
        /* #5 */ ProgramObject::Method {
            name: ConstantPoolIndex::new(4),
            arguments: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(2, 4),
        },
    );

    let globals = vec!(ConstantPoolIndex::new(2));
    let entry = ConstantPoolIndex::new(5);

    Program::new(code, constants, globals, entry)
}

fn feeny_hello_world_bytes() -> Vec<u8> {
    vec!(
        0x06, 0x00, 0x02, 0x0C, 0x00, 0x00, 0x00, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57,
        0x6F, 0x72, 0x6C, 0x64, 0x0A, 0x02, 0x04, 0x00, 0x00, 0x00, 0x6D, 0x61, 0x69, 0x6E,
        0x03, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        0x0F, 0x01, 0x02, 0x07, 0x00, 0x00, 0x00, 0x65, 0x6E, 0x74, 0x72, 0x79, 0x33, 0x35,
        0x03, 0x04, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08, 0x01, 0x00, 0x00,
        0x10, 0x01, 0x03, 0x00, 0x0F, 0x01, 0x00, 0x02, 0x00, 0x05, 0x00,
    )
}

#[test] fn feeny_hello_world_deserialize() {
    let object = Program::from_bytes(&mut Cursor::new(feeny_hello_world_bytes()));
    assert_eq!(feeny_hello_world_program(), object);
}

#[test] fn feeny_hello_world_serialize() {
    let mut output: Vec<u8> = Vec::new();
    feeny_hello_world_program().serialize(&mut output).unwrap();
    assert_eq!(feeny_hello_world_bytes(), output);
}

#[test] fn feeny_hello_world_print() {
    let mut bytes: Vec<u8> = Vec::new();
    feeny_hello_world_program().pretty_print(&mut bytes);
    assert_eq!(&String::from_utf8(bytes).unwrap(), feeny_hello_world_source());
}

#[test] fn feeny_hello_world_eval() {
    let program = feeny_hello_world_program();
    let mut state = State::from(&program);
    let mut output = String::new();

    loop {
        interpret(&mut state, &mut output, &program);
        if let None = state.instruction_pointer() {
            break;
        }
    }

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
Entry : #24"#}

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
    vec!(
        0x19, 0x00, 0x02, 0x08, 0x00, 0x00, 0x00, 0x63, 0x6F, 0x6E, 0x73, 0x65, 0x71, 0x33,
        0x39, 0x02, 0x05, 0x00, 0x00, 0x00, 0x65, 0x6E, 0x64, 0x34, 0x30, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00, 0x65, 0x71, 0x02, 0x08, 0x00, 0x00, 0x00,
        0x63, 0x6F, 0x6E, 0x73, 0x65, 0x71, 0x34, 0x31, 0x02, 0x05, 0x00, 0x00, 0x00, 0x65,
        0x6E, 0x64, 0x34, 0x32, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x06, 0x00, 0x00, 0x00,
        0x74, 0x65, 0x73, 0x74, 0x34, 0x33, 0x02, 0x06, 0x00, 0x00, 0x00, 0x6C, 0x6F, 0x6F,
        0x70, 0x34, 0x34, 0x02, 0x03, 0x00, 0x00, 0x00, 0x61, 0x64, 0x64, 0x02, 0x03, 0x00,
        0x00, 0x00, 0x73, 0x75, 0x62, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x02, 0x00, 0x00,
        0x00, 0x67, 0x65, 0x01, 0x02, 0x03, 0x00, 0x00, 0x00, 0x66, 0x69, 0x62, 0x03, 0x0E,
        0x00, 0x01, 0x03, 0x00, 0x31, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x01, 0x02, 0x00,
        0x07, 0x03, 0x00, 0x02, 0x0D, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x01, 0x06, 0x00, 0x07,
        0x03, 0x00, 0x02, 0x0D, 0x04, 0x00, 0x01, 0x06, 0x00, 0x09, 0x01, 0x00, 0x10, 0x01,
        0x06, 0x00, 0x09, 0x02, 0x00, 0x10, 0x0E, 0x07, 0x00, 0x00, 0x08, 0x00, 0x0A, 0x01,
        0x00, 0x0A, 0x02, 0x00, 0x07, 0x09, 0x00, 0x02, 0x09, 0x03, 0x00, 0x10, 0x0A, 0x02,
        0x00, 0x09, 0x01, 0x00, 0x10, 0x0A, 0x03, 0x00, 0x09, 0x02, 0x00, 0x10, 0x0A, 0x00,
        0x00, 0x01, 0x06, 0x00, 0x07, 0x0A, 0x00, 0x02, 0x09, 0x00, 0x00, 0x10, 0x00, 0x07,
        0x00, 0x0A, 0x00, 0x00, 0x01, 0x0B, 0x00, 0x07, 0x0C, 0x00, 0x02, 0x0D, 0x08, 0x00,
        0x01, 0x0D, 0x00, 0x10, 0x0A, 0x02, 0x00, 0x0E, 0x05, 0x00, 0x00, 0x04, 0x00, 0x01,
        0x06, 0x00, 0x00, 0x05, 0x00, 0x0E, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x06, 0x00,
        0x00, 0x01, 0x00, 0x0F, 0x02, 0x06, 0x00, 0x00, 0x00, 0x74, 0x65, 0x73, 0x74, 0x34,
        0x35, 0x02, 0x06, 0x00, 0x00, 0x00, 0x6C, 0x6F, 0x6F, 0x70, 0x34, 0x36, 0x02, 0x0B,
        0x00, 0x00, 0x00, 0x46, 0x69, 0x62, 0x28, 0x7E, 0x29, 0x20, 0x3D, 0x20, 0x7E, 0x0A,
        0x00, 0x14, 0x00, 0x00, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00, 0x6C, 0x74, 0x02, 0x04,
        0x00, 0x00, 0x00, 0x6D, 0x61, 0x69, 0x6E, 0x03, 0x15, 0x00, 0x00, 0x01, 0x00, 0x16,
        0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x09, 0x00, 0x00, 0x10, 0x0E, 0x10, 0x00, 0x00,
        0x11, 0x00, 0x0A, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x08, 0x0E, 0x00, 0x01, 0x02, 0x12,
        0x00, 0x02, 0x10, 0x0A, 0x00, 0x00, 0x01, 0x06, 0x00, 0x07, 0x09, 0x00, 0x02, 0x09,
        0x00, 0x00, 0x10, 0x00, 0x10, 0x00, 0x0A, 0x00, 0x00, 0x01, 0x13, 0x00, 0x07, 0x14,
        0x00, 0x02, 0x0D, 0x11, 0x00, 0x01, 0x0D, 0x00, 0x0F, 0x02, 0x07, 0x00, 0x00, 0x00,
        0x65, 0x6E, 0x74, 0x72, 0x79, 0x34, 0x37, 0x03, 0x17, 0x00, 0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x08, 0x15, 0x00, 0x00, 0x10, 0x01, 0x0D, 0x00, 0x0F, 0x02, 0x00,
        0x0F, 0x00, 0x16, 0x00, 0x18, 0x00,
    )
}

fn feeny_fibonacci_program () -> Program {
    let code = Code::from(vec!(
        /* method fib: start: 0, length: 39 */
        /* 00 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },   // arg0
        /* 01 */ OpCode::Literal { index: ConstantPoolIndex::new(2) },  // 0
        /* 02 */ OpCode::CallMethod {                                   // 0.eq(arg0)
            name: ConstantPoolIndex::new(3),
            arguments: Arity::new(2) },
        /* 03 */ OpCode::Branch { label: ConstantPoolIndex::new(0) },   // branch conseq39
        /* 04 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },   // also x
        /* 05 */ OpCode::Literal { index: ConstantPoolIndex::new(6) },  // 1
        /* 06 */ OpCode::CallMethod {                                   // arg0.eq(1)
            name: ConstantPoolIndex::new(3),
            arguments: Arity::new(2) },
        /* 07 */ OpCode::Branch { label: ConstantPoolIndex::new(4) },   // branch conseq41
        /* 08 */ OpCode::Literal { index: ConstantPoolIndex::new(6) },  // 1
        /* 09 */ OpCode::SetLocal { index: LocalFrameIndex::new(1) },   // var1 = 1
        /* 10 */ OpCode::Drop,
        /* 11 */ OpCode::Literal { index: ConstantPoolIndex::new(6) },  // 1
        /* 12 */ OpCode::SetLocal { index: LocalFrameIndex::new(2) },   // var2 = 1
        /* 13 */ OpCode::Drop,
        /* 14 */ OpCode::Jump { label: ConstantPoolIndex::new(7) },     // goto test43

        /* 15 */ OpCode::Label { name: ConstantPoolIndex::new(8) },     // label loop44
        /* 16 */ OpCode::GetLocal { index: LocalFrameIndex::new(1) },   // var1
        /* 17 */ OpCode::GetLocal { index: LocalFrameIndex::new(2) },   // var2
        /* 18 */ OpCode::CallMethod {                                   // var1.add(var2) -> result1
            name: ConstantPoolIndex::new(9),
            arguments: Arity::new(2) },
        /* 19 */ OpCode::SetLocal { index: LocalFrameIndex::new(3) },   // var3 = result1
        /* 20 */ OpCode::Drop,
        /* 21 */ OpCode::GetLocal { index: LocalFrameIndex::new(2) },   // var2
        /* 22 */ OpCode::SetLocal { index: LocalFrameIndex::new(1) },   // var1 = var2
        /* 23 */ OpCode::Drop,
        /* 24 */ OpCode::GetLocal { index: LocalFrameIndex::new(3) },   // var3
        /* 25 */ OpCode::SetLocal { index: LocalFrameIndex::new(2) },   // var2 = var3
        /* 26 */ OpCode::Drop,
        /* 27 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },   // arg0
        /* 28 */ OpCode::Literal { index: ConstantPoolIndex::new(6) },  // 1
        /* 29 */ OpCode::CallMethod {                                   // arg0.sub(1) -> result2
            name: ConstantPoolIndex::new(10),
            arguments: Arity::new(2) },
        /* 30 */ OpCode::SetLocal { index: LocalFrameIndex::new(0) },   // arg0 = result2
        /* 31 */ OpCode::Drop,
        /* 32 */ OpCode::Label { name: ConstantPoolIndex::new(7) },     // label test43
        /* 33 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },   // arg0
        /* 34 */ OpCode::Literal { index: ConstantPoolIndex::new(11) }, // 2
        /* 35 */ OpCode::CallMethod {                                   // arg0.ge(2) -> result3
            name: ConstantPoolIndex::new(12),
            arguments: Arity::new(2) },
        /* 36 */ OpCode::Branch { label: ConstantPoolIndex::new(8) },   // loop44
        /* 37 */ OpCode::Literal { index: ConstantPoolIndex::new(13) }, // null
        /* 38 */ OpCode::Drop,
        /* 39 */ OpCode::GetLocal { index: LocalFrameIndex::new(2) },   // arg2 (return arg2)
        /* 40 */ OpCode::Jump { label: ConstantPoolIndex::new(5) },     // goto end42
        /* 41 */ OpCode::Label { name: ConstantPoolIndex::new(4) },     // label conseq41
        /* 42 */ OpCode::Literal { index: ConstantPoolIndex::new(6) },  // 1 (return 1)
        /* 43 */ OpCode::Label { name: ConstantPoolIndex::new(5) },     // label end42
        /* 44 */ OpCode::Jump { label: ConstantPoolIndex::new(1) },     // goto end40
        /* 45 */ OpCode::Label { name: ConstantPoolIndex::new(0) },     // label conseq39
        /* 46 */ OpCode::Literal { index: ConstantPoolIndex::new(6) },  // 1 (return 1)
        /* 47 */ OpCode::Label { name: ConstantPoolIndex::new(1) },     // label end40
        /* 48 */ OpCode::Return,

        /* method main: start: 49, length: 22 */
        /* 49 */ OpCode::Literal { index: ConstantPoolIndex::new(2) },  // 0
        /* 50 */ OpCode::SetLocal { index: LocalFrameIndex::new(0) },   // var0 = 0
        /* 51 */ OpCode::Drop,
        /* 52 */ OpCode::Jump { label: ConstantPoolIndex::new(16) },    // goto loop45
        /* 53 */ OpCode::Label { name: ConstantPoolIndex::new(17) },    // label loop46
        /* 54 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },   // var0
        /* 55 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },   // var0 ... again?
        /* 56 */ OpCode::CallFunction {                                 // fib(var0) -> result1
            name: ConstantPoolIndex::new(14),
            arguments: Arity::new(1) },
        /* 57 */ OpCode::Print {                                        // printf "Fib(~) = ~\n" var0 result1
            format: ConstantPoolIndex::new(18),
            arguments: Arity::new(2) },
        /* 58 */ OpCode::Drop,
        /* 59 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },    // var0
        /* 60 */ OpCode::Literal { index: ConstantPoolIndex::new(6) },   // 1
        /* 61 */ OpCode::CallMethod {                                    // var0.add(1) -> result2
            name: ConstantPoolIndex::new(9),
            arguments: Arity::new(2) },
        /* 62 */ OpCode::SetLocal { index: LocalFrameIndex::new(0) },    // var0 = result2
        /* 63 */ OpCode::Drop,
        /* 64 */ OpCode::Label { name: ConstantPoolIndex::new(16) },     // label test45
        /* 65 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },    // var0
        /* 66 */ OpCode::Literal { index: ConstantPoolIndex::new(19) },  // 20
        /* 67 */ OpCode::CallMethod {                                    // var0.lt(20) -> result3
            name: ConstantPoolIndex::new(20),
            arguments: Arity::new(2) },
        /* 68 */ OpCode::Branch { label: ConstantPoolIndex::new(17) },   // branch loop46
        /* 69 */ OpCode::Literal { index: ConstantPoolIndex::new(13) },  // null
        /* 70 */ OpCode::Return,

        /* method entry: start: 71, length: 4 */
        /* 71 */ OpCode::CallFunction {                                 // main() -> result0
            name: ConstantPoolIndex::new(21),
            arguments: Arity::new(0) },
        /* 72 */ OpCode::Drop,
        /* 73 */ OpCode::Literal { index: ConstantPoolIndex::new(13) }, // null
        /* 74 */ OpCode::Return
    ));

    let constants = vec!(
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
        /* #15 0x0F */ ProgramObject::Method {                             // fib
            name: ConstantPoolIndex::new(14),
            arguments: Arity::new(1),
            locals: Size::new(3),
            code: AddressRange::from(0, 49),
        },
        /* #16 0x10 */ ProgramObject::String("test45".to_string()),
        /* #17 0x11 */ ProgramObject::String("loop46".to_string()),
        /* #18 0x11 */ ProgramObject::String("Fib(~) = ~\n".to_string()),
        /* #19 0x12 */ ProgramObject::Integer(20),
        /* #20 0x13 */ ProgramObject::String("lt".to_string()),
        /* #21 0x14 */ ProgramObject::String("main".to_string()),
        /* #22 0x15 */ ProgramObject::Method {                             // main
            name: ConstantPoolIndex::new(21),
            arguments: Arity::new(0),
            locals: Size::new(1),
            code: AddressRange::from(49, 22),
        },
        /* #23 0x15 */ ProgramObject::String("entry47".to_string()),
        /* #24 0x16 */ ProgramObject::Method {                             // entry47
            name: ConstantPoolIndex::new(23),
            arguments: Arity::new(0),
            locals: Size::new(0),
            code: AddressRange::from(71,4),
        }
    );

    let globals = vec!(
        ConstantPoolIndex::new(15),
        ConstantPoolIndex::new(22)
    );

    let entry = ConstantPoolIndex::new(24);

    Program::new (code, constants, globals, entry)
}

#[test] fn feeny_fibonacci_deserialize() {
    let object = Program::from_bytes(&mut Cursor::new(feeny_fibonacci_bytes()));
    assert_eq!(feeny_fibonacci_program(), object);
}

#[test] fn feeny_fibonacci_serialize() {
    let mut output: Vec<u8> = Vec::new();
    feeny_fibonacci_program().serialize(&mut output).unwrap();
    assert_eq!(feeny_fibonacci_bytes(), output);
}

#[test] fn feeny_fibonacci_print() {
    let mut bytes: Vec<u8> = Vec::new();
    feeny_fibonacci_program().pretty_print(&mut bytes);
    assert_eq!(&String::from_utf8(bytes).unwrap(), feeny_fibonacci_source());
}

#[test] fn feeny_fibonacci_eval() {
    let program = feeny_fibonacci_program();
    let mut state = State::from(&program);
    let mut output = String::new();


    let mut source:Vec<u8> = Vec::new();
    program.pretty_print(&mut source);
    println!("{}", String::from_utf8(source).unwrap());

    loop {
        match state.instruction_pointer() {
            Some(address) => println!("{:?} => {:?}", address, program.get_opcode(address)),
            _ => println!("None => ..."),
        }
        println!("stack before: {:?}", state.operands);
        println!("frame before: {:?}", state.frames.last());
        interpret(&mut state, &mut output, &program);
        if let None = state.instruction_pointer() {
            break;
        }
        println!("stack after:  {:?}", state.operands);
        println!("frame after:  {:?}", state.frames.last());
        println!();
    }

    assert_eq!(output, feeny_fibonacci_expected_output());
}