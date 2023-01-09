use super::{
    objects::{Boolean, BuiltinFunctionObj, Integer, Null},
    AllObjects, ObjectType,
};
use crate::object::Object;
use anyhow::{anyhow, Result};
use std::{thread, time::Duration};

/// Defines an index for the builtin functions for the VM to access using an operand
pub static BUILTIN_FUNCTIONS: &[(usize, &str)] = &[
    (1, "len"),
    (2, "print"),
    (3, "push"),
    (4, "pop"),
    (5, "is_null"),
    (6, "insert"),
    (7, "delete"),
    (8, "sleep"),
];

/// Return the builtin function associated with the passed index number
pub fn get_builtin_function(index: usize) -> Option<AllObjects> {
    let func = match index {
        1 => BuiltinFunctionObj::new("len", 1, len),
        2 => BuiltinFunctionObj::new("print", usize::MAX, print),
        3 => BuiltinFunctionObj::new("push", 2, push),
        4 => BuiltinFunctionObj::new("pop", 1, pop),
        5 => BuiltinFunctionObj::new("is_null", 1, is_null),
        6 => BuiltinFunctionObj::new("insert", 3, insert),
        7 => BuiltinFunctionObj::new("delete", 2, delete),
        8 => BuiltinFunctionObj::new("sleep", 1, sleep),
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
        v => return Err(err_argument_not_supported("len", v.object_type())),
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
        v => return Err(err_argument_not_supported("push", v.object_type())),
    };

    // since all array borrows are temporary, this wouldn't cause a panic.
    array.elements.borrow_mut().push(args.remove(0));

    Ok(AllObjects::Null(Null))
}

/// Removes the last element from an array and returns it.
///
/// Returns null, if the array is empty
pub fn pop(mut args: Vec<AllObjects>) -> Result<AllObjects> {
    let array = match args.remove(0) {
        AllObjects::ArrayObj(v) => v,
        v => return Err(err_argument_not_supported("pop", v.object_type())),
    };

    // since all array borrows are temporary, this wouldn't cause a panic.
    let popped = match array.elements.borrow_mut().pop() {
        Some(v) => v,
        None => AllObjects::Null(Null),
    };

    Ok(popped)
}

/// Checks if the passed value is a null
pub fn is_null(mut args: Vec<AllObjects>) -> Result<AllObjects> {
    let is_null = matches!(args.remove(0), AllObjects::Null(_));
    Ok(AllObjects::Boolean(Boolean { value: is_null }))
}

/// Inserts a key-value pair into the map.
///
/// If the map did not have this key present, Null is returned.
///
/// If the map have this key present, the value is updated, and the old value is returned
pub fn insert(mut args: Vec<AllObjects>) -> Result<AllObjects> {
    let map_arg = args.remove(0);
    let key = args.remove(0);
    let value = args.remove(0);

    let m = match map_arg {
        AllObjects::HashMap(v) => v,
        v => return Err(err_argument_not_supported("insert", v.object_type())),
    };

    if let Some(v) = m.map.borrow_mut().insert(key, value) {
        return Ok(v);
    }

    Ok(AllObjects::Null(Null))
}

/// Removes a key from the map, returning the value at the key if the key was previously in the map and
/// returns Null otherwise
pub fn delete(mut args: Vec<AllObjects>) -> Result<AllObjects> {
    let m = match args.remove(0) {
        AllObjects::HashMap(v) => v,
        v => return Err(err_argument_not_supported("delete", v.object_type())),
    };
    let key = args.remove(0);

    if let Some(v) = m.map.borrow_mut().remove(&key) {
        return Ok(v);
    }

    Ok(AllObjects::Null(Null))
}

/// Puts the main thread to sleep for the specified amount of time given in seconds
pub fn sleep(mut args: Vec<AllObjects>) -> Result<AllObjects> {
    let seconds = match args.remove(0) {
        AllObjects::Integer(n) => n,
        v => return Err(err_argument_not_supported("sleep", v.object_type())),
    };

    let Ok(seconds) = TryInto::<u64>::try_into(seconds.value) else {
        return Err(anyhow!("sleep only takes a positive integer value"));
    };

    thread::sleep(Duration::from_secs(seconds));

    Ok(AllObjects::Null(Null))
}

fn err_argument_not_supported(fn_name: &str, obj_type: ObjectType) -> anyhow::Error {
    anyhow!("argument to `{fn_name}` not supported, got {obj_type}")
}
