use super::{
    objects::{BuiltinFunctionObj, Integer, Null},
    AllObjects,
};
use crate::object::Object;
use anyhow::{anyhow, Result};

/// Defines an index for the builtin functions for the VM to access using an operand
pub static BUILTIN_FUNCTIONS: &[(usize, &str)] = &[
    (1, "len"),
    (2, "print"),
    (3, "push"),
    (4, "pop"),
    (5, "is_null"),
];

/// Return the builtin function associated with the passed index number
pub fn get_builtin_function(index: usize) -> Option<AllObjects> {
    let func = match index {
        1 => BuiltinFunctionObj::new("len", 1, len),
        2 => BuiltinFunctionObj::new("print", usize::MAX, print),
        3 => BuiltinFunctionObj::new("push", 2, push),
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
                "argument to `len` not supported, got {}",
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

/// Takes a variable number of arguments and prints each one consecutively to the stdout with a single space separator.
///
/// If no arguments are provided, it will print a newline.
pub fn print(args: Vec<AllObjects>) -> Result<AllObjects> {
    for (i, arg) in args.iter().enumerate() {
        print!("{}", arg.inspect());
        if i != args.len() - 1 {
            print!(" ");
        }
    }

    if args.is_empty() {
        println!();
    }

    Ok(AllObjects::Null(Null))
}

/// Appends an element to the back of the array
pub fn push(mut args: Vec<AllObjects>) -> Result<AllObjects> {
    let array = match args.remove(0) {
        AllObjects::ArrayObj(v) => v,
        v => {
            return Err(anyhow!(
                "argument to `push` not supported, got {}",
                v.object_type()
            ))
        }
    };

    // since all array borrows are temporary, this wouldn't cause a panic.
    array.elements.borrow_mut().push(args.remove(0));

    Ok(AllObjects::Null(Null))
}
