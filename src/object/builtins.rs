use super::{
    objects::{BuiltinFunctionObj, Integer},
    AllObjects,
};
use anyhow::{anyhow, Result};

/// Defines an index for the builtin functions for the VM to access using an operand
pub static BUILTIN_FUNCTIONS: &[(usize, &str)] = &[
    (1, "len"),
    (2, "print"),
    (3, "push"),
    (4, "pop"),
    (5, "is_null"),
];

pub fn get_builtin_function(index: usize) -> Option<AllObjects> {
    let func = match index {
        1 => BuiltinFunctionObj::new("len", 1, len),
        _ => return None,
    };

    Some(AllObjects::BuiltinFunction(func))
}

/// Returns the length of a string, an array or a hashmap.
///
/// The function expects an argument called value, which must be one of the said types.
pub fn len(mut values: Vec<AllObjects>) -> Result<AllObjects> {
    let length = match values.remove(0) {
        AllObjects::StringObj(v) => v.value.len(),
        AllObjects::ArrayObj(v) => v.elements.borrow().len(),
        AllObjects::HashMap(v) => v.map.borrow().len(),
        v => {
            return Err(anyhow!(
                "value should either be an array, a hashmap or a string, received a {}",
                v.object_type()
            ))
        }
    };

    // panic of conversion from usize to i64 is highly unlikely
    let length = AllObjects::Integer(Integer {
        value: length.try_into()?,
    });

    Ok(length)
}
