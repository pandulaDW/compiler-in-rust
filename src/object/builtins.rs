/// Defines an index for the builtin functions for the VM to access using an operand
pub static BUILTIN_FUNCTIONS: &[(u8, &str)] = &[
    (1, "len"),
    (2, "print"),
    (3, "push"),
    (4, "pop"),
    (5, "is_null"),
];
