use crate::parser::AST;

mod bytecode;
mod compiler;
mod debug;
mod objects;
pub mod program;
pub mod serializable;
mod types;
pub mod interpreter;

pub fn compile(ast: &AST) -> program::Program {
    compiler::compile(ast)
}