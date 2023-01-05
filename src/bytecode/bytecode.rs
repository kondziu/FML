use std::io::{Read, Write};

use super::serializable;
use super::serializable::*;
use crate::bytecode::program::*;

/**
 * # Bytecode operation
 *
 */
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum OpCode {
    /**
     * ## Push literal onto stack
     *
     * Retrieves the [ProgramObject] at the given `index` from the [ConstantPool] and pushes it onto
     * the [OperandStack].
     *
     * The [ProgramObject] retrieved from the [ConstantPool] is *guaranteed* to be one of:
     *  - [ProgramObject::Integer],
     *  - [ProgramObject::Boolean], or
     *  - [ProgramObject::Null].
     *
     * Serialized as opcode `0x01`.
     *
     * [ConstantPool]: ../interpreter/struct.ConstantPool.html
     * [OperandStack]: ../interpreter/struct.OperandStack.html
     * [ProgramObject]: ../objects/enum.ProgramObject.html
     * [ProgramObject::Boolean]: ../objects/enum.ProgramObject.html#variant.Boolean
     * [ProgramObject::Integer]: ../objects/enum.ProgramObject.html#variant.Integer
     * [ProgramObject::Null]: ../objects/enum.ProgramObject.html#variant.Null
     */
    Literal { index: ConstantPoolIndex }, // rename to constant

    /**
     * ## Push the value of local variable onto stack
     *
     * Retrieves a slot in the current [LocalFrame] at the given index and pushes it onto the
     * [OperandStack].
     *
     * Serialized as opcode `0x0A`.
     *
     * [LocalFrame]: ../interpreter/struct.LocalFrame.html
     * [OperandStack]: ../interpreter/struct.OperandStack.html
     */
    // FIXME writing out all those links in each variant is not sustainable...
    GetLocal { index: LocalFrameIndex },

    /**
     * ## Set the value local variable to top value from stack
     *
     * Sets the slot in the current `LocalFrame` at the given index to the top value in the
     * `OperandStack`.
     *
     * Serialized as opcode `0x09`.
     */
    SetLocal { index: LocalFrameIndex },

    /**
     * ## Push the value of global variable onto stack
     *
     * Retrieves the value of the global variable with name specified by the `ProgramObject::String`
     * object at the given index and pushes it onto the `OperandStack`.
     *
     * Serialized as opcode `0x0C`.
     */
    GetGlobal { name: ConstantPoolIndex },

    /**
     * ## Set the value of global variable to the top value from stack
     *
     * Sets the global variable with the name specified by the `ProgramObject::String` object at the
     * given index to the top value in the `OperandStack`.
     *
     * Serialized as opcode `0x0B`.
     */
    SetGlobal { name: ConstantPoolIndex },

    /**
     * ## Create a new (runtime) object
     *
     * Retrieves the `ProgramObject::Class` object at the given index.Suppose that the
     * `ProgramObject::Class` object contains n `ProgramObject::Slot` objects and m
     * `ProgramObject::Method` objects. This instruction will pop n values from the `OperandStack`
     * for use as initial values of the variable slots in the object, then an additional value for
     * use as the parent of the object.
     *
     * The first variable slot is initialized to the deepest value on the `OperandStack` (last
     * popped). The last variable slot is initialized to the shallowest value on the `OperandStack`
     * (first popped).
     *
     * A new `RuntimeObject` is created with the variable slots and method slots indicated
     * by the Class object with the given parent object. The `RuntimeObject` is pushed onto the
     * `OperandStack`.
     *
     * Serialized as opcode `0x04`.
     */
    Object { class: ConstantPoolIndex },

    /**
     * ## Create a new array (runtime) object
     *
     * First pops the initializing value from the `OperandStack`. Then pops the `size` of the array
     * from the `OperandStack`. Creates a new array with the given `size`, with each element
     * initialized to the initializing value. Then, pushes the array onto the `OperandStack`.
     *
     * **Warning**: this is different from the semantics of the `array` operation in FML, which
     * evaluates the initializing value separately for each element.
     *
     * Serialized as opcode `0x03`.
     */
    Array,

    /**
     * ## Push the value of an object's field member to stack
     *
     *  Pops a value from the `OperandStack` assuming it is a `RuntimeObject`. Then, retrieves a
     *  `ProgramObject::String` object at the index specified by `name`. The `ProgramObject::String`
     *  object is then used to reference a field member of the `RuntimeObject` by name, producing
     *  a value that is also a `RuntimeObject`. The value is pushed onto the operand stack.
     *
     * Serialized as opcode `0x05`.
     */
    GetField { name: ConstantPoolIndex },

    /**
     * ## Set the value of an object's field member variable to the top value from stack
     *
     * Pops a value to store from the `OperandStack`, assume it is a `RuntimeObject`. Then,
     * pops a host object from the `OperandStack`, also a `RuntimeObject`. Then, looks
     * up the index given by `name` in the `ConstantPool` and retrieves a `ProgramObject::String`
     * object. Afterwards, sets the value of the member field of the host object specified by the
     * `ProgramObject::String` to the value. Finally, push the value onto the operand stack.
     *
     * Serialized as opcode `0x06`.
     */
    SetField { name: ConstantPoolIndex },

    /**
     * ## Call a member method
     *
     * Pops `arguments` values from the `OperandStack` for the arguments to the call. The last popped
     *`RuntimeObject` from the `OperandStack` will be used as the method call's receiver.
     * Afterwards, a `ProgramObject::String` object representing the name of the method to call is
     * retrieved from the `ConstantPool` from the index specified by `name`.
     *
     * If the receiver is a `RuntimeObject::Integer` or `RuntimeObject::Array`, then the result of
     * the method call (as specified by the semantics of Feeny/FML) is pushed onto the stack.
     *
     * If the receiver is a `RuntimeObject::Object`, then a new `LocalFrame` is created for the
     * context of the call. Slot 0 in the new `LocalFrame` holds the receiver object, and the
     * following n slots hold the argument values starting with the deepest value on the stack (last
     * popped) and ending with the shallowest value on the stack (first popped). The new
     * `LocalFrame` has the current frame as its parent and the current `InstructionPointer` as the
     * return `Address`.
     *
     * Execution proceeds by registering the newly created frame as the current `LocalFrame`, and
     * setting the `InstructionPointer` to the `Address` of the body of the method.
     *
     * Serialized as opcode `0x07`.
     */
    CallMethod { name: ConstantPoolIndex, arguments: Arity },

    /**
     * ## Call a global function
     *
     * Pops `arguments` values from the `OperandStack` for the arguments to the call. Then, a
     * `ProgramObject::Method` object representing the function to call is retrieved from the
     * `ConstantPool` from the index specified by `function`.
     *
     * The first `arguments` slots in the frame hold argument values starting with the deepest value
     * on the stack (last popped) and ending with the shallowest value on the stack (first popped).
     * The new `LocalFrame` has the current frame as its parent, and the current
     * `InstructionPointer` as the return address. Execution proceeds by registering the newly
     * created `LocalFrame` as the current frame, and setting the `InstructionPointer` to the
     * `Address` of the body of the function.
     *
     * Serialized as opcode `0x08`.
     */
    CallFunction { name: ConstantPoolIndex, arguments: Arity },

    /**
     * ## Define a new label here
     *
     * Associates `name` with the address of this instruction. The name is given by the
     * `ProgramObject::String`object at the specified index.
     *
     * Serialized as opcode `0x00`.
     */
    Label { name: ConstantPoolIndex },

    /**
     * ## Print a formatted string
     *
     * Pops `arguments` values from the `OperandStack`. Then retrieves a `ProgramObject::String`
     * object referenced by the given `format` index. Then, prints out all the values retrieved from
     * the `OperandStack` out according to the given retrieved format string. `Null` is then pushed
     * onto the `OperandStack`.
     *
     * Arguments are spliced in from the deepest value in the stack (last popped) to the
     * shallowest value in the stack (first popped).
     *
     * Serialized as opcode `0x02`.
     */
    Print { format: ConstantPoolIndex, arguments: Arity },

    /**
     * ## Jump to a label
     *
     * Sets the `InstructionPointer` to the instruction `Address` associated with the name given
     * by the `ProgramObject::String` at the given index in the `ConstantPool`.
     *
     * Serialized as opcode `0x0E`.
     */
    Jump { label: ConstantPoolIndex },

    /**
     * ## Conditionally jump to a label
     *
     * Pops a value from the `OperandStack`. If this value is not `Null`, then sets the
     * `InstructionPointer` to the instruction `Address` associated with the name given by the
     * `ProgramObject::String` object at the given index.
     *
     * Serialized as opcode `0x0D`.
     */
    Branch { label: ConstantPoolIndex },

    /**
     * ## Return from the current function or method
     *
     * Registers the parent frame of the current `LocalFrame` as the current frame. Execution
     * proceeds by setting the `InstructionPointer` to the return `Address` stored in the current
     * `LocalFrame`.
     *
     * The `LocalFrame` is no longer used after a `Return` instruction and any storage allocated
     * for it may be reclaimed.
     *
     * Serialized as opcode `0x0F`.
     */
    Return,

    /**
     * ## Discard top of stack
     *
     * Pops and discards the top value from the `OperandStack`.
     *
     * Serialized as opcode `0x10`.
     */
    Drop,
}

impl Serializable for OpCode {
    fn serialize<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        serializable::write_u8(sink, self.to_hex())?;

        use OpCode::*;
        match self {
            Label { name } => name.serialize(sink),
            Literal { index } => index.serialize(sink),
            Print { format, arguments } => {
                format.serialize(sink)?;
                arguments.serialize(sink)
            }
            Array => Ok(()),
            Object { class } => class.serialize(sink),
            GetField { name } => name.serialize(sink),
            SetField { name } => name.serialize(sink),
            CallMethod { name, arguments } => {
                name.serialize(sink)?;
                arguments.serialize(sink)
            }
            CallFunction { name: function, arguments } => {
                function.serialize(sink)?;
                arguments.serialize(sink)
            }
            SetLocal { index } => index.serialize(sink),
            GetLocal { index } => index.serialize(sink),
            SetGlobal { name } => name.serialize(sink),
            GetGlobal { name } => name.serialize(sink),
            Branch { label } => label.serialize(sink),
            Jump { label } => label.serialize(sink),
            Return => Ok(()),
            Drop => Ok(()),
            // Skip => { Ok(()) },
        }
    }

    fn from_bytes<R: Read>(input: &mut R) -> Self {
        let tag = serializable::read_u8(input);

        use OpCode::*;
        match tag {
            0x00 => Label { name: ConstantPoolIndex::from_bytes(input) },
            0x01 => Literal { index: ConstantPoolIndex::from_bytes(input) },
            0x02 => Print {
                format: ConstantPoolIndex::from_bytes(input),
                arguments: Arity::from_bytes(input),
            },
            0x03 => Array {},
            0x04 => Object { class: ConstantPoolIndex::from_bytes(input) },
            0x05 => GetField { name: ConstantPoolIndex::from_bytes(input) },
            0x06 => SetField { name: ConstantPoolIndex::from_bytes(input) },
            0x07 => CallMethod {
                name: ConstantPoolIndex::from_bytes(input),
                arguments: Arity::from_bytes(input),
            },
            0x08 => CallFunction {
                name: ConstantPoolIndex::from_bytes(input),
                arguments: Arity::from_bytes(input),
            },
            0x09 => SetLocal { index: LocalFrameIndex::from_bytes(input) },
            0x0A => GetLocal { index: LocalFrameIndex::from_bytes(input) },
            0x0B => SetGlobal { name: ConstantPoolIndex::from_bytes(input) },
            0x0C => GetGlobal { name: ConstantPoolIndex::from_bytes(input) },
            0x0D => Branch { label: ConstantPoolIndex::from_bytes(input) },
            0x0E => Jump { label: ConstantPoolIndex::from_bytes(input) },
            0x0F => Return,
            0x10 => Drop,
            tag => panic!("Cannot deserialize opcode: unknown tag {}", tag),
        }
    }
}

impl OpCode {
    pub fn to_hex(&self) -> u8 {
        use OpCode::*;
        match self {
            Label { name: _ } => 0x00,
            Literal { index: _ } => 0x01,
            Print { format: _, arguments: _ } => 0x02,
            Array {} => 0x03,
            Object { class: _ } => 0x04,
            GetField { name: _ } => 0x05,
            SetField { name: _ } => 0x06,
            CallMethod { name: _, arguments: _ } => 0x07,
            CallFunction { name: _, arguments: _ } => 0x08,
            SetLocal { index: _ } => 0x09,
            GetLocal { index: _ } => 0x0A,
            SetGlobal { name: _ } => 0x0B,
            GetGlobal { name: _ } => 0x0C,
            Branch { label: _ } => 0x0D,
            Jump { label: _ } => 0x0E,
            Return => 0x0F,
            Drop => 0x10,
            // Skip => 0xFF,
        }
    }

    pub fn read_opcode_vector<R: Read>(reader: &mut R) -> Vec<OpCode> {
        let length = serializable::read_u32_as_usize(reader);
        let mut opcodes: Vec<OpCode> = Vec::new();
        for _ in 0..length {
            opcodes.push(OpCode::from_bytes(reader));
        }
        opcodes
    }

    pub fn write_opcode_vector<W: Write>(
        sink: &mut W,
        vector: &Vec<&OpCode>,
    ) -> anyhow::Result<()> {
        serializable::write_usize_as_u32(sink, vector.len())?;
        for opcode in vector {
            opcode.serialize(sink)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Literal { index } => write!(f, "lit {}", index),
            OpCode::GetLocal { index } => write!(f, "get local {}", index),
            OpCode::SetLocal { index } => write!(f, "set local {}", index),
            OpCode::GetGlobal { name } => write!(f, "get global {}", name),
            OpCode::SetGlobal { name } => write!(f, "set global {}", name),
            OpCode::Object { class } => write!(f, "object {}", class),
            OpCode::Array => write!(f, "array"),
            OpCode::GetField { name } => write!(f, "get slot {}", name),
            OpCode::SetField { name } => write!(f, "set slot {}", name),
            OpCode::CallMethod { name, arguments } => write!(f, "call slot {} {}", name, arguments),
            OpCode::CallFunction { name, arguments } => write!(f, "call {} {}", name, arguments),
            OpCode::Print { format, arguments } => write!(f, "printf {} {}", format, arguments),
            OpCode::Label { name } => write!(f, "label {}", name),
            OpCode::Jump { label } => write!(f, "goto {}", label),
            OpCode::Branch { label } => write!(f, "branch {}", label),
            OpCode::Return => write!(f, "return"),
            OpCode::Drop => write!(f, "drop"),
        }
    }
}
