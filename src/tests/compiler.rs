use crate::parser::*;

use crate::bytecode::bytecode::*;
use crate::bytecode::program::*;

use crate::bytecode::compiler::*;

#[test] fn number () {
    let ast = AST::Integer(1);

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Literal { index: ConstantPoolIndex::new(0) }
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec!(
        /* 0 */ ProgramObject::Integer(1)
    ));

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn some_more_numbers () {
    let asts = vec!(AST::Integer(1), AST::Integer(42), AST::Integer(0), AST::Integer(42));

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];


    for ast in asts {
        let ext_ast = ast.split_into_compilation_units();
        ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();
    }

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Literal { index: ConstantPoolIndex::new(0) },
        /* 1 */ OpCode::Literal { index: ConstantPoolIndex::new(1) },
        /* 2 */ OpCode::Literal { index: ConstantPoolIndex::new(2) },
        /* 3 */ OpCode::Literal { index: ConstantPoolIndex::new(1) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::Integer(1),
        /* 1 */ ProgramObject::Integer(42),
        /* 2 */ ProgramObject::Integer(0),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn boolean () {
    let ast = AST::Boolean(true);

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    println!("ext_ast {:?}", ext_ast);
    println!("ext_ast {:?}", program);

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Literal { index: ConstantPoolIndex::new(0) }
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::Boolean(true)
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn unit () {
    let ast = AST::Null;

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Literal { index: ConstantPoolIndex::new(0) }
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::Null
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn local_definition () {
    let ast = AST::Variable { name: Identifier::from("x"),
        value: Box::new(AST::Integer(1)) };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec!["x".to_owned()]);

    let expected_code = Code::from(vec!(
        OpCode::Literal { index: ConstantPoolIndex::new(0) },    // value
        OpCode::SetLocal { index: LocalFrameIndex::new(0) }
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::Integer(1)
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn global_definition () {
    let ast = AST::Variable { name: Identifier::from("x"),
        value: Box::new(AST::Integer(1)) };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::Top;
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::Top;

    let expected_code = Code::from(vec!(
        OpCode::Literal { index: ConstantPoolIndex::new(0) },    // value
        OpCode::SetGlobal { name: ConstantPoolIndex::new(1) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::from_i32(1),
        /* 1 */ ProgramObject::from_str("x"),
        /* 2 */ ProgramObject::Slot { name: ConstantPoolIndex::from_usize(1) },
    ]);

    let expected_globals = Globals::from(vec![ConstantPoolIndex::from_usize(2)]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn local_access_x () {
    let ast = AST::AccessVariable { name: Identifier::from("x") };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec!["x".to_string(), "y".to_string()]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec!["x".to_string(), "y".to_string()]);

    let expected_code = Code::from(vec!(
        OpCode::GetLocal { index: LocalFrameIndex::new(0) }
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn local_access_y () {
    let ast = AST::AccessVariable { name: Identifier::from("y") };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec!["x".to_string(), "y".to_string()]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec!["x".to_string(), "y".to_string()]);

    let expected_code = Code::from(vec!(
        OpCode::GetLocal { index: LocalFrameIndex::new(1) }
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn global_access () {
    let ast = AST::AccessVariable { name: Identifier::from("x") };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        OpCode::GetGlobal { name: ConstantPoolIndex::new(0) }
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        ProgramObject::from_str("x")
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn global_access_from_elsewhere () {
    let ast = AST::AccessVariable { name: Identifier::from("z") };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::from_locals(vec!["x".to_string()]);
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::from_locals(vec!["x".to_string()]);
    let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        OpCode::GetGlobal { name: ConstantPoolIndex::new(0) }
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::from_str("z"),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn loop_de_loop () {
    let ast = AST::Loop { condition: Box::new(AST::Boolean(false)), body: Box::new(AST::Null) };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Jump { label: ConstantPoolIndex::new(1) },
        /* 1 */ OpCode::Label { name: ConstantPoolIndex::new(0) },
        /* 2 */ OpCode::Literal { index: ConstantPoolIndex::new(2) },
        /* 3 */ OpCode::Drop,
        /* 4 */ OpCode::Label { name: ConstantPoolIndex::new(1) },
        /* 5 */ OpCode::Literal { index: ConstantPoolIndex::new(3) },
        /* 6 */ OpCode::Branch { label: ConstantPoolIndex::new(0) },
        /* 7 */ OpCode::Literal { index: ConstantPoolIndex::new(2) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::String("loop:body:0".to_string()),
        /* 1 */ ProgramObject::String("loop:condition:0".to_string()),
        /* 2 */ ProgramObject::Null,
        /* 3 */ ProgramObject::Boolean(false),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn conditional () {
    let ast = AST::Conditional {
        condition: Box::new(AST::Boolean(true)),
        consequent: Box::new(AST::Integer(1)),
        alternative: Box::new(AST::Integer(-1))
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Literal { index: ConstantPoolIndex::new(2) },
        /* 1 */ OpCode::Branch { label: ConstantPoolIndex::new(0) },
        /* 2 */ OpCode::Literal { index: ConstantPoolIndex::new(3) },
        /* 3 */ OpCode::Jump { label: ConstantPoolIndex::new(1) },
        /* 4 */ OpCode::Label { name: ConstantPoolIndex::new(0) },
        /* 5 */ OpCode::Literal { index: ConstantPoolIndex::new(4) },
        /* 6 */ OpCode::Label { name: ConstantPoolIndex::new(1) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::String("if:consequent:0".to_string()),
        /* 1 */ ProgramObject::String("if:end:0".to_string()),
        /* 2 */ ProgramObject::Boolean(true),
        /* 3 */ ProgramObject::Integer(-1),
        /* 4 */ ProgramObject::Integer(1),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn array_definition_simple_test() {
    let ast = AST::Array {
        value: Box::new(AST::Null),
        size: Box::new(AST::Integer(10)),
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Literal { index: ConstantPoolIndex::new(0) },
        /* 1 */ OpCode::Literal { index: ConstantPoolIndex::new(1) },
        /* 2 */ OpCode::Array,
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::Integer(10),
        /* 1 */ ProgramObject::Null,
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn array_definition_complex_test() { // FIXME test is wrong
    let ast = AST::Array {
        size: Box::new(AST::Integer(10)),
        value: Box::new(AST::CallFunction {
            name: Identifier::from("f"),
            arguments: vec!()
        }),
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals_at(
        vec![
            "::size".to_string(),
            "::array".to_string(),
            "::i".to_string()
        ], 0);

    let expected_code = Code::from(vec!(
        OpCode::Literal { index: ConstantPoolIndex::new(0) },                                  // 10
        OpCode::SetLocal { index: LocalFrameIndex::new(0) },                                   // size = 10
        OpCode::Drop,
        OpCode::GetLocal { index: LocalFrameIndex::new(0) },                                   // ?size = 10
        OpCode::Literal { index: ConstantPoolIndex::new(1) },                                  // null
        OpCode::Array,                                                                               // array(size = 10, null)
        OpCode::SetLocal { index: LocalFrameIndex::new(1) },                                   // arr = array(size = 10, null)
        OpCode::Drop,

        OpCode::Literal { index: ConstantPoolIndex::new(2) },                                  // 0
        OpCode::SetLocal { index: LocalFrameIndex::new(2) },                                   // i = 0
        OpCode::Drop,
        OpCode::Jump { label: ConstantPoolIndex::new(4) },                                     // jump to loop_condition_0

        OpCode::Label { name: ConstantPoolIndex::new(3) },                                     // label loop_body_0:
        OpCode::GetLocal { index: LocalFrameIndex::new(1) },                                   // arr
        OpCode::GetLocal { index: LocalFrameIndex::new(2) },                                   // i
        OpCode::CallFunction { name: ConstantPoolIndex::new(5),arguments: Arity::new(0)},// call f() -> result on stack
        OpCode::CallMethod { name: ConstantPoolIndex::new(6), arguments: Arity::new(3) },// call arr.set(i, result of f())
        OpCode::Drop,
        OpCode::GetLocal { index: LocalFrameIndex::new(2) },                                   // i
        OpCode::Literal { index: ConstantPoolIndex::new(8) },                                  // 1
        OpCode::CallMethod { name: ConstantPoolIndex::new(7), arguments: Arity::new(2) },// i + 1
        OpCode::SetLocal { index: LocalFrameIndex::new(2) },                                   // i = i + 1
        OpCode::Drop,

        OpCode::Label { name: ConstantPoolIndex::new(4) },                                     // label loop_condition_0:
        OpCode::GetLocal { index: LocalFrameIndex::new(2) },                                   // i
        OpCode::GetLocal { index: LocalFrameIndex::new(0) },                                   // size
        OpCode::CallMethod { name: ConstantPoolIndex::new(9), arguments: Arity::new(2) },// i < size
        OpCode::Branch { label: ConstantPoolIndex::new(3) },                                   // conditional jump to loop_body_0
        OpCode::GetLocal { index: LocalFrameIndex::new(1) },                                   // arr
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::from_i32(10),
        /* 1 */ ProgramObject::Null,
        /* 2 */ ProgramObject::Integer(0),
        /* 3 */ ProgramObject::from_str("loop:body:0"),
        /* 4 */ ProgramObject::from_str("loop:condition:0"),
        /* 5 */ ProgramObject::from_str("f"),
        /* 6 */ ProgramObject::from_str("set"),
        /* 7 */ ProgramObject::from_str("+"),
        /* 8 */ ProgramObject::from_i32(1),
        /* 9 */ ProgramObject::from_str("<"),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    // println!("Constants:");
    // for (i, constant) in program.constants().iter().enumerate() {
    //     println!("#{}: {:?}", i, constant);
    // }

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn array_access_test() {
    let ast = AST::AccessArray {
        array: Box::new(AST::AccessVariable { name: Identifier("x".to_string()) }),
        index: Box::new(AST::Integer(1)),
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec!["x".to_string()]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec!["x".to_string()]);

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },
        /* 1 */ OpCode::Literal { index: ConstantPoolIndex::new(0) },
        /* 2 */ OpCode::CallMethod { name: ConstantPoolIndex::new(1), arguments: Arity::new(2) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::Integer(1),
        /* 1 */ ProgramObject::String("get".to_string()),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn array_mutation_test() {
    let ast = AST::AssignArray {
        array: Box::new(AST::AccessVariable { name: Identifier("x".to_string()) }),
        index: Box::new(AST::Integer(1)),
        value: Box::new(AST::Integer(42)),
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec!["x".to_string()]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec!["x".to_string()]);

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },
        /* 1 */ OpCode::Literal { index: ConstantPoolIndex::new(0) },
        /* 2 */ OpCode::Literal { index: ConstantPoolIndex::new(1) },
        /* 3 */ OpCode::CallMethod { name: ConstantPoolIndex::new(2), arguments: Arity::new(3) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::Integer(1),
        /* 1 */ ProgramObject::Integer(42),
        /* 2 */ ProgramObject::String("set".to_string()),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn print_test () {
    let ast = AST::Print {
        format: "~ + ~".to_string(),
        arguments: vec!(
            Box::new(AST::Integer(2)),
            Box::new(AST::Integer(5)),
        ),
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Literal { index:  ConstantPoolIndex::new(1) },
        /* 1 */ OpCode::Literal { index:  ConstantPoolIndex::new(2) },
        /* 2 */ OpCode::Print   { format: ConstantPoolIndex::new(0), arguments: Arity::new(2) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::String("~ + ~".to_string()),
        /* 1 */ ProgramObject::Integer(2),
        /* 2 */ ProgramObject::Integer(5),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn function_application_test_three () {
    let ast = AST::CallFunction {
        name: Identifier("f".to_string()),
        arguments: vec!(
            Box::new(AST::Null),
            Box::new(AST::Integer(0)),
            Box::new(AST::Boolean(true)),
        ),
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Literal { index:  ConstantPoolIndex::new(1) },
        /* 1 */ OpCode::Literal { index:  ConstantPoolIndex::new(2) },
        /* 2 */ OpCode::Literal { index:  ConstantPoolIndex::new(3) },
        /* 3 */ OpCode::CallFunction { name: ConstantPoolIndex::new(0), arguments: Arity::new(3) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::String("f".to_string()),
        /* 1 */ ProgramObject::Null,
        /* 2 */ ProgramObject::Integer(0),
        /* 3 */ ProgramObject::Boolean(true),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn function_application_test_one () {
    let ast = AST::CallFunction {
        name: Identifier("f".to_string()),
        arguments: vec!(Box::new(AST::Integer(42))),
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Literal { index:  ConstantPoolIndex::new(1) },
        /* 1 */ OpCode::CallFunction { name: ConstantPoolIndex::new(0), arguments: Arity::new(1) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::String("f".to_string()),
        /* 1 */ ProgramObject::Integer(42),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn function_application_test_zero () {
    let ast = AST::CallFunction {
        name: Identifier("f".to_string()),
        arguments: vec!()
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::CallFunction { name: ConstantPoolIndex::new(0), arguments: Arity::new(0) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::String("f".to_string()),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn function_definition_three () {
    let ast = AST::Function {
        name: Identifier("project_right".to_string()),
        parameters: vec!(Identifier::from("left"),
                         Identifier::from("middle"),
                         Identifier::from("right")),
        body: Box::new(AST::AccessVariable { name: Identifier::from("left") })
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Jump { label: ConstantPoolIndex::new(0) },
        /* 1 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },
        /* 2 */ OpCode::Return,
        /* 3 */ OpCode::Label { name: ConstantPoolIndex::new(0) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::String("λ:project_right:0".to_string()),
        /* 1 */ ProgramObject::String("project_right".to_string()),
        /* 2 */ ProgramObject::Method {
            name: ConstantPoolIndex::new(1),
            parameters: Arity::new(3),
            locals: Size::new(0),
            code: AddressRange::from(1, 3),
        },
    ]);

    let expected_globals = Globals::from(vec![ConstantPoolIndex::new(2)]);
    let expected_entry = Entry::new();
    let expected_methods = vec![ConstantPoolIndex::from_usize(2)];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn object_with_methods_and_fields () {
    let ast = AST::Object {
        extends: Box::new(AST::Boolean(true)),
        members: vec!(
            Box::new(AST::Function {
                name: Identifier::from("implies"),
                parameters: vec!(Identifier::from("x")),
                body: Box::new(AST::Boolean(true))}),

            Box::new(AST::Variable {
                name: Identifier::from("id"),
                value: Box::new(AST::Integer(1))}),

            Box::new(AST::Function {
                name: Identifier::from("identity"),
                parameters: vec!(),
                body: Box::new(AST::Boolean(true))}),

            Box::new(AST::Function {
                name: Identifier::from("or"),
                parameters: vec!(Identifier::from("x")),
                body: Box::new(AST::Boolean(true))}),

            Box::new(AST::Function {
                name: Identifier::from("and"),
                parameters: vec!(Identifier::from("x")),
                body: Box::new(AST::AccessVariable { name: Identifier::from("x") })}),

            Box::new(AST::Variable {
                name: Identifier::from("hash"),
                value: Box::new(AST::Integer(1))}),

            Box::new(AST::Function {
                name: Identifier::from(Operator::Addition),
                parameters: vec!(Identifier::from("x")),
                body: Box::new(AST::Boolean(true))}),

            Box::new(AST::Function {
                name: Identifier::from(Operator::Multiplication),
                parameters: vec!(Identifier::from("x")),
                body: Box::new(AST::AccessVariable { name: Identifier::from("x") })}),

            Box::new(AST::Function {
                name: Identifier::from("me"),
                parameters: vec!(),
                body: Box::new(AST::AccessVariable { name: Identifier::from("this") })}),
        )
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    // for compilation_unit in ast.split_into_compilation_units() {
    //     compilation_unit.compile_into(&mut program,
    //                                   &mut global_environment,
    //                                   &mut current_frame,
    //                                   &mut methods,
    //                                   true).unwrap();
    // }
    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::new();

    let expected_code = Code::from(vec!(
        /*  0 */ OpCode::Jump { label: ConstantPoolIndex::new(0) },      // function_guard_0 - implies
        /*  1 */ OpCode::Literal { index: ConstantPoolIndex::new(1) },   // true
        /*  2 */ OpCode::Return,
        /*  3 */ OpCode::Label { name: ConstantPoolIndex::new(0) },      // function_guard_0

        /*  4 */ OpCode::Jump { label: ConstantPoolIndex::new(4) },      // function_guard_1 - identity
        /*  5 */ OpCode::Literal { index: ConstantPoolIndex::new(1) },   // true
        /*  6 */ OpCode::Return,
        /*  7 */ OpCode::Label { name: ConstantPoolIndex::new(4) },      // function_guard_1

        /* 10 */ OpCode::Jump { label: ConstantPoolIndex::new(7) },     // function_guard_2 - or
        /* 11 */ OpCode::Literal { index: ConstantPoolIndex::new(1) },   // true
        /* 12 */ OpCode::Return,
        /* 13 */ OpCode::Label { name: ConstantPoolIndex::new(7) },     // function_guard_2

        /* 14 */ OpCode::Jump { label: ConstantPoolIndex::new(10) },     // function_guard_3 - or
        /* 15 */ OpCode::GetLocal { index: LocalFrameIndex::new(1) },    // x
        /* 16 */ OpCode::Return,
        /* 17 */ OpCode::Label { name: ConstantPoolIndex::new(10) },     // function_guard_3

        /* 18 */ OpCode::Jump { label: ConstantPoolIndex::new(13) },     // function_guard_4 - +
        /* 20 */ OpCode::Literal { index: ConstantPoolIndex::new(1) },   // true
        /* 21 */ OpCode::Return,
        /* 22 */ OpCode::Label { name: ConstantPoolIndex::new(13) },     // function_guard_4

        /* 23 */ OpCode::Jump { label: ConstantPoolIndex::new(16) },     // function_guard_5 - *
        /* 24 */ OpCode::GetLocal { index: LocalFrameIndex::new(1) },    // x
        /* 25 */ OpCode::Return,
        /* 26 */ OpCode::Label { name: ConstantPoolIndex::new(16) },     // function_guard_5

        /* 27 */ OpCode::Jump { label: ConstantPoolIndex::new(19) },     // function_guard_6 - me
        /* 28 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },    // this
        /* 29 */ OpCode::Return,
        /* 30 */ OpCode::Label { name: ConstantPoolIndex::new(19) },     // function_guard_6

        /* 18 */ OpCode::Literal { index: ConstantPoolIndex::new(1) },   // 1 - hash
        /*  4 */ OpCode::Literal { index: ConstantPoolIndex::new(22) },   // 1 - slot id

        /*  0 */ OpCode::Literal { index: ConstantPoolIndex::new(22) },   // true
        /* 31 */ OpCode::Object { class: ConstantPoolIndex:: new(27) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 00 */ ProgramObject::from_str("λ:implies:0"),
        /* 01 */ ProgramObject::from_bool(true),
        /* 02 */ ProgramObject::from_str("implies"),
        /* 03 */ ProgramObject::Method {
            name: ConstantPoolIndex::new(2),    // implies
            parameters: Arity::new(1+1),
            locals: Size::new(0),
            code: AddressRange::from(1, 2),     // addresses: 2, 3
        },

        /* 04 */ ProgramObject::from_str("λ:identity:1"),
        /* 05 */ ProgramObject::from_str("identity"),
        /* 06 */ ProgramObject::Method {
            name: ConstantPoolIndex::new(5),    // identity
            parameters: Arity::new(0+1),
            locals: Size::new(0),
            code: AddressRange::from(5, 2),     // addresses: 6, 7
        },

        /* 07 */ ProgramObject::from_str("λ:or:2"),
        /* 08 */ ProgramObject::from_str("or"),
        /* 09 */ ProgramObject::Method {
            name: ConstantPoolIndex::new(8),    // or
            parameters: Arity::new(1+1),
            locals: Size::new(0),
            code: AddressRange::from(9, 2),     // addresses: 10, 11
        },

        /* 10 */ ProgramObject::from_str("λ:and:3"),
        /* 11 */ ProgramObject::from_str("and"),
        /* 12 */ ProgramObject::Method {
            name: ConstantPoolIndex::new(11),    // and
            parameters: Arity::new(1+1),
            locals: Size::new(0),
            code: AddressRange::from(13, 2),     // addresses: 14, 15
        },

        /* 13 */ ProgramObject::from_str("λ:+:4"),
        /* 14 */ ProgramObject::from_str("+"),
        /* 15 */ ProgramObject::Method {
            name: ConstantPoolIndex::new(14),          // +
            parameters: Arity::new(1+1),
            locals: Size::new(0),
            code: AddressRange::from(17, 2),
        },

        /* 16 */ ProgramObject::from_str("λ:*:5"),
        /* 17 */ ProgramObject::from_str("*"),
        /* 18 */ ProgramObject::Method {
            name: ConstantPoolIndex::new(17),          // *
            parameters: Arity::new(1+1),
            locals: Size::new(0),
            code: AddressRange::from(21, 2),
        },

        /* 19 */ ProgramObject::from_str("λ:me:6"),
        /* 20 */ ProgramObject::from_str("me"),
        /* 21 */ ProgramObject::Method {
            name: ConstantPoolIndex::new(20),          // *
            parameters: Arity::new(1),
            locals: Size::new(0),
            code: AddressRange::from(25, 2),
        },

        /* 22 */ ProgramObject::from_i32(1),
        /* 23 */ ProgramObject::from_str("id"),
        /* 24 */ ProgramObject::slot_from_u16(23),

        /* 25 */ ProgramObject::from_str("hash"),
        /* 26 */ ProgramObject::slot_from_u16(25),
        /* 27 */ ProgramObject::class_from_vec(vec!(3, 24, 6, 9, 12, 26, 15, 18, 21)),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods: Vec<ConstantPoolIndex> = vec![3, 6, 9, 12, 15, 18, 21].into_iter()
        .map(|e| ConstantPoolIndex::from_usize(e))
        .collect();

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(format!("{}", program), format!("{}", expected_program));

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn block_many () {
    let ast = AST::Block(vec!(
        Box::new(AST::Null),
        Box::new(AST::Integer(1)),
        Box::new(AST::Integer(42)),
        Box::new(AST::Integer(0)),
        Box::new(AST::Boolean(true)),
        Box::new(AST::Integer(42))));

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let mut expected_current_frame = Frame::new();
    match expected_current_frame {
        Frame::Top => unreachable!(),
        Frame::Local(ref mut environment) => {
            environment.enter_scope();
            environment.leave_scope();
        }
    }

    let expected_code = Code::from(vec!(
        /*  0 */ OpCode::Literal { index: ConstantPoolIndex::new(0) },
        /*  1 */ OpCode::Drop,
        /*  2 */ OpCode::Literal { index: ConstantPoolIndex::new(1) },
        /*  3 */ OpCode::Drop,
        /*  4 */ OpCode::Literal { index: ConstantPoolIndex::new(2) },
        /*  5 */ OpCode::Drop,
        /*  6 */ OpCode::Literal { index: ConstantPoolIndex::new(3) },
        /*  7 */ OpCode::Drop,
        /*  8 */ OpCode::Literal { index: ConstantPoolIndex::new(4) },
        /*  9 */ OpCode::Drop,
        /* 10 */ OpCode::Literal { index: ConstantPoolIndex::new(2) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::Null,
        /* 1 */ ProgramObject::from_i32(1),
        /* 2 */ ProgramObject::from_i32(42),
        /* 3 */ ProgramObject::from_i32(0),
        /* 4 */ ProgramObject::from_bool(true),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn block_one () {
    let ast = AST::Block(vec!(Box::new(AST::Null)));

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let mut expected_current_frame = Frame::new();
    match expected_current_frame {
        Frame::Top => unreachable!(),
        Frame::Local(ref mut environment) => {
            environment.enter_scope();
            environment.leave_scope();
        }
    }

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::Literal { index: ConstantPoolIndex::new(0) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::Null,
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn block_zero () {
    let ast = AST::Block(vec!());

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::new();
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let mut expected_current_frame = Frame::new();
    match expected_current_frame {
        Frame::Top => unreachable!(),
        Frame::Local(ref mut environment) => {
            environment.enter_scope();
            environment.leave_scope();
        }
    }

    let expected_code = Code::from(vec!());

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn field_access_test () {
    let ast = AST::AccessField {
        object: Box::new(AST::AccessVariable { name: Identifier::from("obj") }),
        field: Identifier::from("x"),
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec!["obj".to_string()]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec!["obj".to_string()]);

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },
        /* 1 */ OpCode::GetField { name: ConstantPoolIndex::new(0) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::from_str("x"),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn field_mutation_test () {
    let ast = AST::AssignField {
        object: Box::new(AST::AccessVariable { name: Identifier::from("obj") }),
        field: Identifier::from("x"),
        value: Box::new(AST::Integer(42)),
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec!["obj".to_string()]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec!["obj".to_string()]);

    let expected_code = Code::from(vec!(
        /* 0 */ OpCode::GetLocal { index: LocalFrameIndex::new(0) },
        /* 1 */ OpCode::Literal { index: ConstantPoolIndex::new(0) },
        /* 2 */ OpCode::SetField { name: ConstantPoolIndex::new(1) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::from_i32(42),
        /* 1 */ ProgramObject::from_str("x"),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn method_call_test_three () {
    let ast = AST::CallMethod {
        name: Identifier::from("f"),
        arguments: vec!(Box::new(AST::Integer(1)),
                        Box::new(AST::Integer(2)),
                        Box::new(AST::Integer(3))),
        object: Box::new(AST::AccessVariable { name: Identifier::from("obj") })
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec!["obj".to_string()]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec!["obj".to_string()]);

    let expected_code = Code::from(vec!(
        OpCode::GetLocal { index: LocalFrameIndex::new(0) },

        OpCode::Literal { index: ConstantPoolIndex::new(1) },
        OpCode::Literal { index: ConstantPoolIndex::new(2) },
        OpCode::Literal { index: ConstantPoolIndex::new(3) },

        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(4) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::from_str("f"),
        /* 1 */ ProgramObject::from_i32(1),
        /* 2 */ ProgramObject::from_i32(2),
        /* 3 */ ProgramObject::from_i32(3),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn method_call_test_one () {
    let ast = AST::CallMethod {
        name: Identifier::from("f"),
        arguments: vec!(Box::new(AST::Integer(42))),
        object: Box::new(AST::AccessVariable { name: Identifier::from("obj") })
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec!["obj".to_string()]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec!["obj".to_string()]);

    let expected_code = Code::from(vec!(
        OpCode::GetLocal { index: LocalFrameIndex::new(0) },
        OpCode::Literal { index: ConstantPoolIndex::new(1) },
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(2) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::from_str("f"),
        /* 1 */ ProgramObject::from_i32(42),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn method_call_test_zero () {
    let ast = AST::CallMethod {
        name: Identifier::from("f"),
        arguments: vec!(),
        object: Box::new(AST::AccessVariable { name: Identifier::from("obj") })
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec!["obj".to_string()]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec!["obj".to_string()]);

    let expected_code = Code::from(vec!(
        OpCode::GetLocal { index: LocalFrameIndex::new(0) },
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(1) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::from_str("f"),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn operator_call_test () {
    let ast = AST::CallMethod {
        name: Identifier::from(Operator::Subtraction),
        arguments: vec!(Box::new(AST::Integer(1))),
        object: Box::new(AST::Integer(7)),
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec![]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec![]);

    let expected_code = Code::from(vec!(
        OpCode::Literal { index: ConstantPoolIndex::new(1) },
        OpCode::Literal { index: ConstantPoolIndex::new(2) },
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(2) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::from_str("-"),
        /* 1 */ ProgramObject::from_i32(7),
        /* 2 */ ProgramObject::from_i32(1),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

#[test] fn operation_test () {
    let ast = AST::CallMethod {
        name: Identifier::from(Operator::Subtraction),
        object: Box::new(AST::Integer(1)),
        arguments: vec![Box::new(AST::Integer(7))],
    };

    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::from_locals(vec![]);
    let mut methods = vec![];

    let ext_ast = ast.split_into_compilation_units();
    ext_ast.compile_into(&mut program, &mut global_environment, &mut current_frame, &mut methods, true).unwrap();

    let expected_global_environment = Environment::new();
    let expected_current_frame = Frame::from_locals(vec![]);

    let expected_code = Code::from(vec!(
        OpCode::Literal { index: ConstantPoolIndex::new(1) },
        OpCode::Literal { index: ConstantPoolIndex::new(2) },
        OpCode::CallMethod { name: ConstantPoolIndex::new(0), arguments: Arity::new(2) },
    ));

    let expected_constants = <ConstantPool as From<Vec<ProgramObject>>>::from(vec![
        /* 0 */ ProgramObject::from_str("-"),
        /* 1 */ ProgramObject::from_i32(1),
        /* 2 */ ProgramObject::from_i32(7),
    ]);

    let expected_globals = Globals::from(vec![]);
    let expected_entry = Entry::new();
    let expected_methods = vec![];

    let expected_program =
        Program::from(expected_code, expected_constants, expected_globals, expected_entry).unwrap();

    assert_eq!(program, expected_program);
    assert_eq!(global_environment, expected_global_environment);
    assert_eq!(current_frame, expected_current_frame);
    assert_eq!(methods, expected_methods);
}

