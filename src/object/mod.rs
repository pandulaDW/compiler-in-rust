use std::fmt::{self, Display};

pub mod builtins;
pub mod objects;

pub trait Object {
    fn inspect(&self) -> String;
}

/// This is useful when doing just type comparisons disregarding underlying value
#[derive(PartialEq, Eq)]
pub enum ObjectType {
    Integer,
    String,
    Boolean,
    Null,
    CompiledFunction,
    BuiltInFunction,
    Array,
    HashMap,
    Closure,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let out = match self {
            ObjectType::Integer => "INTEGER",
            ObjectType::String => "STRING",
            ObjectType::Boolean => "BOOLEAN",
            ObjectType::Null => "NULL",
            ObjectType::CompiledFunction => "COMPILED_FUNCTION",
            ObjectType::BuiltInFunction => "BUILTIN_FUNCTION",
            ObjectType::Array => "ARRAY",
            ObjectType::HashMap => "HASH_MAP",
            ObjectType::Closure => "CLOSURE",
        };
        write!(f, "{}", out)
    }
}

/// A thin wrapper for all objects that implements `Object`.
///
/// This is the main structure that will be returned by all the evaluators.
#[derive(PartialEq, Eq, Hash, Clone)]
pub enum AllObjects {
    Integer(objects::Integer),
    StringObj(objects::StringObj),
    Boolean(objects::Boolean),
    Null(objects::Null),
    CompiledFunction(objects::CompiledFunctionObj),
    BuiltinFunction(objects::BuiltinFunctionObj),
    ArrayObj(objects::ArrayObj),
    HashMap(objects::HashMapObj),
    Closure(objects::Closure),
}

impl Object for AllObjects {
    fn inspect(&self) -> String {
        match self {
            Self::Integer(v) => v.inspect(),
            Self::StringObj(v) => v.inspect(),
            Self::Boolean(v) => v.inspect(),
            Self::Null(v) => v.inspect(),
            Self::CompiledFunction(v) => v.inspect(),
            Self::BuiltinFunction(v) => v.inspect(),
            Self::ArrayObj(v) => v.inspect(),
            Self::HashMap(v) => v.inspect(),
            Self::Closure(v) => v.inspect(),
        }
    }
}

impl AllObjects {
    pub fn object_type(&self) -> ObjectType {
        match self {
            Self::Integer(_) => ObjectType::Integer,
            Self::StringObj(_) => ObjectType::String,
            Self::Boolean(_) => ObjectType::Boolean,
            Self::Null(_) => ObjectType::Null,
            Self::CompiledFunction(_) => ObjectType::CompiledFunction,
            Self::BuiltinFunction(_) => ObjectType::BuiltInFunction,
            Self::ArrayObj(_) => ObjectType::Array,
            Self::HashMap(_) => ObjectType::HashMap,
            Self::Closure(_) => ObjectType::Closure,
        }
    }

    pub fn is_integer(&self) -> bool {
        self.object_type() == ObjectType::Integer
    }

    pub fn is_boolean(&self) -> bool {
        self.object_type() == ObjectType::Boolean
    }

    pub fn is_null(&self) -> bool {
        self.object_type() == ObjectType::Null
    }

    pub fn is_string(&self) -> bool {
        self.object_type() == ObjectType::String
    }
}
