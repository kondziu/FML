use std::collections::HashMap;
use std::ops::Deref;

use crate::parser::*;

use super::bytecode::OpCode;
use super::program::Program;
use crate::bytecode::program::*;

use anyhow::*;

pub fn compile(ast: &AST) -> Result<Program> {
    let mut program: Program = Program::new();
    let mut global_environment = Environment::new();
    let mut current_frame = Frame::Top;

    ast.compile_into(&mut program, &mut global_environment, &mut current_frame, true)?;
    Ok(program)
}

type Scope = usize;

#[derive(PartialEq,Debug,Clone)]
pub enum Frame {
    Local(Environment),
    Top,
}

impl Frame {
    #[allow(dead_code)] // only in tests
    pub fn new() -> Self {
        Frame::Local(Environment::new())
    }
    #[allow(dead_code)] // only in tests
    pub fn from_locals(locals: Vec<String>) -> Self {
        Frame::Local(Environment::from_locals(locals))
    }
    #[allow(dead_code)] // only in tests
    pub fn from_locals_at(locals: Vec<String>, level: usize) -> Self {
        Frame::Local(Environment::from_locals_at(locals, level))
    }
}

#[derive(PartialEq,Debug,Clone)]
pub struct Environment {
    locals: HashMap<(Scope, String), LocalFrameIndex>,
    scopes: Vec<Scope>,
    scope_sequence: Scope,
}

impl Environment {
    pub fn new() -> Self {
        Environment { locals: HashMap::new(), scopes: vec!(0), scope_sequence: 0 }
    }

    pub fn from_locals(locals: Vec<String>) -> Self {
        let mut local_map: HashMap<(Scope, String), LocalFrameIndex> = HashMap::new();

        for (i, local) in locals.into_iter().enumerate() {
            local_map.insert((0, local), LocalFrameIndex::from_usize(i));
        }

        Environment { locals: local_map, scopes: vec!(0), scope_sequence: 0 }
    }


    pub fn from_locals_at(locals: Vec<String>, level: usize) -> Self {
        let mut local_map: HashMap<(Scope, String), LocalFrameIndex> = HashMap::new();

        for (i, local) in locals.into_iter().enumerate() {
            local_map.insert((level, local), LocalFrameIndex::from_usize(i));
        }

        Environment { locals: local_map, scopes: vec!(0), scope_sequence: level + 1 }
    }

    fn current_scope(&self) -> Scope {
        *self.scopes.last().expect("Cannot pop from empty scope stack")
    }

    fn register_new_local(&mut self, id: &str) -> Result<LocalFrameIndex, String> {
        let key = (self.current_scope(), id.to_string());

        if let Some(index) = self.locals.get(&key) {
            return Err(format!("Local {} already exist (at index {:?}) and cannot be redefined",
                                id, index))
        }

        let index = LocalFrameIndex::from_usize(self.locals.len());
        let previous = self.locals.insert(key, index);
        assert!(previous.is_none());
        Ok(index)
    }

    fn register_local(&mut self, id: &str) -> LocalFrameIndex {
        for scope in self.scopes.iter().rev() {
            let key = (*scope, id.to_owned());
            if let Some(index) = self.locals.get(&key) {
                return *index;
            }
        }

        let key = (self.current_scope(), id.to_string());
        let index = LocalFrameIndex::from_usize(self.locals.len());
        self.locals.insert(key, index);
        index
    }

    fn has_local(&self, id: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if self.locals.contains_key(&(*scope, id.to_string())) {
                return true;
            }
        }
        return false;
    }

    fn in_outermost_scope(&self) -> bool {
        assert!(!self.scopes.is_empty());
        self.scopes.len() == 1
    }

    fn count_locals(&self) -> usize {
        self.locals.len()
    }

    pub fn enter_scope(&mut self) {
        self.scope_sequence += 1;
        self.scopes.push(self.scope_sequence);
    }

    pub fn leave_scope(&mut self) {
        self.scopes.pop()
            .expect("Cannot leave scope: the scope stack is empty");
    }
}

pub trait Compiled {
    fn compile_into(&self, program: &mut Program, global_environment: &mut Environment, current_frame: &mut Frame, keep_result: bool) -> Result<()>;
    fn compile(&self, program: &mut Program, global_environment: &mut Environment, current_frame: &mut Frame) -> Result<()> {
        self.compile_into(program, global_environment, current_frame, true)
    }
}

impl Compiled for AST {
    fn compile_into(&self, program: &mut Program, global_environment: &mut Environment, current_frame: &mut Frame, keep_result: bool) -> Result<()> {
        match self {
            AST::Integer(value) => {
                let constant = ProgramObject::Integer(*value);
                let index = program.constant_pool.register(constant);
                program.code.emit(OpCode::Literal { index });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::Boolean(value) => {
                let constant = ProgramObject::Boolean(*value);
                let index = program.constant_pool.register(constant);
                program.code.emit(OpCode::Literal { index });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::Null => {
                let constant = ProgramObject::Null;
                let index = program.constant_pool.register(constant);
                program.code.emit(OpCode::Literal { index });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::Variable { name: Identifier(name), value } => {
                value.deref().compile_into(program, global_environment, current_frame, true)?;
                match current_frame {
                    Frame::Local(environment) => {
                        let index = environment.register_new_local(name)
                            .expect(&format!("Cannot register new variable {}", &name))
                            .clone();
                        program.code.emit(OpCode::SetLocal { index });
                    },
                    Frame::Top if !global_environment.in_outermost_scope() => {
                        let index = global_environment.register_new_local(name)
                            .expect(&format!("Cannot register new variable {}", &name))
                            .clone();
                        program.code.emit(OpCode::SetLocal { index });
                    },
                    _ => {
                        let name_index = program.constant_pool.register(ProgramObject::from_str(name));
                        let slot_index =
                            program.constant_pool.register(ProgramObject::Slot { name: name_index });
                        program.globals.register(slot_index)
                            .expect(&format!("Cannot register new global {}", name));
                        program.code.emit(OpCode::SetGlobal { name: name_index });
                    },
                }
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::AccessVariable { name: Identifier(name) } => {
                match current_frame {
                    Frame::Local(environment) if environment.has_local(name) => {
                        let index = environment.register_local(name).clone();
                        program.code.emit(OpCode::GetLocal { index });
                    },
                    Frame::Top if !global_environment.in_outermost_scope() && global_environment.has_local(name) => {
                        let index = global_environment.register_local(name).clone();
                        program.code.emit(OpCode::GetLocal { index });
                    },
                    _ => {
                        let index = program.constant_pool.register(ProgramObject::from_str(name));
                        program.code.emit(OpCode::GetGlobal { name: index });
                    },
                }
            }

            AST::AssignVariable { name: Identifier(name), value } => {
                match current_frame {
                    Frame::Local(environment) if environment.has_local(name) => {
                        let index = environment.register_local(name).clone(); // FIXME error if does not exists
                        value.deref().compile_into(program, global_environment, current_frame, true)?;    // FIXME scoping!!!
                        program.code.emit(OpCode::SetLocal { index });
                    },
                    Frame::Top if !global_environment.in_outermost_scope() && global_environment.has_local(name) => {
                        let index = global_environment.register_local(name).clone(); // FIXME error if does not exists
                        value.deref().compile_into(program, global_environment, current_frame, true)?;    // FIXME scoping!!!
                        program.code.emit(OpCode::SetLocal { index });
                    },
                    _ => {
                        let index = program.constant_pool.register(ProgramObject::from_str(name));
                        value.deref().compile_into(program, global_environment, current_frame, true)?;
                        program.code.emit(OpCode::SetGlobal { name: index });
                    },
                }
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::Conditional { condition, consequent, alternative } => {
                let label_generator = program.labels.create_group();
                let consequent_label = label_generator.generate_name("if:consequent")?;
                let end_label = label_generator.generate_name("if:end")?;

                let consequent_label_index =
                    program.constant_pool.register(ProgramObject::from_str(&consequent_label));
                let end_label_index =
                    program.constant_pool.register(ProgramObject::from_str(&end_label));

                (**condition).compile_into(program, global_environment, current_frame, true)?;
                program.code.emit(OpCode::Branch { label: consequent_label_index } );
                (**alternative).compile_into(program, global_environment, current_frame, keep_result)?;
                program.code.emit(OpCode::Jump { label: end_label_index } );
                program.code.emit(OpCode::Label { name: consequent_label_index });
                program.labels.set(consequent_label, program.code.current_address())?;
                (**consequent).compile_into(program, global_environment, current_frame, keep_result)?;
                program.code.emit(OpCode::Label { name: end_label_index });
                program.labels.set(end_label, program.code.current_address())?;
            }

            AST::Loop { condition, body } => {
                let label_generator = program.labels.create_group();
                let body_label = label_generator.generate_name("loop:body")?;
                let condition_label = label_generator.generate_name("loop:condition")?;

                let body_label_index =
                    program.constant_pool.register(ProgramObject::from_str(&body_label));
                let condition_label_index =
                    program.constant_pool.register(ProgramObject::from_str(&condition_label));

                program.code.emit(OpCode::Jump { label: condition_label_index });
                program.code.emit(OpCode::Label { name: body_label_index });
                program.labels.set(body_label, program.code.current_address())?;
                (**body).compile_into(program, global_environment, current_frame, false)?;
                program.code.emit(OpCode::Label { name: condition_label_index });
                program.labels.set(condition_label, program.code.current_address())?;
                (**condition).compile_into(program, global_environment, current_frame, true)?;
                program.code.emit(OpCode::Branch { label: body_label_index });

                if keep_result {
                    let constant = ProgramObject::Null;
                    let index = program.constant_pool.register(constant);
                    program.code.emit(OpCode::Literal { index });
                }
            }

            AST::Array { size, value } => {
                match value.deref() {
                    AST::Boolean(_) | AST::Integer(_) | AST::Null |
                    AST::AccessVariable { name:_ } | AST::AccessField { object:_, field:_ } => {
                        size.deref().compile_into(program, global_environment, current_frame, true)?;
                        value.deref().compile_into(program, global_environment, current_frame, true)?;
                        program.code.emit(OpCode::Array);
                        program.code.emit_unless(OpCode::Drop, keep_result);
                    },
                    _ => {
                        let i_id = Identifier::from("::i");
                        let size_id = Identifier::from("::size");
                        let array_id = Identifier::from("::array");

                        // let ::size = eval SIZE;
                        let size_definition =
                            AST::variable(size_id.clone(), *size.clone());

                        // let ::array = array(::size, null);
                        let array_definition = AST::variable(
                            array_id.clone(),
                            AST::array(
                                AST::access_variable(size_id.clone()),
                                AST::null()));

                        // let ::i = 0;
                        let i_definition = AST::variable(
                            i_id.clone(), AST::integer(0));

                        // ::array[::i] <- eval VALUE;
                        let set_array = AST::assign_array(
                            AST::access_variable(array_id.clone()),
                            AST::access_variable(i_id.clone()),
                            *value.clone());

                        // ::i <- ::i + 1;
                        let increment_i = AST::assign_variable(
                            i_id.clone(),
                            AST::operation(
                                Operator::Addition,
                                AST::access_variable(i_id.clone()),
                                AST::Integer(1)));

                        // ::i < ::size
                        let comparison = AST::operation(
                            Operator::Less,
                            AST::access_variable(i_id),
                            AST::access_variable(size_id));

                        // while ::i < ::size do
                        // begin
                        //   ::array[::i] <- eval VALUE;
                        //   ::i <- ::i + 1;
                        // end;
                        let loop_de_loop = AST::loop_de_loop(
                            comparison,
                            AST::block(vec![set_array, increment_i]));

                        // ::array
                        let array = AST::access_variable(array_id);

                        // let ::size = eval SIZE;
                        // let ::array = array(::size, null);
                        // let ::i = 0;
                        // while ::i < ::size do
                        // begin
                        //   ::array[::i] <- eval VALUE;
                        //   ::i <- ::i + 1;
                        // end;
                        // ::array
                        size_definition.compile_into(program, global_environment, current_frame, false)?;
                        array_definition.compile_into(program, global_environment, current_frame, false)?;
                        i_definition.compile_into(program, global_environment, current_frame, false)?;
                        loop_de_loop.compile_into(program, global_environment, current_frame, false)?;
                        array.compile_into(program, global_environment, current_frame, keep_result)?;
                    }
                }
            }

            AST::AccessArray { array, index } => {
                (**array).compile_into(program, global_environment, current_frame, true)?;
                (**index).compile_into(program, global_environment, current_frame, true)?;

                let name =
                    program.constant_pool.register(ProgramObject::String("get".to_string()));

                program.code.emit(OpCode::CallMethod { name, arguments: Arity::new(2) });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::AssignArray { array, index, value } => {
                (**array).compile_into(program, global_environment, current_frame, true)?;
                (**index).compile_into(program, global_environment, current_frame, true)?;
                (**value).compile_into(program, global_environment, current_frame, true)?;

                let name =
                    program.constant_pool.register(ProgramObject::String("set".to_string()));

                program.code.emit(OpCode::CallMethod { name, arguments: Arity::new(3) });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::Print { format, arguments } => {
                let format: ConstantPoolIndex =
                    program.constant_pool.register(ProgramObject::String(format.to_string()));

                for argument in arguments.iter() {
                    argument.compile_into(program, global_environment, current_frame, true)?;
                }

                let arguments = Arity::from_usize(arguments.len());
                program.code.emit(OpCode::Print { format, arguments });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::Function { name: Identifier(name), parameters, body } => {
                let end_label = program.labels.generate_name(format!("λ:{}", name))?;
                let end_label_index =
                    program.constant_pool.register(ProgramObject::from_str(&end_label));

                program.code.emit(OpCode::Jump { label: end_label_index });
                let start_address = program.code.upcoming_address();

                let mut child_environment = Environment::new();
                for parameter in parameters.into_iter() { // TODO Environment::from
                    child_environment.register_local(parameter.as_str());
                }
                let mut child_frame = &mut Frame::Local(child_environment);

                (**body).compile_into(program, global_environment, &mut child_frame, true)?;

                let locals_in_frame = match child_frame {
                    Frame::Local(child_environment) => child_environment.count_locals(),
                    Frame::Top => unreachable!(),
                };

                program.code.emit(OpCode::Return);

                program.code.emit(OpCode::Label { name: end_label_index });
                program.labels.set(end_label, program.code.current_address())?;
                let end_address = program.code.current_address();

                let name = ProgramObject::String(name.to_string());
                let name_index = program.constant_pool.register(name);

                let method = ProgramObject::Method {
                    name: name_index,
                    locals: Size::from_usize(locals_in_frame - parameters.len()),
                    parameters: Arity::from_usize(parameters.len()),
                    code: AddressRange::from_addresses(start_address, end_address),
                };

                let constant = program.constant_pool.register(method);
                program.globals.register(constant)?;
            }

            AST::CallFunction { name: Identifier(name), arguments } => {
                let index = program.constant_pool.register(ProgramObject::String(name.to_string()));
                for argument in arguments.iter() {
                    argument.compile_into(program, global_environment, current_frame, true)?;
                }
                let arity = Arity::from_usize(arguments.len());
                program.code.emit(OpCode::CallFunction { name: index, arguments: arity });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::Object { extends, members } => {
                (**extends).compile_into(program, global_environment, current_frame, true)?;

                let slots: Result<Vec<ConstantPoolIndex>> = members.iter().map(|m| m.deref()).map(|m| match m {
                    AST::Function { name, parameters, body } => {
                        compile_function_definition(name.as_str(), true, parameters, body.deref(),
                                                    program, global_environment, current_frame)

                    }
                    AST::Variable { name: Identifier(name), value } => {
                        (*value).compile_into(program, global_environment, current_frame, true)?;
                        let index = program.constant_pool.register(ProgramObject::from_str(name));
                        Ok(program.constant_pool.register(ProgramObject::slot_from_index(index)))
                    },
                    _ => panic!("Object definition: cannot define a member from {:?}", m)
                }).collect();


                let class = ProgramObject::Class(slots?);
                let class_index = program.constant_pool.register(class);

                program.code.emit(OpCode::Object { class: class_index });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::Block(children) => {
                match current_frame {
                    Frame::Local(environment) => environment.enter_scope(),
                    Frame::Top => global_environment.enter_scope(),
                }

                let length = children.len();
                for (i, child) in children.iter().enumerate() {
                    let last = i + 1 == length;
                    child.deref().compile_into(program, global_environment, current_frame, last && keep_result)?;
                }

                match current_frame {
                    Frame::Local(environment) => environment.leave_scope(),
                    Frame::Top => global_environment.leave_scope(),
                }
            }

            AST::AccessField { object, field: Identifier(name) } => {
                object.deref().compile_into(program, global_environment, current_frame, true)?;
                let index = program.constant_pool.register(ProgramObject::from_str(name));
                program.code.emit(OpCode::GetField { name: index });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::AssignField { object, field: Identifier(name), value } => {
                object.deref().compile_into(program, global_environment, current_frame, true)?;
                value.deref().compile_into(program, global_environment, current_frame, true)?;
                let index = program.constant_pool.register(ProgramObject::from_str(name));
                program.code.emit(OpCode::SetField { name: index });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::CallMethod { object, name: Identifier(name), arguments } => {
                let index = program.constant_pool.register(ProgramObject::from_str(name));
                object.deref().compile_into(program, global_environment, current_frame, true)?;
                for argument in arguments.iter() {
                    argument.compile_into(program, global_environment, current_frame, true)?;
                }
                let arity = Arity::from_usize(arguments.len() + 1);
                program.code.emit(OpCode::CallMethod { name: index, arguments: arity });
                program.code.emit_unless(OpCode::Drop, keep_result);
            }

            AST::Top (children) => {
                let function_name_index=
                    program.constant_pool.register(ProgramObject::from_string("λ:".to_owned()));

                let start_address = program.code.upcoming_address();

                let children_count = children.len();
                for (i, child) in children.iter().enumerate() {
                    let last = children_count == i + 1;
                    child.deref().compile_into(program, global_environment, current_frame, last)?;                        // TODO uggo
                    // TODO could be cute to pop exit status off of stack
                }

                let end_address = program.code.current_address();

                // println!("start addr: {}", start_address);
                // println!("end addr: {}", start_address);

                let method = ProgramObject::Method {
                    name: function_name_index,
                    locals: Size::from_usize(global_environment.count_locals()),
                    parameters: Arity::from_usize(0),
                    code: AddressRange::from_addresses(start_address, end_address),
                };

                let function_index = program.constant_pool.register(method);
                program.entry.set(function_index);
            }
        };

        Ok(())
    }
}

fn compile_function_definition(name: &str,
                               receiver: bool,
                               parameters: &Vec<Identifier>,
                               body: &AST,
                               program: &mut Program,
                               global_environment: &mut Environment,
                               _current_frame: &mut Frame) -> Result<ConstantPoolIndex> {

    let end_label = program.labels.generate_name(format!("λ:{}", name))?;
    let end_label_index =
        program.constant_pool.register(ProgramObject::from_str(&end_label));

    program.code.emit(OpCode::Jump { label: end_label_index });

    let expected_arguments = parameters.len() + if receiver { 1 } else { 0 };

    let start_address = program.code.upcoming_address();

    let mut child_environment = Environment::new();
    if receiver {
        child_environment.register_local("this");
    }
    for parameter in parameters.into_iter() { // TODO Environment::from
        child_environment.register_local(parameter.as_str());
    }
    let mut child_frame = &mut Frame::Local(child_environment);

    body.compile_into(program, global_environment, &mut child_frame, true)?;

    let locals_in_frame = match child_frame {
        Frame::Local(child_environment) => child_environment.count_locals(),
        Frame::Top => unreachable!(),
    };
    //child_environment.remove_frame();

    program.code.emit(OpCode::Return);
    let end_address = program.code.current_address();

    program.code.emit(OpCode::Label { name: end_label_index });
    program.labels.set(end_label, program.code.current_address())?;

    let name_object = ProgramObject::String(name.to_string());
    let name_index = program.constant_pool.register(name_object);

    let method = ProgramObject::Method {
        name: name_index,
        locals: Size::from_usize(locals_in_frame - expected_arguments),
        parameters: Arity::from_usize(expected_arguments),
        code: AddressRange::from_addresses(start_address, end_address),
    };

    Ok(program.constant_pool.register(method))
}
