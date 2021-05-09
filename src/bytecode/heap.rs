use anyhow::*;
use indexmap::IndexMap;

use crate::bytecode::state::{OperandStack, FrameStack};
use crate::bytecode::program::{ProgramObject, ConstantPoolIndex, AddressRange, Arity, Size};
use std::path::PathBuf;
use std::fs::{File, create_dir_all};
use std::time::SystemTime;
use std::io::Write;
use std::mem::size_of;
use std::collections::{HashSet, BTreeSet};

macro_rules! heap_log {
    (START -> $file:expr) => {
        if let Some(file) = &mut $file {
            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
            write!(file, "{},S,0\n", timestamp).unwrap();
        }
    };
    (ALLOCATE -> $file:expr, $memory:expr) => {
        if let Some(file) = &mut $file {
            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
            write!(file, "{},A,{}\n", timestamp, $memory).unwrap();
        }
    };
    (GC -> $file:expr, $memory:expr) => {
        if let Some(file) = &mut $file {
            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
            write!(file, "{},G,{}\n", timestamp, $memory).unwrap();
        }
    }
}

#[derive(Debug)]
pub struct Heap{ max_size: usize, size: usize, log: Option<File>, memory: Vec<HeapObject>, holes: usize }

impl Eq for Heap {}
impl PartialEq for Heap {
    fn eq(&self, other: &Self) -> bool {
        self.memory.eq(&other.memory)
    }
}

impl Heap {
    pub fn set_size(&mut self, size: usize /* in MB */) {
        self.max_size = size * 1024 * 1024 /* in B */
    }
    pub fn set_log(&mut self, path: PathBuf) {

        let mut dir = path.clone();
        dir.pop();
        create_dir_all(dir).unwrap();

        let mut file = File::create(path).unwrap();
        write!(file, "timestamp,event,memory\n").unwrap();

        heap_log!(START -> Some(&mut file));
        self.log = Some(file)
    }
    pub fn new() -> Self {
        Heap { max_size: 0, log: None, memory: Vec::new(), size: 0, holes: 0 }
    }
    pub fn will_fit(&self, object: &HeapObject) -> bool {
        self.size + object.size() > self.max_size && self.max_size != 0
    }
    pub fn allocate(&mut self, object: HeapObject) -> HeapIndex {
        self.size += object.size();
        if self.size > self.max_size && self.max_size != 0 {
            panic!("Cannot allocate object {}. \
            Current heap size: {}. Max heap size: {}. Object size: {}",
                   object, self.size - object.size(), self.max_size, object.size())
        }
        heap_log!(ALLOCATE -> self.log, self.size);
        if self.holes == 0 {
            self.bump_allocate(object)
        } else {
            self.first_fit_allocate(object)
        }
    }
    fn bump_allocate(&mut self, object: HeapObject) -> HeapIndex {
        let index = HeapIndex::from(self.memory.len());
        self.memory.push(object);
        index
    }
    fn first_fit_allocate(&mut self, object: HeapObject) -> HeapIndex {
        for i in 0..self.memory.len() {
            if self.memory[i].is_free() {
                self.holes -= 1;
                self.size -= HeapObject::Free.size();
                return HeapIndex::from(i)
            }
        }
        return self.bump_allocate(object);
    }
    #[allow(dead_code)]
    pub fn free(&mut self, index: HeapIndex) {
        self.size -= self.memory.get(index.as_usize()).unwrap().size();
        let free_node = HeapObject::Free;
        self.size += free_node.size();
        self.holes += 1;
        self.memory.insert(index.as_usize(), free_node);
    }
    pub fn free_all(&mut self, indices: BTreeSet<HeapIndex>) {
        let removed_size: usize =
            indices.iter().map(|index| self.memory.get(index.as_usize()).unwrap().size()).sum();
        self.size -= removed_size;
        self.size += HeapObject::Free.size() * indices.len();
        self.holes += indices.len();
        for index in indices.into_iter() {
            self.memory.insert(index.as_usize(), HeapObject::Free);
        }
        // TODO coalesce at the end of the vector
    }
    pub fn dereference(&self, index: &HeapIndex) -> Result<&HeapObject> {
        self.memory.get(index.as_usize())
            .with_context(||
                format!("Cannot dereference object from the heap at index: `{}`", index))
    }
    pub fn dereference_mut(&mut self, index: &HeapIndex) -> Result<&mut HeapObject> {
        self.memory.get_mut(index.as_usize())
            .with_context(||
                format!("Cannot dereference object from the heap at index: `{}`", index))
    }
    pub fn gc(&mut self, stack: &OperandStack, frames: &FrameStack) {
        let roots: BTreeSet<HeapIndex> = stack.find_roots().chain(frames.find_roots()).collect();

        // Mark
        let mut visited: HashSet<HeapIndex> = HashSet::new();
        let mut todo: Vec<HeapIndex> = roots.into_iter().collect();

        while !todo.is_empty() {
            let index = todo.pop().unwrap();
            if visited.contains(&index) {
                continue;
            }

            let object = self.dereference(&index).unwrap();
            let references = object.find_all_heap_references();
            todo.extend(references.into_iter());

            visited.insert(index);
        }

        // Sweep
        let reachable = visited;
        let unreachable =
            (0..self.memory.len()).into_iter()
                .map(|i| HeapIndex::from(i))
                .filter(|index| reachable.contains(&index))
                .collect();

        self.free_all(unreachable);
    }
}

impl From<Vec<HeapObject>> for Heap {
    fn from(objects: Vec<HeapObject>) -> Self {
        Heap {
            size: objects.iter().map(|o| o.size()).sum(),
            max_size: 0,
            holes: 0,
            log: None,
            memory: objects
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum HeapObject {
    Array(ArrayInstance),
    Object(ObjectInstance),
    Free
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
            HeapObject::Free => Ok("<free>".to_owned()),
            HeapObject::Array(array) => array.evaluate_as_string(heap),
            HeapObject::Object(object) => object.evaluate_as_string(heap),
        }
    }
    pub fn is_free(&self) -> bool {
        match self {
            HeapObject::Free => true,
            _ => false,
        }
    }
    pub fn size(&self) -> usize {
        match self {
            HeapObject::Free => {
                size_of::<HeapObject>()
            }
            HeapObject::Array(array) => {
                size_of::<ArrayInstance>() + array.length() * size_of::<Pointer>()
            }
            HeapObject::Object(object) => {
                let header = size_of::<ObjectInstance>();
                let fields: usize =
                    object.fields.iter().map(|(string, _pointer)| string.len() + size_of::<Pointer>()).sum();
                let methods: usize =
                    object.methods.iter().map(|(string, program_object)| string.len() + match program_object {
                        ProgramObject::Method { .. } =>
                            size_of::<ConstantPoolIndex>() +
                                size_of::<Arity>() +
                                size_of::<Size>() +
                                size_of::<AddressRange>(),
                        _ => unreachable!()
                    }).sum();
                header + fields + methods
            }
        }
    }
    pub fn find_all_heap_references(&self) -> BTreeSet<HeapIndex> {
        match self {
            HeapObject::Array(instance) => instance.find_all_heap_references(),
            HeapObject::Object(instance) => instance.find_all_heap_references(),
            HeapObject::Free => panic!("Attempting to mark area of free memory"),
        }
    }
}

impl std::fmt::Display for HeapObject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HeapObject::Free => write!(f, "<free>"),
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
    pub fn find_all_heap_references(&self) -> BTreeSet<HeapIndex> {
        self.0.iter()
            .filter(|pointer| pointer.is_heap_reference())
            .map(|pointer| pointer.into_heap_reference().unwrap())
            .collect()
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
    pub fn find_all_heap_references(&self) -> BTreeSet<HeapIndex> {
        self.fields.iter().map(|(_, pointer)| pointer).chain(vec![self.parent].iter())
            .filter(|pointer| pointer.is_heap_reference())
            .map(|pointer| pointer.into_heap_reference().unwrap())
            .collect()
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