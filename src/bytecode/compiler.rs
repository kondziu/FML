use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use crate::parser::*;

use super::types::*;
use super::bytecode::OpCode;
use super::program::Program;
use super::objects::ProgramObject;

pub fn compile(ast: &AST) -> Program {
    let mut program: Program = Program::empty();
    let mut bookkeeping: Bookkeeping = Bookkeeping::without_frame();
    ast.compile_into(&mut program, &mut bookkeeping, true);
    program
}

type Scope = usize;

#[derive(PartialEq,Debug,Clone)]
struct LocalFrame {
    locals: HashMap<(Scope, String), LocalFrameIndex>,
    scopes: Vec<Scope>,
    scope_sequence: Scope,
}

impl LocalFrame {
    fn new() -> Self {
        LocalFrame { locals: HashMap::new(), scopes: vec!(0), scope_sequence: 0 }
    }


    fn from_locals(locals: Vec<String>) -> Self {
        let mut local_map: HashMap<(Scope, String), LocalFrameIndex> = HashMap::new();

        for (i, local) in locals.into_iter().enumerate() {
            local_map.insert((0, local), LocalFrameIndex::from_usize(i));
        }

        LocalFrame { locals: local_map, scopes: vec!(0), scope_sequence: 0 }
    }


    fn from_locals_at(locals: Vec<String>, level: usize) -> Self {
        let mut local_map: HashMap<(Scope, String), LocalFrameIndex> = HashMap::new();

        for (i, local) in locals.into_iter().enumerate() {
            local_map.insert((level, local), LocalFrameIndex::from_usize(i));
        }

        LocalFrame { locals: local_map, scopes: vec!(0), scope_sequence: 0 }
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

    fn generate_new_local(&mut self, name: &str) -> LocalFrameIndex {
        let index = LocalFrameIndex::from_usize(self.locals.len());
        let label = format!("?{}_{}", name, self.locals.len());
        let key = (self.current_scope(), label);
        let result = self.locals.insert(key, index);
        assert!(result.is_none());
        index
    }

    fn enter_scope(&mut self) {
        self.scope_sequence += 1;
        self.scopes.push(self.scope_sequence);
    }

    fn leave_scope(&mut self) {
        self.scopes.pop()
            .expect("Cannot leave scope: the scope stack is empty");
    }
}

#[derive(PartialEq,Debug,Clone)]
pub struct Bookkeeping { // TODO rename
    frames: Vec<LocalFrame>,
    globals: HashSet<String>,
    top: LocalFrame,
}

// enum VariableIndex {
//     Global(ConstantPoolIndex),
//     Local(LocalFrameIndex),
// }

impl Bookkeeping {
    #[allow(dead_code)]
    pub fn with_frame() -> Bookkeeping {
        Bookkeeping {
            frames: vec!(LocalFrame::new()),
            globals: HashSet::new(),
            top: LocalFrame::new(),
        }
    }

    #[allow(dead_code)]
    pub fn without_frame() -> Bookkeeping {
        Bookkeeping {
            frames: vec!(),
            globals: HashSet::new(),
            top: LocalFrame::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from(locals: Vec<String>, globals: Vec<String>) -> Bookkeeping {
        Bookkeeping {
            frames: vec!(LocalFrame::from_locals(locals)),
            globals: globals.into_iter().collect(),
            top: LocalFrame::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_locals(locals: Vec<String>) -> Bookkeeping {
        Bookkeeping {
            frames: vec!(LocalFrame::from_locals(locals)),
            globals: HashSet::new(),
            top: LocalFrame::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_locals_at(locals: Vec<String>, level: usize) -> Bookkeeping {
        Bookkeeping {
            frames: vec!(LocalFrame::from_locals_at(locals, level)),
            globals: HashSet::new(),
            top: LocalFrame::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_globals(globals: Vec<String>) -> Bookkeeping {
        Bookkeeping {
            frames: vec!(),
            globals: globals.into_iter().collect(),
            top: LocalFrame::new(),
        }
    }

    fn add_frame(&mut self) {
        self.frames.push(LocalFrame::new())
    }

    fn remove_frame(&mut self)  {
        self.frames.pop()
            .expect("Bookkeeping: cannot pop frame from an empty stack");       //FIXME
    }

    pub fn enter_scope(&mut self) {
        if self.frames.is_empty() {
            self.top.enter_scope()
        } else {
            self.frames.last_mut().unwrap().enter_scope()
        }
    }

    pub fn leave_scope(&mut self) {
        if self.frames.is_empty() {
            self.top.leave_scope()
        } else {
            self.frames.last_mut().unwrap().leave_scope()
        }
    }

    fn has_frame(&self) -> bool {
        !(self.frames.is_empty() && self.top.in_outermost_scope())
    }

    fn register_global(&mut self, id: &str) {
        self.globals.insert(id.to_string());
    }

    fn has_local(&self, id: &str) -> bool {
        match self.frames.last() {
            None => {
                if self.top.in_outermost_scope() {
                    false
                } else {
                    self.top.has_local(id)
                }
            }
            Some(frame) => {
                frame.has_local(id)
            },
        }
    }

    fn register_new_local(&mut self, id: &str) -> Result<LocalFrameIndex, String> {
        if self.frames.is_empty() {
            self.top.register_new_local(id)
        } else {
            self.frames.last_mut().unwrap().register_new_local(id)
        }
    }

    fn register_local(&mut self, id: &str) -> LocalFrameIndex {
        if self.frames.is_empty() {
            self.top.register_local(id)
        } else {
            self.frames.last_mut().unwrap().register_local(id)
        }
    }

    fn count_locals(&self) -> usize {
        if self.frames.is_empty() {
            self.top.count_locals()
        } else {
            self.frames.last().unwrap().count_locals()
        }
    }

    #[allow(dead_code)]
    fn generate_new_local(&mut self, name: &str) -> LocalFrameIndex {
        if self.frames.is_empty() {
            self.top.generate_new_local(name)
        } else {
            self.frames.last_mut().unwrap().generate_new_local(name)
        }
    }
}

macro_rules! unpack {
    ((_) from $vector:expr) => {{
        let vector = $vector;
        vector[0]
    }};
    ((_,_) from $vector:expr) => {{
        let vector = $vector;
        (vector[0], vector[1])
    }};
    ((_,_,_) from $vector:expr) => {{
        let vector = $vector;
        (vector[0], vector[1], vector[2])
    }};
}

pub trait Compiled {
    fn compile_into(&self, program: &mut Program, environment: &mut Bookkeeping, keep_result: bool);
    fn compile(&self, program: &mut Program, environment: &mut Bookkeeping) {
        self.compile_into(program, environment, true);
    }
}

impl Compiled for AST {
    fn compile_into(&self, program: &mut Program, environment: &mut Bookkeeping, keep_result: bool) {
        match self {
            AST::Integer(value) => {
                let constant = ProgramObject::Integer(*value);
                let index = program.register_constant(constant);
                program.emit_code(OpCode::Literal { index });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::Boolean(value) => {
                let constant = ProgramObject::Boolean(*value);
                let index = program.register_constant(constant);
                program.emit_code(OpCode::Literal { index });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::Null => {
                let constant = ProgramObject::Null;
                let index = program.register_constant(constant);
                program.emit_code(OpCode::Literal { index });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::Variable { name: Identifier(name), value } => {
                if environment.has_frame() {
                    let index = environment.register_new_local(name)
                        .expect(&format!("Cannot register new variable {}", &name))
                        .clone();   // FIXME error if not new
                    value.deref().compile_into(program, environment, true);    // FIXME scoping!!!
                    program.emit_code(OpCode::SetLocal { index });

                } else {
                    let index = program.register_constant(ProgramObject::from_str(name));
                    environment.register_global(name);                  // TODO necessary?
                    value.deref().compile_into(program, environment, true);
                    program.emit_code(OpCode::SetGlobal { name: index });
                }
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::AccessVariable { name: Identifier(name) } => {
                if environment.has_local(name) {
                    let index = environment.register_local(name).clone();   // FIXME error if does not exists
                    program.emit_code(OpCode::GetLocal { index });      // FIXME scoping!!!
                } else {
                    let index = program.register_constant(ProgramObject::from_str(name));
                    environment.register_global(name);                  // TODO necessary?
                    program.emit_code(OpCode::GetGlobal { name: index });
                }
            }

            AST::AssignVariable { name: Identifier(name), value } => {
                if environment.has_local(name) {
                    let index = environment.register_local(name).clone(); // FIXME error if does not exists
                    value.deref().compile_into(program, environment, true);    // FIXME scoping!!!
                    program.emit_code(OpCode::SetLocal { index });
                } else {
                    let index = program.register_constant(ProgramObject::from_str(name));
                    environment.register_global(name);                  // TODO necessary?
                    value.deref().compile_into(program, environment, true);
                    program.emit_code(OpCode::SetGlobal { name: index });
                }
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::Conditional { condition, consequent, alternative } => {
                let (consequent_label_index, end_label_index) =
                    unpack!((_,_) from program.generate_new_label_names(vec!["if_consequent", "if_end"]));

                (**condition).compile_into(program, environment, true);
                program.emit_code(OpCode::Branch { label: consequent_label_index} );
                (**alternative).compile_into(program, environment, keep_result);
                program.emit_code(OpCode::Jump { label: end_label_index} );
                program.emit_code(OpCode::Label { name: consequent_label_index });
                (**consequent).compile_into(program, environment, keep_result);
                program.emit_code(OpCode::Label { name: end_label_index });
            }

            AST::Loop { condition, body } => {
                let (body_label_index, condition_label_index)
                    = unpack!((_,_) from program.generate_new_label_names(vec!["loop_body", "loop_condition"]));

                program.emit_code(OpCode::Jump { label: condition_label_index });
                program.emit_code(OpCode::Label { name: body_label_index });
                (**body).compile_into(program, environment, false);
                program.emit_code(OpCode::Label { name: condition_label_index });
                (**condition).compile_into(program, environment, true);
                program.emit_code(OpCode::Branch { label: body_label_index });

                if keep_result {
                    let constant = ProgramObject::Null;
                    let index = program.register_constant(constant);
                    program.emit_code(OpCode::Literal { index });
                }
            }

            AST::Array { size, value } => {
                match value.deref() {
                    AST::Boolean(_) | AST::Integer(_) | AST::Null |
                    AST::AccessVariable { name:_ } | AST::AccessField { object:_, field:_ } => {
                        size.deref().compile_into(program, environment, true);
                        value.deref().compile_into(program, environment, true);
                        program.emit_code(OpCode::Array);
                        program.emit_conditionally(OpCode::Drop, !keep_result);
                    },
                    _ => {
                        let i_id = Identifier::from("::i");
                        let size_id = Identifier::from("::size");
                        let array_id = Identifier::from("::array");

                        // let ::size = eval SIZE;
                        let size_definition = AST::Variable {
                            name: size_id.clone(), value: size.clone(),
                        };

                        // let ::array = array(::size, null);
                        let array_definition = AST::Variable {
                            name: array_id.clone(),
                            value: Box::new(AST::Array {
                                size: Box::new(AST::AccessVariable { name: size_id.clone() }),
                                value: Box::new(AST::Null),
                            })
                        };

                        // let ::i = 0;
                        let i_definition = AST::Variable {
                            name: i_id.clone(), value: Box::new(AST::Integer(0)),
                        };

                        // ::array[::i] <- eval VALUE;
                        let set_array = AST::AssignArray {
                            array: Box::new(AST::AccessVariable { name: array_id.clone() }),
                            index: Box::new(AST::AccessVariable { name: i_id.clone() }),
                            value: value.clone(),
                        };

                        // ::i <- ::i + 1;
                        let increment_i = AST::AssignVariable {
                            name: i_id.clone(),
                            value: Box::new(AST::Operation {
                                operator: Operator::Addition,
                                left: Box::new(AST::AccessVariable { name: i_id.clone() }),
                                right: Box::new(AST::Integer(1) )
                            })
                        };

                        // ::i < ::size
                        let comparison = AST::Operation {
                            operator: Operator::Less,
                            left: Box::new(AST::AccessVariable { name: i_id }),
                            right: Box::new(AST::AccessVariable { name: size_id }),
                        };

                        // while ::i < ::size do
                        // begin
                        //   ::array[::i] <- eval VALUE;
                        //   ::i <- ::i + 1;
                        // end;
                        let loop_de_loop = AST::Loop {
                            condition: Box::new(comparison),
                            body: Box::new(AST::Block(vec![
                                Box::new(set_array),
                                Box::new(increment_i),
                            ]))
                        };

                        // ::array
                        let array = AST::AccessVariable { name: array_id };

                        // let ::size = eval SIZE;
                        // let ::array = array(::size, null);
                        // let ::i = 0;
                        // while ::i < ::size do
                        // begin
                        //   ::array[::i] <- eval VALUE;
                        //   ::i <- ::i + 1;
                        // end;
                        // ::array
                        size_definition.compile_into(program, environment, false);
                        array_definition.compile_into(program, environment, false);
                        i_definition.compile_into(program, environment, false);
                        loop_de_loop.compile_into(program, environment, false);
                        array.compile_into(program, environment, keep_result);
                    }
                }
            }

            AST::AccessArray { array, index } => {
                (**array).compile_into(program, environment, true);
                (**index).compile_into(program, environment, true);
                let name = program.register_constant(ProgramObject::String("get".to_string()));
                program.emit_code(OpCode::CallMethod { name, arguments: Arity::new(2) });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::AssignArray { array, index, value } => {
                (**array).compile_into(program, environment, true);
                (**index).compile_into(program, environment, true);
                (**value).compile_into(program, environment, true);
                let name = program.register_constant(ProgramObject::String("set".to_string()));
                program.emit_code(OpCode::CallMethod { name, arguments: Arity::new(3) });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::Print { format, arguments } => {
                let format: ConstantPoolIndex =
                    program.register_constant(ProgramObject::String(format.to_string()));

                for argument in arguments.iter() {
                    argument.compile_into(program, environment, true);
                }

                let arguments = Arity::from_usize(arguments.len());
                program.emit_code(OpCode::Print { format, arguments });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::Operator { operator, parameters, body } => {
                let name = operator.as_str();
                let end_label_index = unpack!((_) from program.generate_new_label_names(vec!["function_guard"])); // FIXME merge with FunctionDefinition

                program.emit_code(OpCode::Jump { label: end_label_index });
                let start_address = program.get_upcoming_address();

                environment.add_frame();
                for parameter in parameters.into_iter() {
                    environment.register_local(parameter.as_str());
                }

                (**body).compile_into(program, environment, true);

                let locals_in_frame = environment.count_locals();
                environment.remove_frame();

                program.emit_code(OpCode::Return);
                program.emit_code(OpCode::Label { name: end_label_index });
                let end_address = program.get_current_address();

                let name = ProgramObject::String(name.to_string());
                let name_index = program.register_constant(name);

                let method = ProgramObject::Method {
                    name: name_index,
                    locals: Size::from_usize(locals_in_frame - parameters.len()),
                    arguments: Arity::from_usize(parameters.len()),
                    code: AddressRange::from_addresses(start_address, end_address),
                };
                program.register_constant(method);
            }

            AST::Function { name: Identifier(name), parameters, body } => {
                let end_label_index = unpack!((_) from program.generate_new_label_names(vec!["function_guard"]));

                program.emit_code(OpCode::Jump { label: end_label_index });
                let start_address = program.get_upcoming_address();

                environment.add_frame();
                for parameter in parameters.into_iter() {
                    environment.register_local(parameter.as_str());
                }

                (**body).compile_into(program, environment, true);

                let locals_in_frame = environment.count_locals();
                environment.remove_frame();

                program.emit_code(OpCode::Return);
                program.emit_code(OpCode::Label { name: end_label_index });
                let end_address = program.get_current_address();

                let name = ProgramObject::String(name.to_string());
                let name_index = program.register_constant(name);

                let method = ProgramObject::Method {
                    name: name_index,
                    locals: Size::from_usize(locals_in_frame - parameters.len()),
                    arguments: Arity::from_usize(parameters.len()),
                    code: AddressRange::from_addresses(start_address, end_address),
                };
                let constant = program.register_constant(method);
                program.register_global(constant)  // FIXME local functions should not be visible globally
            }

            AST::CallFunction { name: Identifier(name), arguments } => {
                let index = program.register_constant(ProgramObject::String(name.to_string()));
                for argument in arguments.iter() {
                    argument.compile_into(program, environment, true);
                }
                let arity = Arity::from_usize(arguments.len());
                program.emit_code(OpCode::CallFunction { name: index, arguments: arity });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::Object { extends, members } => {
                (**extends).compile_into(program, environment, true);

                let slots: Vec<ConstantPoolIndex> = members.iter().map(|m| m.deref()).map(|m| match m {
                    AST::Function { name, parameters, body } => {
                        compile_function_definition(name.as_str(), true, parameters, body.deref(),
                                                    program, environment)

                    }
                    AST::Operator { operator, parameters, body } => {
                        compile_function_definition(operator.as_str(), true, parameters, body.deref(),
                                                    program, environment)

                    }
                    AST::Variable { name: Identifier(name), value } => {
                        (*value).compile_into(program, environment, true);
                        let index = program.register_constant(ProgramObject::from_str(name));
                        program.register_constant(ProgramObject::slot_from_index(index))
                    },
                    _ => panic!("Object definition: cannot define a member from {:?}", m)
                }).collect();


                let class = ProgramObject::Class(slots);
                let class_index = program.register_constant(class);

                program.emit_code(OpCode::Object { class: class_index });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::Block(children) => {
                environment.enter_scope();
                let length = children.len();
                for (i, child) in children.iter().enumerate() {
                    let last = i + 1 == length;
                    child.deref().compile_into(program, environment, last && keep_result)
                }
                environment.leave_scope();
            }

            AST::AccessField { object, field: Identifier(name) } => {
                object.deref().compile_into(program, environment, true);
                let index = program.register_constant(ProgramObject::from_str(name));
                program.emit_code(OpCode::GetSlot { name: index });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::AssignField { object, field: Identifier(name), value } => {
                object.deref().compile_into(program, environment, true);
                value.deref().compile_into(program, environment, true);
                let index = program.register_constant(ProgramObject::from_str(name));
                program.emit_code(OpCode::SetSlot { name: index });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::CallMethod { object, name: Identifier(name), arguments } => {
                let index = program.register_constant(ProgramObject::from_str(name));
                object.deref().compile_into(program, environment, true);
                for argument in arguments.iter() {
                    argument.compile_into(program, environment, true);
                }
                let arity = Arity::from_usize(arguments.len() + 1);
                program.emit_code(OpCode::CallMethod { name: index, arguments: arity });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::CallOperator { object, operator, arguments } => {
                let index = program.register_constant(ProgramObject::from_str(operator.as_str()));
                object.deref().compile_into(program, environment, true);
                for argument in arguments.iter() {
                    argument.compile_into(program, environment, true);
                }
                let arity = Arity::from_usize(arguments.len() + 1);
                program.emit_code(OpCode::CallMethod { name: index, arguments: arity });
                program.emit_conditionally(OpCode::Drop, !keep_result);
            }

            AST::Operation { operator, left, right } => {
                let index = program.register_constant(ProgramObject::from_str(operator.as_str()));
                left.deref().compile_into(program, environment, true);
                right.deref().compile_into(program, environment, true);
                let arity = Arity::from_usize(2);
                program.emit_code(OpCode::CallMethod { name: index, arguments: arity });
            }

            AST::Top (children) => {
                let (function_name_index, end_label_index )
                    = unpack!((_,_) from program.generate_new_label_names(vec!["^", "$"]));

                program.emit_code(OpCode::Jump { label: end_label_index });
                let start_address = program.get_upcoming_address();

                let children_count = children.len();
                for (i, child) in children.iter().enumerate() {
                    let last = children_count == i + 1;
                    child.deref().compile_into(program, environment, last)
                    // TODO could be cute to pop exit status off of stack
                }

                program.emit_code(OpCode::Label { name: end_label_index });
                let end_address = program.get_current_address();

                let method = ProgramObject::Method {
                    name: function_name_index,
                    locals: Size::from_usize(environment.top.count_locals()),
                    arguments: Arity::from_usize(0),
                    code: AddressRange::from_addresses(start_address, end_address),
                };

                let function_index = program.register_constant(method);
                program.set_entry(function_index);
            }
        }
    }
}

fn compile_function_definition(name: &str,
                               receiver: bool,
                               parameters: &Vec<Identifier>,
                               body: &AST,
                               program: &mut Program,
                               environment: &mut Bookkeeping) -> ConstantPoolIndex {

    let end_label_index =
        unpack!((_) from program.generate_new_label_names(vec!["function_guard"]));
    program.emit_code(OpCode::Jump { label: end_label_index });

    let expected_arguments = parameters.len() + if receiver { 1 } else { 0 };

    let start_address = program.get_upcoming_address();

    environment.add_frame();

    if receiver {
        environment.register_local("this");
    }

    for parameter in parameters.into_iter() {
        environment.register_local(parameter.as_str());
    }

    body.compile_into(program, environment, true);

    let locals_in_frame = environment.count_locals();
    environment.remove_frame();

    program.emit_code(OpCode::Return);
    let end_address = program.get_current_address();

    program.emit_code(OpCode::Label { name: end_label_index });

    let name = ProgramObject::String(name.to_string());
    let name_index = program.register_constant(name);

    let method = ProgramObject::Method {
        name: name_index,
        locals: Size::from_usize(locals_in_frame - expected_arguments),
        arguments: Arity::from_usize(expected_arguments),
        code: AddressRange::from_addresses(start_address, end_address),
    };

    program.register_constant(method)
}
