use std::io::Write;

use super::types::*;

use super::objects::ProgramObject;
use super::bytecode::OpCode;
use super::program::{Program, Code};

pub trait PrettyPrint: UglyPrint {
    fn pretty_print<W: Write>(&self, sink: &mut W) {
        self.ugly_print(sink, 0, false);
    }
    fn pretty_print_indent<W: Write>(&self, sink: &mut W, indent: usize) {
        self.ugly_print(sink, indent, true);
    }
    fn pretty_print_no_indent<W: Write>(&self, sink: &mut W) {
        self.ugly_print(sink, 0, false);
    }
    fn pretty_print_no_first_line_indent<W: Write>(&self, sink: &mut W, indent: usize) {
        self.ugly_print(sink, indent, false);
    }
}

pub trait PrettyPrintWithContext: UglyPrintWithContext {
    fn pretty_print<W: Write>(&self, sink: &mut W, code: &Code) {
        self.ugly_print(sink, code, 0, false);
    }
    fn pretty_print_indent<W: Write>(&self, sink: &mut W, code: &Code, indent: usize) {
        self.ugly_print(sink, code, indent, true);
    }
    fn pretty_print_no_indent<W: Write>(&self, sink: &mut W, code: &Code) {
        self.ugly_print(sink, code, 0, false);
    }
    fn pretty_print_no_first_line_indent<W: Write>(&self, sink: &mut W, code: &Code, indent: usize) {
        self.ugly_print(sink, code, indent, false);
    }
}

impl PrettyPrint for ConstantPoolIndex {}
impl PrettyPrint for LocalFrameIndex {}
impl PrettyPrint for Address {}
impl PrettyPrint for Size {}
impl PrettyPrint for Arity {}
impl PrettyPrint for OpCode {}
impl PrettyPrint for Program {}
impl PrettyPrintWithContext for ProgramObject {}

pub trait UglyPrint {
    fn ugly_print<W: Write>(&self, sink: &mut W, indent: usize, indent_first_line: bool);
}

pub trait UglyPrintWithContext {
    fn ugly_print<W: Write>(&self, sink: &mut W, code: &Code, indent: usize, indent_first_line: bool);
}

macro_rules! write_string {
    ($sink: expr, $indent: expr, $value: expr) => {
        if ($indent == 0) {
            $sink.write_all(format!("{}", $value).as_bytes()).unwrap()
        } else {
            $sink.write_all(format!("{:indent$}{}"," ", $value, indent=$indent).as_bytes()).unwrap()
        }
    };
    ($sink: expr, $indent: expr, $fmt: expr, $value: expr) => {
        if ($indent == 0) {
            $sink.write_all(format!($fmt, $value).as_bytes()).unwrap()
        } else {
            $sink.write_all(format!("{:indent$}{}"," ", format!($fmt, $value),
                                                        indent=$indent).as_bytes()).unwrap()
        }
    }
}

macro_rules! in_margin {
    ($indent: expr) => {
        if $indent == 0 {
            $indent
        } else {
            $indent - 3
        }
    }
}

macro_rules! further {
    ($indent: expr) => {$indent + 4}
}

macro_rules! big_further {
    ($indent: expr) => {$indent + 6}
}


macro_rules! first {
    ($indent: expr, $condition: expr) => {if $condition {$indent} else {0}}
}

impl UglyPrint for ConstantPoolIndex {
    fn ugly_print<W: Write>(&self, sink: &mut W, indent: usize, prefix_first_line: bool) {
        write_string!(sink, first!(indent, prefix_first_line), "#{}", self.value())
    }
}

impl UglyPrint for LocalFrameIndex {
    fn ugly_print<W: Write>(&self, sink: &mut W, indent: usize, prefix_first_line: bool) {
        write_string!(sink, first!(indent, prefix_first_line), self.value())
    }
}

impl UglyPrint for Size {
    fn ugly_print<W: Write>(&self, sink: &mut W, indent: usize, prefix_first_line: bool) {
        write_string!(sink, first!(indent, prefix_first_line), self.value().to_string())
    }
}

impl UglyPrint for Arity {
    fn ugly_print<W: Write>(&self, sink: &mut W, indent: usize, prefix_first_line: bool) {
        write_string!(sink, first!(indent, prefix_first_line), self.value().to_string())
    }
}

impl UglyPrint for Address {
    fn ugly_print<W: Write>(&self, sink: &mut W, indent: usize, prefix_first_line: bool) {
        write_string!(sink, first!(indent, prefix_first_line), "0x{:X}", self.value_u32());
    }
}

impl UglyPrintWithContext for ProgramObject {
    fn ugly_print<W: Write>(&self, sink: &mut W, code: &Code, indent: usize, prefix_first_line: bool) {
        match self {
            ProgramObject::Null  =>
                write_string!(sink, first!(indent, prefix_first_line), "Null"),

            ProgramObject::Integer(value) =>
                write_string!(sink, first!(indent, prefix_first_line), "Int({})", value),

            ProgramObject::Boolean(value) =>
                write_string!(sink, first!(indent, prefix_first_line), "Bool({})", value),

            ProgramObject::String(value) => {
                let string = value.escape_default().to_string();
                write_string!(sink, first!(indent, prefix_first_line), "String(\"{}\")", string)
            }

            ProgramObject::Slot {name} => {
                write_string!(sink, first!(indent, prefix_first_line), "Slot(");
                name.pretty_print_no_indent(sink);
                write_string!(sink, 0, ")");
            },

            ProgramObject::Class(slots) => {
                write_string!(sink, first!(indent, prefix_first_line), "Class(");
                let mut first = true;
                for slot in slots {
                    if !first { write_string!(sink, 0, ", "); } else { first = false; }
                    slot.pretty_print(sink)
                }
                write_string!(sink, 0, ")");
            },

            ProgramObject::Method {name, arguments, locals, code: range} => {
                write_string!(sink, first!(indent, prefix_first_line), "Method(");
                name.pretty_print_no_indent(sink);
                write_string!(sink, 0, ", nargs:");
                arguments.pretty_print_no_indent(sink);
                write_string!(sink, 0, ", nlocals:");
                locals.pretty_print_no_indent(sink);
                write_string!(sink, 0, ") :");

                for opcode in code.addresses_to_code_vector(range) {
                    write_string!(sink, 0, "\n");
                    //println!("indent {:?} {} -> {} ", self, indent, further!(indent));
                    opcode.pretty_print_indent(sink, further!(indent))
                }
            },
        }
    }
}

impl UglyPrint for OpCode {
    fn ugly_print<W: Write>(&self, sink: &mut W, indent: usize, _prefix_first_line: bool) {
        match self {
            OpCode::Literal { index } => {
                write_string!(sink, indent, "lit ");
                index.pretty_print_no_indent(sink);
            },
            OpCode::GetLocal { index } => {
                write_string!(sink, indent, "get local ");
                index.pretty_print_no_indent(sink);
            },
            OpCode::SetLocal { index } => {
                write_string!(sink, indent, "set local ");
                index.pretty_print_no_indent(sink);
            },
            OpCode::GetGlobal { name } => {
                write_string!(sink, indent, "get global ");
                name.pretty_print_no_indent(sink);
            },
            OpCode::SetGlobal { name } => {
                write_string!(sink, indent, "set global ");
                name.pretty_print_no_indent(sink);
            },
            OpCode::Object { class } => {
                write_string!(sink, indent, "object ");
                class.pretty_print_no_indent(sink);
            },
            OpCode::Array => {
                write_string!(sink, indent, "array");
            },
            OpCode::GetField { name } => {
                write_string!(sink, indent, "get slot ");
                name.pretty_print_no_indent(sink);
            },
            OpCode::SetField { name } => {
                write_string!(sink, indent, "set slot ");
                name.pretty_print_no_indent(sink);
            },
            OpCode::CallMethod { name, arguments } => {
                write_string!(sink, indent, "call slot ");
                name.pretty_print_no_indent(sink);
                arguments.pretty_print_indent(sink, 1);
            },
            OpCode::CallFunction { name, arguments } => {
                write_string!(sink, indent, "call ");
                name.pretty_print_no_indent(sink);
                arguments.pretty_print_indent(sink, 1);
            },
            OpCode::Print { format, arguments } => {
                write_string!(sink, indent, "printf ");
                format.pretty_print_no_indent(sink);
                arguments.pretty_print_indent(sink, 1);
            },
            OpCode::Label { name } => {
                write_string!(sink, in_margin!(indent), "label ");
                name.pretty_print_no_indent(sink);
            },
            OpCode::Jump { label } => {
                write_string!(sink, indent, "goto ");
                label.pretty_print_no_indent(sink);
            },
            OpCode::Branch { label } => {
                write_string!(sink, indent, "branch ");
                label.pretty_print_no_indent(sink);
            },
            OpCode::Return => {
                write_string!(sink, indent, "return");
            },
            OpCode::Drop => {
                write_string!(sink, indent, "drop");
            },
        }
    }
}

impl UglyPrint for Program {
    fn ugly_print<W: Write>(&self, sink: &mut W, indent: usize, _prefix_first_line: bool) {
        write_string!(sink, indent, "Constants :\n");
        for (index, opcode) in self.constants().iter().enumerate() {
            ConstantPoolIndex::new(index as u16).pretty_print_indent(sink, further!(indent));
            write_string!(sink, 0, ": ");
            opcode.pretty_print_no_first_line_indent(sink, self.code(), big_further!(indent));
            write_string!(sink, 0, "\n");
        }
        write_string!(sink, indent, "Globals :\n");
        for global in self.globals().iter() {
            global.pretty_print_indent(sink, further!(indent));
            write_string!(sink, 0, "\n");
        }
        write_string!(sink, indent, "Entry : ");
        self.entry().pretty_print_no_indent(sink);
    }
}
