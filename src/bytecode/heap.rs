use std::collections::HashMap;

use anyhow::*;

use crate::bytecode::interp::OperandStack;
use crate::bytecode::program::ProgramObject;

#[derive(PartialEq,Debug)]
pub struct Heap(Vec<HeapObject>);

impl Heap {
    pub fn new() -> Self {
        Heap(Vec::new())
    }

    #[allow(dead_code)]
    pub fn from(objects: Vec<HeapObject>) -> Self {
        Heap(objects)
    }

    pub fn allocate(&mut self, object: HeapObject) -> HeapIndex {
        let pointer = HeapIndex::from(self.0.len());
        self.0.push(object);
        pointer
    }

    pub fn dereference(&self, pointer: &HeapIndex) -> Option<&HeapObject> {
        let index = pointer.as_usize();
        if self.0.len() > index {
            Some(&self.0[index])
        } else {
            None
        }
    }

    pub fn dereference_mut(&mut self, pointer: &HeapIndex) -> Option<&mut HeapObject> {
        let index = pointer.as_usize();
        if self.0.len() > index {
            Some(&mut self.0[index])
        } else {
            None
        }
    }

    pub fn copy(&mut self, pointer: &HeapIndex) -> Option<HeapIndex> {
        self.dereference(pointer)
            .map(|object| object.clone())
            .map(|object| self.allocate(object))
    }

    pub fn dereference_heap_object_to_string(&self, object: &HeapObject) -> String {
        match object {
            // HeapObject::Null => "null".to_string(),
            // HeapObject::Integer(n) => n.to_string(),
            // HeapObject::Boolean(b) => b.to_string(),
            HeapObject::Array(elements) => {
                let element_string = elements.iter()
                    .map(|p| self.dereference_to_string(p))
                    .collect::<Vec<String>>()
                    .join(", ");

                format!("[{}]", element_string)
            },
            HeapObject::Object(ObjectInstance { parent, fields, methods:_ }) => {
                let parent_string = self.dereference_to_string(parent);
                let parent_string = if parent_string == "null" {
                    String::new()
                } else {
                    format!("..={}{}", parent_string, if fields.len() == 0 { "" } else { ", " })
                };
                let fields_string = fields.iter()
                    .map(|(name, field)| {
                        format!("{}={}", name, self.dereference_to_string(field))
                    })
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("object({}{})", parent_string, fields_string)
            }
        }
    }

    pub fn dereference_to_string(&self, pointer: &Pointer) -> String {
        match pointer {
            Pointer::Null => "null".to_string(),
            Pointer::Integer(n) => n.to_string(),
            Pointer::Boolean(b) => b.to_string(),
            Pointer::Reference(pointer) => {
                let object = self.dereference(pointer)
                    .expect(&format!("Expected object at {:?} to convert to string, but none was found",
                                     pointer));
                self.dereference_heap_object_to_string(object)
            }
        }
    }
}

#[derive(PartialEq,Debug,Clone)]
pub enum HeapObject {
    Array(ArrayInstance),
    Object(ObjectInstance)
}

impl HeapObject {
    pub fn new_object(parent: Pointer, fields: HashMap<String, Pointer>, methods: HashMap<String, ProgramObject>) -> Self {
        HeapObject::Object(ObjectInstance { parent, fields, methods })
    }

    pub fn as_object_instance(&self) -> Result<&ObjectInstance> {
        match self {
            HeapObject::Object(instance) => Ok(instance),
            array => Err(anyhow!("Attempt to cast an array as an object instance `{}`.", array)),
        }
    }

    pub fn as_object_instance_mut(&mut self) -> Result<&mut ObjectInstance> {
        match self {
            HeapObject::Object(instance) => Ok(instance),
            array => Err(anyhow!("Attempt to cast an array as an object instance `{}`.", array)),
        }
    }

    pub fn empty_object() -> Self {
        HeapObject::Object(ObjectInstance::new())
    }

    pub fn empty_array() -> Self {
        HeapObject::Array(ArrayInstance::new())
    }

    pub fn from_pointers(v: Vec<Pointer>) -> Self {
        HeapObject::Array(ArrayInstance::from(v))
    }

    pub fn from(parent: Pointer, fields: HashMap<String, Pointer>, methods: HashMap<String, ProgramObject>) -> Self {
        HeapObject::Object(ObjectInstance { parent, fields, methods })
    }
}

impl std::fmt::Display for HeapObject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HeapObject::Array(array) => write!(f, "{}", array),
            HeapObject::Object(object) => write!(f, "{}", object),
        }
    }
}


#[derive(PartialEq,Debug,Clone)]
pub struct ArrayInstance(Vec<Pointer>);

impl ArrayInstance {
    pub fn new() -> Self {
        ArrayInstance(vec![])
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=&Pointer> + 'a {
        self.0.iter()
    }

    pub fn length(&self) -> usize {
        self.0.len()
    }

    pub fn get_element(&self, index: usize) -> Result<&Pointer> {
        let length = self.0.len();
        bail_if!(index >= length,
                 "Index out of range {} for array `{}` with length {}",
                 index, self, length);
        Ok(&self.0[index])
    }

    pub fn set_element(&mut self, index: usize, value_pointer: Pointer) -> Result<&Pointer> {
        let length = self.0.len();
        bail_if!(index >= length,
                 "Index out of range {} for array `{}` with length {}",
                 index, self, length);
        self.0[index] = value_pointer;
        Ok(&self.0[index])
    }
}

impl From<Vec<Pointer>> for ArrayInstance {
    fn from(v: Vec<Pointer>) -> Self {
        ArrayInstance(v)
    }
}

impl std::fmt::Display for ArrayInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.iter()
            .map(|pointer| format!("{}", pointer))
            .collect::<Vec<String>>()
            .join(", "))
    }
}

#[derive(PartialEq,Debug,Clone)]
pub struct ObjectInstance {
    pub parent: Pointer,
    pub fields: HashMap<String, Pointer>, // TODO make private
    pub methods: HashMap<String, ProgramObject> // TODO make private
}

impl ObjectInstance {
    pub fn new() -> Self {
        ObjectInstance  {
            parent: Pointer::Null,
            fields: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    pub fn get_field(&self, name: &str) -> Result<&Pointer> {
        self.fields.get(name)
            .with_context(|| format!("There is no field named `{}` in object `{}`", name, self))
    }

    pub fn set_field(&mut self, name: &str, pointer: Pointer) -> Result<Pointer> {
        self.fields.insert(name.to_owned(), pointer)
            .with_context(|| format!("There is no field named `{}` in object `{}`", name, self))
    }
}

impl std::fmt::Display for ObjectInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "object(..={}, {})", self.parent, self.fields.iter()
            .map(|(name, field)| format!("{}={}", name, field))
            .collect::<Vec<String>>()
            .join(", "))

    }
}

#[derive(PartialEq,Eq,Debug,Hash,Clone,Copy)]
pub struct HeapIndex(usize);

impl From<usize> for HeapIndex {
    fn from(n: usize) -> Self {
        HeapIndex(n)
    }
}

impl From<&Pointer> for HeapIndex {
    fn from(p: &Pointer) -> Self {
        match p {
            Pointer::Reference(p) => p.clone(),
            Pointer::Null => panic!("Cannot create heap reference from a null-tagged pointer"),
            Pointer::Integer(_) => panic!("Cannot create heap reference from an integer-tagged pointer"),
            Pointer::Boolean(_) => panic!("Cannot create heap reference from a boolean-tagged pointer"),
        }
    }
}

impl From<Pointer> for HeapIndex {
    fn from(p: Pointer) -> Self {
        match p {
            Pointer::Reference(p) => p.clone(),
            Pointer::Null => panic!("Cannot create heap reference from a null-tagged pointer"),
            Pointer::Integer(_) => panic!("Cannot create heap reference from an integer-tagged pointer"),
            Pointer::Boolean(_) => panic!("Cannot create heap reference from a boolean-tagged pointer"),
        }
    }
}

impl HeapIndex {
    pub fn as_usize(&self) -> usize { self.0 }
}

impl std::fmt::Display for HeapIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x>8}", self.0)
    }
}

#[derive(PartialEq,Eq,Debug,Hash,Clone,Copy)]
pub enum Pointer {
    Null,
    Integer(i32),
    Boolean(bool),
    Reference(HeapIndex),
}

// impl Deref for Pointer {
//     type Target = Pointer;
//     fn deref(&self) -> &Self::Target {
//         self.clone()
//     }
// }

impl Pointer {
    pub fn push_onto(self, stack: &mut OperandStack) {
        stack.push(self);
    }
}

impl Pointer {
    pub fn from_literal(program_object: &ProgramObject) -> Result<Pointer> {
        match program_object {
            ProgramObject::Null => Ok(Self::Null),
            ProgramObject::Integer(value) => Ok(Self::Integer(*value)),
            ProgramObject::Boolean(value) => Ok(Self::Boolean(*value)),
            _ => bail!("Expecting either a null, an integer, or a boolean, but found `{}`.", program_object),
        }
    }

    pub fn is_heap_reference(&self) -> bool {
        match self {
            Pointer::Reference(_) => true,
            _ => false,
        }
    }
    pub fn as_heap_reference(&self) -> Option<&HeapIndex> {
        match self {
            Pointer::Reference(reference) => Some(reference),
            _ => None,
        }
    }
    pub fn into_heap_reference(self) -> Result<HeapIndex> {
        match self {
            Pointer::Reference(reference) => Ok(reference),
            pointer => Err(anyhow!("Expecting a heap reference, but found `{}`.", pointer)),
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Pointer::Null => true,
            _ => false,
        }
    }
    pub fn as_null(&self) -> Option<()> {
        match self {
            Pointer::Null => Some(()),
            _ => None,
        }
    }

    pub fn is_i32(&self) -> bool {
        match self {
            Pointer::Integer(_) => true,
            _ => false,
        }
    }
    pub fn as_i32(&self) -> Result<i32> {
        match self {
            Pointer::Integer(i) => Ok(*i),
            pointer => Err(anyhow!("Expecting an integer, but found `{}`", pointer)),
        }
    }

    pub fn as_usize(&self) -> Result<usize> {
        match self {
            Pointer::Integer(i) if *i >= 0 => Ok(*i as usize),
            pointer => Err(anyhow!("Expecting a positive integer, but found `{}`", pointer)),
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Pointer::Boolean(_) => true,
            _ => false,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Pointer::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn evaluate_as_condition(&self) -> bool {
        match self {
            Pointer::Null => false,
            Pointer::Integer(_) => true,
            Pointer::Boolean(b) => *b,
            Pointer::Reference(_) => true,
        }
    }

    pub fn evaluate_as_string(&self) -> String {
        unimplemented!()
    }
}

impl Into<bool> for Pointer {
    fn into(self) -> bool {
        match self {
            Pointer::Boolean(b) => b,
            p => panic!("Cannot cast `{}` into a boolean pointer.", p),
        }
    }
}

impl Into<i32> for Pointer {
    fn into(self) -> i32 {
        match self {
            Pointer::Integer(i) => i,
            p => panic!("Cannot cast `{}` into an integer pointer.", p),
        }
    }
}

impl From<&ProgramObject> for Pointer {
    fn from(constant: &ProgramObject) -> Self {
        match constant {
            ProgramObject::Null => Self::Null,
            ProgramObject::Integer(value) => Self::Integer(*value),
            ProgramObject::Boolean(value) => Self::Boolean(*value),
            _ => unimplemented!(),
        }
    }
}

impl From<ProgramObject> for Pointer {
    fn from(constant: ProgramObject) -> Self {
        match constant {
            ProgramObject::Null => Self::Null,
            ProgramObject::Integer(value) => Self::Integer(value),
            ProgramObject::Boolean(value) => Self::Boolean(value),
            _ => unimplemented!(),
        }
    }
}

impl From<&HeapIndex> for Pointer {
    fn from(p: &HeapIndex) -> Self {
        Pointer::Reference(p.clone())
    }
}

impl From<HeapIndex> for Pointer {
    fn from(p: HeapIndex) -> Self {
        Pointer::Reference(p)
    }
}

impl From<usize> for Pointer {
    fn from(n: usize) -> Self {
        Pointer::from(HeapIndex::from(n))
    }
}

impl From<i32> for Pointer {
    fn from(i: i32) -> Self {
        Pointer::Integer(i)
    }
}

impl From<bool> for Pointer {
    fn from(b: bool) -> Self {
        Pointer::Boolean(b)
    }
}

impl std::fmt::Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pointer::Null => write!(f, "null"),
            Pointer::Integer(i) => write!(f, "{}", i),
            Pointer::Boolean(b) => write!(f, "{}", b),
            Pointer::Reference(p) => write!(f, "{}", p),
        }
    }
}