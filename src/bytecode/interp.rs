use crate::bytecode::types::{ConstantPoolIndex, LocalFrameIndex};

use std::fmt::Write;

use crate::bytecode::bytecode::OpCode;
use crate::bytecode::objects::{ProgramObject, Pointer, HeapObject};

use anyhow::*;
use anyhow::Context;
use std::collections::HashMap;

pub struct Heap {}
impl Heap {
    pub fn allocate(&mut self, object: HeapObject) -> Result<Pointer> { unimplemented!() }
}

pub struct InstructionPointer {}
impl InstructionPointer {
    pub fn bump(&self, program: &Program) { unimplemented!() }
}

pub struct OperandStack {}
impl OperandStack {
    pub fn push(&mut self, pointer: Pointer) { unimplemented!() }
    pub fn pop(&mut self) -> Result<Pointer> { unimplemented!() }
    pub fn peek(&self) -> Result<&Pointer> { unimplemented!() }
}

pub struct ConstantPool {}
impl ConstantPool {
    pub fn get(&self, index: &ConstantPoolIndex) -> Result<&ProgramObject> { unimplemented!() }
}

pub struct Frame {}
impl Frame {
    pub fn get(&self, index: &LocalFrameIndex) -> Result<&Pointer> { unimplemented!() }
    pub fn set(&mut self, index: &LocalFrameIndex, pointer: Pointer) -> Result<()> { unimplemented!() }
}

pub struct Dictionary {}
impl Dictionary {
    pub fn get(&self, name: &str) -> Result<&Pointer> { unimplemented!() }
    pub fn set(&mut self, name: String, pointer: Pointer) -> Result<()> { unimplemented!() }
}

pub struct FrameStack {}
impl FrameStack {
    pub fn get_locals(&self) -> Result<&Frame> { unimplemented!() }
    pub fn get_locals_mut(&mut self) -> Result<&mut Frame> { unimplemented!() }
    pub fn get_globals(&self) -> Result<&Dictionary> { unimplemented!() }
    pub fn get_globals_mut(&mut self) -> Result<&mut Dictionary> { unimplemented!() }
}

pub struct State { operand_stack: OperandStack, frame_stack: FrameStack, instruction_pointer: InstructionPointer, heap: Heap }
pub struct Program { constant_pool: ConstantPool }

trait OpCodeEvaluationResult<T> {
    #[inline(always)]
    fn check(self, opcode: &OpCode) -> Result<T>;
}

impl<T> OpCodeEvaluationResult<T> for Result<T> {
    #[inline(always)]
    fn check(self, opcode: &OpCode) -> Result<T> {
        self.with_context(|| format!("Error evaluating {}:", opcode))
    }
}

pub fn eval_opcode<Output>(program: &Program, state: &mut State, opcode: &OpCode) -> Result<()> where Output : Write {
    match opcode {
        OpCode::Literal { index } => eval_literal(program, state, opcode, index).check(opcode),
        _ => unimplemented!()
    }
}

#[inline(always)]
pub fn eval_literal(program: &Program, state: &mut State, opcode: &OpCode, index: &ConstantPoolIndex) -> Result<()> {
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
    let globals = state.frame_stack.get_globals()?;
    let pointer = *globals.get(name)?;
    state.operand_stack.push(pointer);
    state.instruction_pointer.bump(program);
    Ok(())
}

#[inline(always)]
pub fn eval_set_global(program: &Program, state: &mut State, index: &ConstantPoolIndex) -> Result<()> {
    let program_object = program.constant_pool.get(index)?;
    let name = program_object.as_str()?.to_owned();
    let globals = state.frame_stack.get_globals_mut()?;
    let pointer = *state.operand_stack.peek()?;
    globals.set(name, pointer)?;
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

    for member in members {
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
                    .with_context(|| format!("Member method `{}` has a non-unique name within object", name));
            }
            _ => bail!("Class members must be either Methods or Slots, but found `{}`", member)
        }
    }

    let mut fields = HashMap::new();
    for name in slots.into_iter().rev() {
        let pointer = state.operand_stack.pop()?;
        fields.insert(name.to_owned(), pointer)
            .with_context(|| format!("Member field `{}` has a non-unique name in object", name));
    }

    let parent = state.operand_stack.pop()?;

    let pointer = state.heap.allocate(HeapObject::Object { parent, fields, methods })?;
    state.operand_stack.push(pointer);
    state.instruction_pointer.bump(program);

    Ok(())
}

