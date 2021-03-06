use crate::parser::AST;

pub(crate) mod bytecode;
pub(crate) mod compiler;
pub(crate) mod debug;
pub mod program;
pub mod serializable;
pub mod interpreter;
#[macro_use] mod helpers;
pub mod heap;
pub mod state;

use anyhow::Result;

pub fn compile(ast: &AST) -> Result<program::Program> {
    compiler::compile(ast)
}