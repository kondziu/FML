use crate::parser::AST;

pub(crate) mod bytecode;
pub(crate) mod compiler;
pub(crate) mod debug;
pub mod program;
pub mod serializable;
pub mod interpreter;
mod interp;
#[macro_use] mod helpers;
pub mod heap;
pub mod state;

pub fn compile(ast: &AST) -> program::Program {
    compiler::compile(ast)
}