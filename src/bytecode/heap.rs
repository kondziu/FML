use anyhow::*;
use indexmap::IndexMap;

use crate::bytecode::state::OperandStack;
use crate::bytecode::program::{ProgramObject, ConstantPoolIndex};

macro_rules! size {
    ($n:literal B)  => { $n * 8 };
    ($n:literal KB) => { $n * 1024 * 8 };
    ($n:literal MB) => { $n * 1024 * 1024 * 8 };
}

#[derive(Eq, PartialEq, Debug)]
pub struct LLHeap {
    pointer: usize,
    memory: [u8; size!(128 MB)]
}
impl LLHeap {
    pub fn new() -> Self {
        LLHeap {
            pointer: 0,
            memory: [0; size!(128 MB)],
        }
    }
    pub fn write_byte(&mut self, value: u8) {
        self.memory[self.pointer] = value;
        self.pointer += 1;
    }
    pub fn write_usize(&mut self, value: usize) {
        let length = std::mem::size_of::<usize>();
        let end = self.pointer + length;
        let target_slice = &mut self.memory[self.pointer..end];
        let source_slice = unsafe { std::mem::transmute::<usize, [u8; 8]>(value) };
        target_slice.copy_from_slice(&source_slice);
        self.pointer = end;
    }
    pub fn write_pointer(&mut self, value: &Pointer) {
        let length = std::mem::size_of::<Pointer>();
        let end = self.pointer + length;
        let target_slice = &mut self.memory[self.pointer..end];
        let source_slice = unsafe { std::mem::transmute::<&Pointer, &[u8; 16]>(value) };
        target_slice.copy_from_slice(source_slice);
        self.pointer = end;
    }
    pub fn write_constant_pool_index(&mut self, value: &ConstantPoolIndex) {
        let length = std::mem::size_of::<ConstantPoolIndex>();
        let end = self.pointer + length;
        let target_slice = &mut self.memory[self.pointer..end];
        let source_slice = unsafe { std::mem::transmute::<&ConstantPoolIndex, &[u8; 2]>(value) };
        target_slice.copy_from_slice(source_slice);
        self.pointer = end;
    }
    // +-------+-------+
    // |     0 | TAG   |
    // +-------+-------+-------+-------+-------+
    // |     1 | SIZE                          |
    // +-------+-------+-------+-------+-------+-------+
    // | 5+i*5 | ELEMENT i PTR                         |
    // +-------+-------+-------+-------+-------+-------+
    // :       :                                       :
    pub fn allocate_array(&mut self, elements: usize, value: &Pointer) -> HeapIndex {
        let index = HeapIndex::from(self.pointer);
        self.write_byte(0x00);
        self.write_usize(elements);
        (0..elements).for_each(|_| self.write_pointer(value));
        index
    }
    // +--------+--------+
    // |      0 | TAG    |
    // +--------+--------+--------+
    // |      1 | CLASS CPI       |
    // +--------+--------+--------+--------+--------+--------+
    // |      3 | PARENT PTR                                 |
    // +--------+--------+--------+--------+--------+--------+
    // |  8+i*7 | FLD i NAME CPI  |
    // +--------+--------+--------+--------+--------+--------+
    // | 10+i*7 | FLD i VALUE PTR                            |
    // +--------+--------+--------+--------+--------+--------+
    // :        :                                            :
    pub fn allocate_object(&mut self, class: &ConstantPoolIndex, parent: &Pointer,
                           fields: Vec<(&ConstantPoolIndex, &Pointer)>) -> HeapIndex {
        let index = HeapIndex::from(self.pointer);
        self.write_byte(0x01);
        self.write_constant_pool_index(class);
        self.write_pointer(parent);
        fields.into_iter().for_each(|(name, value)| {
            self.write_constant_pool_index(name);
            self.write_pointer(value);
        });
        index
    }
}

#[derive(Eq, PartialEq, Debug)]
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

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum HeapObject {
    Array(ArrayInstance),
    Object(ObjectInstance)
}

impl HeapObject {
    #[allow(dead_code)]
    pub fn new_object(parent: Pointer, fields: IndexMap<String, Pointer>, methods: IndexMap<String, ProgramObject>) -> Self {
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
    #[allow(dead_code)]
    pub fn empty_object() -> Self {
        HeapObject::Object(ObjectInstance::new())
    }
    #[allow(dead_code)]
    pub fn empty_array() -> Self {
        HeapObject::Array(ArrayInstance::new())
    }
    pub fn from_pointers(v: Vec<Pointer>) -> Self {
        HeapObject::Array(ArrayInstance::from(v))
    }
    #[allow(dead_code)]
    pub fn from(parent: Pointer, fields: IndexMap<String, Pointer>, methods: IndexMap<String, ProgramObject>) -> Self {
        HeapObject::Object(ObjectInstance { parent, fields, methods })
    }
    pub fn evaluate_as_string(&self, heap: &Heap) -> Result<String> {
        match self {
            HeapObject::Array(array) => array.evaluate_as_string(heap),
            HeapObject::Object(object) => object.evaluate_as_string(heap),
        }
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


#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub struct ArrayInstance(Vec<Pointer>);

impl ArrayInstance {
    #[allow(dead_code)]
    pub fn new() -> Self {
        ArrayInstance(vec![])
    }
    #[allow(dead_code)]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=&Pointer> + 'a {
        self.0.iter()
    }
    #[allow(dead_code)]
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
    pub fn evaluate_as_string(&self, heap: &Heap) -> Result<String> {
        let elements = self.0.iter()
            .map(|element| element.evaluate_as_string(heap))
            .collect::<Result<Vec<String>>>()?;
        Ok(format!("[{}]", elements.join(", ")))
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

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ObjectInstance {
    pub parent: Pointer,
    pub fields: IndexMap<String, Pointer>, // TODO make private
    pub methods: IndexMap<String, ProgramObject> // TODO make private
}

impl ObjectInstance {
    #[allow(dead_code)]
    pub fn new() -> Self {
        ObjectInstance  {
            parent: Pointer::Null,
            fields: IndexMap::new(),
            methods: IndexMap::new(),
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
    pub fn evaluate_as_string(&self, heap: &Heap) -> Result<String> {
        let parent = match self.parent {
            Pointer::Null => None,
            parent => Some(parent.evaluate_as_string(heap)?),
        };

        let fields = self.fields.iter()
            .map(|(name, value)| {
                value.evaluate_as_string(heap).map(|value| format!("{}={}", name, value))
            })
            .collect::<Result<Vec<String>>>()?;

        match parent {
            Some(parent) if fields.len() > 0 =>
                Ok(format!("object(..={}, {})", parent, fields.join(", "))),
            Some(parent)  =>
                Ok(format!("object(..={})", parent)),
            None => Ok(format!("object({})", fields.join(", "))),
        }
    }
}

impl std::fmt::Display for ObjectInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parent = match self.parent {
            Pointer::Null => None,
            parent => Some(parent.to_string()),
        };

        let fields = self.fields.iter()
            .map(|(name, value)| {
                format!("{}={}", name, value)
            })
            .collect::<Vec<String>>();

        match parent {
            Some(parent) => write!(f, "object(..={}, {})", parent, fields.join(", ")),
            None => write!(f, "object({})", fields.join(", ")),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
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

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
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
    #[allow(dead_code)]
    pub fn is_heap_reference(&self) -> bool {
        match self {
            Pointer::Reference(_) => true,
            _ => false,
        }
    }
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn as_null(&self) -> Option<()> {
        match self {
            Pointer::Null => Some(()),
            _ => None,
        }
    }
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn is_bool(&self) -> bool {
        match self {
            Pointer::Boolean(_) => true,
            _ => false,
        }
    }
    #[allow(dead_code)]
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

    pub fn evaluate_as_string(&self, heap: &Heap) -> Result<String> { // TODO trait candidate
        match self {
            Pointer::Null => Ok("null".to_owned()),
            Pointer::Integer(i) => Ok(i.to_string()),
            Pointer::Boolean(b) => Ok(b.to_string()),
            Pointer::Reference(index) => heap.dereference(index)?.evaluate_as_string(heap),
        }
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