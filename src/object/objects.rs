use super::{AllObjects, Object};
use crate::code::Instructions;
use anyhow::Result;
use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Integer {
    pub value: i64,
}

impl Object for Integer {
    fn inspect(&self) -> String {
        format!("{}", self.value)
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct StringObj {
    pub value: Rc<String>,
}

impl StringObj {
    pub fn new(v: &str) -> Self {
        Self {
            value: Rc::new(v.to_string()),
        }
    }
}

impl Object for StringObj {
    fn inspect(&self) -> String {
        self.value.replace("\\n", "\n").replace("\\t", "\t")
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Boolean {
    pub value: bool,
}

impl Object for Boolean {
    fn inspect(&self) -> String {
        format!("{}", self.value)
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Null;

impl Object for Null {
    fn inspect(&self) -> String {
        "null".to_string()
    }
}

#[derive(Clone)]
pub struct CompiledFunctionObj {
    pub instructions: Instructions,
    pub num_args: usize,
}

impl CompiledFunctionObj {
    pub fn new(instructions: Instructions, num_args: usize) -> Self {
        Self {
            instructions,
            num_args,
        }
    }
}

impl PartialEq for CompiledFunctionObj {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self.instructions) == format!("{:?}", other.instructions)
    }
}

impl Eq for CompiledFunctionObj {}

impl Hash for CompiledFunctionObj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{:?}", self.instructions).hash(state);
    }
}

impl Object for CompiledFunctionObj {
    fn inspect(&self) -> String {
        "fn(){}".to_string()
    }
}

pub type BuiltinFn = fn(Vec<AllObjects>) -> Result<AllObjects>;

#[derive(Clone)]
pub struct BuiltinFunctionObj {
    pub fn_name: String,
    pub num_params: usize,
    pub func: BuiltinFn,
}

impl BuiltinFunctionObj {
    pub fn new(fn_name: &str, num_params: usize, func: BuiltinFn) -> Self {
        Self {
            fn_name: fn_name.to_string(),
            num_params,
            func,
        }
    }
}

impl PartialEq for BuiltinFunctionObj {
    fn eq(&self, other: &Self) -> bool {
        self.fn_name == other.fn_name
    }
}

impl Eq for BuiltinFunctionObj {}

impl Hash for BuiltinFunctionObj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fn_name.hash(state);
    }
}

impl Object for BuiltinFunctionObj {
    fn inspect(&self) -> String {
        self.fn_name.to_string()
    }
}

#[derive(Clone)]
pub struct ArrayObj {
    pub elements: Rc<RefCell<Vec<AllObjects>>>,
}

impl ArrayObj {
    pub fn new(v: Vec<AllObjects>) -> Self {
        Self {
            elements: Rc::new(RefCell::new(v)),
        }
    }
}

impl PartialEq for ArrayObj {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl Eq for ArrayObj {}

impl Hash for ArrayObj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inspect().hash(state)
    }
}

impl Object for ArrayObj {
    fn inspect(&self) -> String {
        format!(
            "[{}]",
            self.elements
                .borrow()
                .iter()
                .map(|v| v.inspect())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone)]
pub struct HashMapObj {
    pub map: Rc<RefCell<HashMap<AllObjects, AllObjects>>>,
}

impl HashMapObj {
    pub fn new(map: HashMap<AllObjects, AllObjects>) -> Self {
        Self {
            map: Rc::new(RefCell::new(map)),
        }
    }
}

impl PartialEq for HashMapObj {
    fn eq(&self, other: &Self) -> bool {
        self.inspect() == other.inspect()
    }
}

impl Eq for HashMapObj {}

impl Hash for HashMapObj {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inspect().hash(state)
    }
}

impl Object for HashMapObj {
    fn inspect(&self) -> String {
        let binding = self.map.borrow();
        let out = binding
            .iter()
            .map(|(k, v)| format!("{}:{}", k.inspect(), v.inspect()))
            .collect::<Vec<String>>()
            .join(", ");

        format!("{{ {} }}", out)
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Closure {
    pub func: CompiledFunctionObj,
    pub free: Vec<AllObjects>,
}

impl Closure {
    pub fn new(func: CompiledFunctionObj) -> Self {
        Self { func, free: vec![] }
    }
}

impl Object for Closure {
    fn inspect(&self) -> String {
        format!("Closure[{}]", self.func.inspect())
    }
}
