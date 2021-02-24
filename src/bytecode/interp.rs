use crate::bytecode::types::{ConstantPoolIndex, LocalFrameIndex, Arity, Size, AddressRange, Address};

use std::fmt::Write;

use crate::bytecode::bytecode::OpCode;
use crate::bytecode::objects::{ProgramObject, Pointer, HeapObject, HeapIndex, ObjectInstance};

use anyhow::*;
use anyhow::Context;
use std::collections::HashMap;
use std::iter::repeat;

macro_rules! bail_if {
    ($condition:expr, $format:expr, $($arguments:expr),*) => {
        if $condition { bail!($format$(, $arguments)*) }
    }
}

macro_rules! veccat {
    ($a:expr, $b:expr) => { $a.into_iter().chain($b.into_iter()).collect() }
}

trait Pairable<T, I> where T: Copy + Default {
    fn pairs(self) -> PairIterator<T, I>;
}

impl<T, I> Pairable<T, I> for I where I: Iterator<Item=T>, T: Copy + Default {
    fn pairs(self) -> PairIterator<T, I> {
        PairIterator { previous: T::default(), iter: self }
    }
}

struct PairIterator<T, I> {
    previous: T,
    iter: I,
}

impl<T, I> Iterator for PairIterator<T, I> where I: Iterator<Item=T>, T: Copy {
    type Item = (T, T);
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next().map(|current| (self.previous, current));
        if let Some((_, current)) = &next {
            self.previous = *current;
        }
        next
    }
}

pub struct Heap(Vec<HeapObject>);
impl Heap {
    pub fn new() -> Self {
        Heap(Vec::new())
    }
    pub fn allocate(&mut self, object: HeapObject) -> HeapIndex {
        let index = HeapIndex::from(self.0.len());
        self.0.push(object);
        index
    }
    pub fn dereference(&self, index: &HeapIndex) -> Result<&HeapObject> {
        self.0.get(index.as_usize())
            .with_context(||
                format!("Cannot dereference object from the heap at index: `{}`", index))
    }
    pub fn dereference_mut(&mut self, index: &HeapIndex) -> Result<&mut HeapObject> {
        self.0.get_mut(index.as_usize())
            .with_context(||
                   format!("Cannot dereference object from the heap at index: `{}`", index))
    }
}
impl From<Vec<HeapObject>> for Heap {
    fn from(objects: Vec<HeapObject>) -> Self { Heap(objects) }
}

pub struct InstructionPointer {}
impl InstructionPointer {
    pub fn bump(&self, program: &Program) { unimplemented!() }
    pub fn set(&self, address: Option<Address>) { unimplemented!() }
    pub fn get(&self) -> Option<Address> { unimplemented!() }
}

pub struct OperandStack(Vec<Pointer>);
impl OperandStack {
    pub fn push(&mut self, pointer: Pointer) {
        self.0.push(pointer)
    }
    pub fn pop(&mut self) -> Result<Pointer> {
        self.0.pop().with_context(|| format!("Cannot pop from an empty operand stack."))
    }
    pub fn peek(&self) -> Result<&Pointer> {
        self.0.last().with_context(|| format!("Cannot peek from an empty operand stack."))
    }
}

pub struct ConstantPool(Vec<ProgramObject>);
impl ConstantPool {
    pub fn get(&self, index: &ConstantPoolIndex) -> Result<&ProgramObject> {
        self.0.get(index.as_usize())
            .with_context(||
                format!("Cannot dereference object from the constant pool at index: `{}`", index))
    }
}

pub struct Frame { return_address: Option<Address>, locals: Vec<Pointer> }
impl Frame {
    pub fn new(return_address: Option<Address>, locals: Vec<Pointer>) -> Self {
        Frame { locals: Vec::new(), return_address: None }
    }
    pub fn get(&self, index: &LocalFrameIndex) -> Result<&Pointer> { unimplemented!() }
    pub fn set(&mut self, index: &LocalFrameIndex, pointer: Pointer) -> Result<()> { unimplemented!() }
}

pub struct Dictionary<T>(HashMap<String, T>);
impl<T> Dictionary<T> {
    pub fn get(&self, name: &str) -> Result<&T> { unimplemented!() }
    pub fn set(&mut self, name: String, pointer: T) -> Result<()> { unimplemented!() }
}

pub struct FrameStack { globals: Dictionary<Pointer>, functions: Dictionary<ConstantPoolIndex> }
impl FrameStack {
    pub fn pop(&mut self) -> Result<Frame> { unimplemented!() }
    pub fn push(&mut self, frame: Frame) { unimplemented!() }
    pub fn get_locals(&self) -> Result<&Frame> { unimplemented!() }
    pub fn get_locals_mut(&mut self) -> Result<&mut Frame> { unimplemented!() }
}

pub struct State {
    operand_stack: OperandStack,
    frame_stack: FrameStack,
    instruction_pointer: InstructionPointer,
    heap: Heap
}

pub struct Program {
    constant_pool: ConstantPool,
    labels: Dictionary<Address>
}

trait OpCodeEvaluationResult<T> {
    #[inline(always)]
    fn attach(self, opcode: &OpCode) -> Result<T>;
}

impl<T> OpCodeEvaluationResult<T> for Result<T> {
    #[inline(always)]
    fn attach(self, opcode: &OpCode) -> Result<T> {
        self.with_context(|| format!("Error evaluating {}:", opcode))
    }
}

pub fn eval_opcode<W>(program: &Program, state: &mut State, output: &mut W, opcode: &OpCode) -> Result<()> where W: Write {
    match opcode {
        OpCode::Literal { index } => eval_literal(program, state, index),
        OpCode::GetLocal { index } => eval_get_local(program, state, index),
        OpCode::SetLocal { index } => eval_set_local(program, state, index),
        OpCode::GetGlobal { name } => eval_get_global(program, state, name),
        OpCode::SetGlobal { name } => eval_set_global(program, state, name),
        OpCode::Object { class } => eval_object(program, state, class),
        OpCode::Array => eval_array(program, state),
        OpCode::GetField { name } => eval_get_field(program, state, name),
        OpCode::SetField { name } => eval_set_field(program, state, name),
        OpCode::CallMethod { name, arguments } => eval_call_method(program, state, name, arguments),
        OpCode::CallFunction { name, arguments } => eval_call_function(program, state, name, arguments),
        OpCode::Label { .. } => eval_label(program, state),
        OpCode::Print { format, arguments } => eval_print(program, state, output, format, arguments),
        OpCode::Jump { label } => eval_jump(program, state, label),
        OpCode::Branch { label } => eval_branch(program, state, label),
        OpCode::Return => eval_return(program, state),
        OpCode::Drop => eval_drop(program, state),
    }.attach(opcode)
}

#[inline(always)]
pub fn eval_literal(program: &Program, state: &mut State, index: &ConstantPoolIndex) -> Result<()> {
    let program_object = program.constant_pool.get(index)?;
    let pointer = Pointer::from_literal(program_object)?;
    state.operand_stack.push(pointer);
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_get_local(program: &Program, state: &mut State, index: &LocalFrameIndex) -> Result<()> { // TODO rename LocalFrameIndex to FrameIndex
    let frame = state.frame_stack.get_locals()?;
    let pointer = *frame.get(index)?;
    state.operand_stack.push(pointer);
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_set_local(program: &Program, state: &mut State, index: &LocalFrameIndex) -> Result<()> {
    let pointer = *state.operand_stack.peek()?;
    let frame = state.frame_stack.get_locals_mut()?;
    frame.set(index, pointer)?;
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_get_global(program: &Program, state: &mut State, index: &ConstantPoolIndex) -> Result<()> {
    let program_object = program.constant_pool.get(index)?;
    let name = program_object.as_str()?;
    let pointer = *state.frame_stack.globals.get(name)?;
    state.operand_stack.push(pointer);
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_set_global(program: &Program, state: &mut State, index: &ConstantPoolIndex) -> Result<()> {
    let program_object = program.constant_pool.get(index)?;
    let name = program_object.as_str()?.to_owned();
    let pointer = *state.operand_stack.peek()?;
    state.frame_stack.globals.set(name, pointer)?;
    state.operand_stack.push(pointer);
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_object(program: &Program, state: &mut State, index: &ConstantPoolIndex) -> Result<()> {
    let program_object = program.constant_pool.get(index)?;
    let members = program_object.as_class_definition()?.iter()
        .map(| index | program.constant_pool.get(index))
        .collect::<Result<Vec<&ProgramObject>>>()?;

    let mut slots = Vec::new();
    let mut methods = HashMap::new();


    for member in members {                                                           // TODO this could probably be a method in ProgramObject, something like: `create prototype object om class`
        match member {
            ProgramObject::Slot { name: index } => {
                let program_object = program.constant_pool.get(index)?;
                let name = program_object.as_str()?;
                slots.push(name);
            }
            ProgramObject::Method { name: index, .. } => {                         // TODO, probably don't need to store methods, tbh, just the class, which would simplify this a lot
                let program_object = program.constant_pool.get(index)?;
                let name = program_object.as_str()?.to_owned();
                methods.insert(name.clone(), member.clone())
                    .with_context(|| format!("Member method `{}` has a non-unique name within object.", name))?;
            }
            _ => bail!("Class members must be either Methods or Slots, but found `{}`.", member)
        }
    }

    let mut fields = HashMap::new();
    for name in slots.into_iter().rev() {
        let pointer = state.operand_stack.pop()?;
        fields.insert(name.to_owned(), pointer)
            .with_context(|| format!("Member field `{}` has a non-unique name in object", name))?;
    }

    let parent = state.operand_stack.pop()?;

    let heap_index = state.heap.allocate(HeapObject::Object(ObjectInstance { parent, fields, methods })); // TODO simplify
    state.operand_stack.push(Pointer::from(heap_index));
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_array(program: &Program, state: &mut State) -> Result<()> {
    let initializer = state.operand_stack.pop()?;
    let size = state.operand_stack.pop()?;

    let n = size.as_i32()?;
    bail_if!(n < 0, "Negative value `{}` cannot be used to specify the size of an array.", n);

    let elements = repeat(initializer).take(n as usize).collect();
    let array = HeapObject::from_pointers(elements);

    let heap_index = state.heap.allocate(array);
    state.operand_stack.push(Pointer::from(heap_index));
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_get_field(program: &Program, state: &mut State, index: &ConstantPoolIndex) -> Result<()> {
    let program_object = program.constant_pool.get(index)?;
    let name = program_object.as_str()?;
    let pointer = state.operand_stack.pop()?;
    let heap_pointer = pointer.into_heap_reference()?;
    let object = state.heap.dereference(&heap_pointer)?;

    let object_instance = object.as_object_instance()?;
    let pointer = object_instance.get_field(name)?;
    state.operand_stack.push(*pointer);
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_set_field(program: &Program, state: &mut State, index: &ConstantPoolIndex) -> Result<()> {
    let program_object = program.constant_pool.get(index)?;
    let name = program_object.as_str()?;
    let value_pointer = state.operand_stack.pop()?;
    let object_pointer = state.operand_stack.pop()?;
    let heap_pointer = object_pointer.into_heap_reference()?;
    let object = state.heap.dereference_mut(&heap_pointer)?;

    let object_instance = object.as_object_instance_mut()?;
    let pointer = object_instance.set_field(name, value_pointer.clone())?;
    state.operand_stack.push(value_pointer);
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_call_method(program: &Program, state: &mut State, index: &ConstantPoolIndex, arguments: &Arity) -> Result<()> {
    bail_if!(arguments.to_usize() == 0, "All method calls require at least {} parameter (receiver)", 1);

    let program_object = program.constant_pool.get(index)?;
    let name = program_object.as_str()?;

    let argument_pointers = (0..arguments.to_usize()).map(|_| state.operand_stack.pop()).rev().collect::<Result<Vec<Pointer>>>()?;
    let receiver_pointer = state.operand_stack.pop()?;



    //let local_pointers = repeat(Pointer::Null).take(locals.to_usize()).collect::<Vec<Pointer>>();

    Ok(())
}

pub fn dispatch_method(heap: &Heap, receiver_pointer: &Pointer, method_name: &str, argument_pointers: &Vec<Pointer>) -> Result<Pointer> {
    match receiver_pointer {
        Pointer::Null => dispatch_null_method(method_name, argument_pointers),
        Pointer::Integer(receiver) => dispatch_integer_method(receiver, method_name, argument_pointers),
        Pointer::Boolean(receiver) => dispatch_boolean_method(receiver, method_name, argument_pointers),
        Pointer::Reference(index) => {
            let heap_object = heap.dereference(index)?;
            match heap_object {
                HeapObject::Array(array) => unimplemented!(),
                HeapObject::Object(instance) => unimplemented!(),
            }
        }
    }
}

pub fn dispatch_null_method(method_name: &str, argument_pointers: &Vec<Pointer>) -> Result<Pointer> {
    bail_if!(argument_pointers.len() != 1,
             "Invalid number of arguments for method `{}` in object `null`", method_name);

    let argument = argument_pointers.last().unwrap();
    let result = match (method_name, argument)  {
        ("==", Pointer::Null) | ("eq", Pointer::Null)  => Pointer::from(true),
        ("==", _) | ("eq", _)                          => Pointer::from(false),
        ("!=", Pointer::Null) | ("neq", Pointer::Null) => Pointer::from(true),
        ("!=", _) | ("neq", _)                         => Pointer::from(false),
        _ => bail!("Call method error: no method `{}` in object `null`", method_name),
    };

    Ok(result)
}

pub fn dispatch_integer_method(receiver: &i32, method_name: &str, argument_pointers: &Vec<Pointer>) -> Result<Pointer> {
    bail_if!(argument_pointers.len() != 1,
             "Invalid number of arguments for method `{}` in object `{}`", method_name, receiver);

    let argument_pointer = argument_pointers.last().unwrap();

    let result = match (method_name, argument_pointer) {
        ("+",  Pointer::Integer(argument)) => Pointer::from(receiver +  argument),
        ("-",  Pointer::Integer(argument)) => Pointer::from(receiver -  argument),
        ("*",  Pointer::Integer(argument)) => Pointer::from(receiver *  argument),
        ("/",  Pointer::Integer(argument)) => Pointer::from(receiver /  argument),
        ("%",  Pointer::Integer(argument)) => Pointer::from(receiver %  argument),
        ("<=", Pointer::Integer(argument)) => Pointer::from(receiver <= argument),
        (">=", Pointer::Integer(argument)) => Pointer::from(receiver >= argument),
        ("<",  Pointer::Integer(argument)) => Pointer::from(receiver <  argument),
        (">",  Pointer::Integer(argument)) => Pointer::from(receiver >  argument),
        ("==", Pointer::Integer(argument)) => Pointer::from(receiver == argument),
        ("!=", Pointer::Integer(argument)) => Pointer::from(receiver != argument),
        ("==", _) => Pointer::from(false),
        ("!=", _) => Pointer::from(true),

        ("add", Pointer::Integer(argument)) => Pointer::from(receiver +  argument),
        ("sub", Pointer::Integer(argument)) => Pointer::from(receiver -  argument),
        ("mul", Pointer::Integer(argument)) => Pointer::from(receiver *  argument),
        ("div", Pointer::Integer(argument)) => Pointer::from(receiver /  argument),
        ("mod", Pointer::Integer(argument)) => Pointer::from(receiver %  argument),
        ("le",  Pointer::Integer(argument)) => Pointer::from(receiver <= argument),
        ("ge",  Pointer::Integer(argument)) => Pointer::from(receiver >= argument),
        ("lt",  Pointer::Integer(argument)) => Pointer::from(receiver <  argument),
        ("gt",  Pointer::Integer(argument)) => Pointer::from(receiver >  argument),
        ("eq",  Pointer::Integer(argument)) => Pointer::from(receiver == argument),
        ("neq", Pointer::Integer(argument)) => Pointer::from(receiver != argument),
        ("eq", _) => Pointer::from(false),
        ("neq", _) => Pointer::from(true),

        _ => bail!("Call method error: no method `{}` in object `{}`", method_name, receiver),
    };

    Ok(result)
}

pub fn dispatch_boolean_method(receiver: &bool, method_name: &str, argument_pointers: &Vec<Pointer>) -> Result<Pointer> {
    bail_if!(argument_pointers.len() != 1,
             "Invalid number of arguments for method `{}` in object `{}`", method_name, receiver);

    let argument_pointer = argument_pointers.last().unwrap();

    let result = match (method_name, argument_pointer) {
        ("&",  Pointer::Boolean(argument)) => Pointer::from(*receiver && *argument),
        ("|",  Pointer::Boolean(argument)) => Pointer::from(*receiver || *argument),
        ("==", Pointer::Boolean(argument)) => Pointer::from(*receiver == *argument),
        ("!=", Pointer::Boolean(argument)) => Pointer::from(*receiver != *argument),
        ("==", _) => Pointer::from(false),
        ("!=", _) => Pointer::from(true),

        ("and", Pointer::Boolean(argument)) => Pointer::from(*receiver && *argument),
        ("or",  Pointer::Boolean(argument)) => Pointer::from(*receiver || *argument),
        ("eq",  Pointer::Boolean(argument)) => Pointer::from(*receiver == *argument),
        ("neq", Pointer::Boolean(argument)) => Pointer::from(*receiver != *argument),
        ("eq",  _) => Pointer::from(false),
        ("neq", _) => Pointer::from(true),

        _ => bail!("Call method error: no method `{}` in object `{}`",  method_name, receiver),
    };

    Ok(result)
}

#[inline(always)]
pub fn eval_call_function(program: &Program, state: &mut State, index: &ConstantPoolIndex, arguments: &Arity) -> Result<()> {
    let program_object = program.constant_pool.get(index)?;
    let name = program_object.as_str()?;
    let function_index = state.frame_stack.functions.get(name)?;
    let function = program.constant_pool.get(function_index)?;

    let parameters = function.get_method_parameters()?;                                      // FIXME perhaps the thing to do here is to have a Method struct inside the ProgramObject::Method constructor
    let locals = function.get_method_locals()?;
    let address = function.get_method_start_address()?;

    bail_if!(arguments != parameters, "Function `{}` requires {} arguments, but {} were supplied", name, parameters, arguments);

    let argument_pointers = (0..arguments.to_usize()).map(|_| state.operand_stack.pop()).rev().collect::<Result<Vec<Pointer>>>()?; // TODO Probably should be a function
    let local_pointers = repeat(Pointer::Null).take(locals.to_usize()).collect::<Vec<Pointer>>(); // TODO Maybe this should be a function too

    state.instruction_pointer.bump(program);
    let frame = Frame::new(state.instruction_pointer.get(), veccat!(argument_pointers, local_pointers));
    state.frame_stack.push(frame);
    state.instruction_pointer.set(Some(*address));
    Ok(())
}

#[inline(always)]
pub fn eval_print<W>(program: &Program, state: &mut State, output: &mut W, index: &ConstantPoolIndex, arguments: &Arity) -> Result<()> where W: Write {
    let program_object = program.constant_pool.get(index)?;
    let format = program_object.as_str()?;
    let mut argument_pointers = (0..arguments.to_usize()).map(|_| state.operand_stack.pop()).collect::<Result<Vec<Pointer>>>()?;

    for (previous, character) in format.chars().pairs() {
        match (previous, character) {
            ('\\', '~' ) => output.write_char('~')?,
            ('\\', '\\') => output.write_char('\\')?,
            ('\\', '"' ) => output.write_char('"')?,
            ('\\', 'n' ) => output.write_char('\n')?,
            ('\\', 't' ) => output.write_char('\t')?,
            ('\\', 'r' ) => output.write_char('\r')?,
            ('\\', ch  )  => bail!("Unknown control sequence \\{}", ch),
            (_,    '\\') => {},
            (_,    '~' ) => {
                let argument = argument_pointers.pop()
                    .with_context(|| "Not enough arguments for format `{}`")?;
                output.write_str(argument.evaluate_as_string(/*state.heap*/).as_str())?
            },
            (_,    ch  ) => output.write_char(ch)?,
        }
    }
    bail_if!(!argument_pointers.is_empty(),
             "{} unused arguments for format `{}`", argument_pointers.len(), format);

    state.operand_stack.push(Pointer::Null);
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_label(program: &Program, state: &mut State) -> Result<()> {
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_jump(program: &Program, state: &mut State, index: &ConstantPoolIndex) -> Result<()> {
    let program_object = program.constant_pool.get(index)?;
    let name = program_object.as_str()?;
    let address = *program.labels.get(name)?;
    state.instruction_pointer.set(Some(address));
    Ok(())
}

#[inline(always)]
pub fn eval_branch(program: &Program, state: &mut State, index: &ConstantPoolIndex) -> Result<()> {
    let program_object = program.constant_pool.get(index)?;
    let name = program_object.as_str()?;
    let pointer = state.operand_stack.pop()?;
    if !pointer.evaluate_as_condition() {
        state.instruction_pointer.bump(program);
    } else {
        let address = *program.labels.get(name)?;
        state.instruction_pointer.set(Some(address));
    }
    Ok(())
}

#[inline(always)]
pub fn eval_return(_program: &Program, state: &mut State) -> Result<()> {
    let frame = state.frame_stack.pop()?;
    state.instruction_pointer.set(frame.return_address);
    Ok(())
}

#[inline(always)]
pub fn eval_drop(_program: &Program, state: &mut State) -> Result<()> {
    state.operand_stack.pop()?;
    Ok(())
}


