use crate::parser::AST;

pub(crate) mod bytecode;
pub(crate) mod compiler;
pub(crate) mod debug;
pub(crate) mod objects;
pub mod program;
pub mod serializable;
pub(crate) mod types;
pub mod interpreter;

pub fn compile(ast: &AST) -> program::Program {
    compiler::compile(ast)
}