use std::fmt::{self, Display};

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
    Error,
    Return,
    Function,
    _BuiltInFunction,
    Array,
    HashMap,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let out = match self {
            ObjectType::Integer => "INTEGER",
            ObjectType::String => "STRING",
            ObjectType::Boolean => "BOOLEAN",
            ObjectType::Null => "NULL",
            ObjectType::Error => "ERROR",
            ObjectType::Return => "RETURN",
            ObjectType::Function => "FUNCTION",
            ObjectType::_BuiltInFunction => "BUILTIN_FUNCTION",
            ObjectType::Array => "ARRAY",
            ObjectType::HashMap => "HASH_MAP",
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
    _Error(objects::Error),
    _ReturnValue(Box<AllObjects>),
    _Function(objects::FunctionObj),
    _BuiltinFunction(objects::BuiltinFunctionObj),
    _ArrayObj(objects::ArrayObj),
    _HashMap(objects::HashMapObj),
}

impl Object for AllObjects {
    fn inspect(&self) -> String {
        match self {
            Self::Integer(v) => v.inspect(),
            Self::StringObj(v) => v.inspect(),
            Self::Boolean(v) => v.inspect(),
            Self::Null(v) => v.inspect(),
            Self::_Error(v) => v.inspect(),
            Self::_ReturnValue(v) => v.inspect(),
            Self::_Function(v) => v.inspect(),
            Self::_BuiltinFunction(v) => v.inspect(),
            Self::_ArrayObj(v) => v.inspect(),
            Self::_HashMap(v) => v.inspect(),
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
            Self::_Error(_) => ObjectType::Error,
            Self::_ReturnValue(_) => ObjectType::Return,
            Self::_Function(_) => ObjectType::Function,
            Self::_BuiltinFunction(_) => ObjectType::Function,
            Self::_ArrayObj(_) => ObjectType::Array,
            Self::_HashMap(_) => ObjectType::HashMap,
        }
    }

    pub fn _new_error(message: &str) -> Self {
        Self::_Error(objects::Error {
            message: message.to_string(),
        })
    }

    pub fn is_integer(&self) -> bool {
        self.object_type() == ObjectType::Integer
    }

    pub fn is_boolean(&self) -> bool {
        self.object_type() == ObjectType::Boolean
    }

    pub fn _is_null(&self) -> bool {
        self.object_type() == ObjectType::Null
    }

    pub fn _is_string(&self) -> bool {
        self.object_type() == ObjectType::String
    }

    pub fn _is_error(&self) -> bool {
        self.object_type() == ObjectType::Error
    }
}
