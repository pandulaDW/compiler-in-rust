use crate::object::builtins::BUILTIN_FUNCTIONS;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

type SymbolScope = &'static str;

pub const GLOBAL_SCOPE: SymbolScope = "GLOBAL";
pub const LOCAL_SCOPE: SymbolScope = "LOCAL";
pub const BUILTIN_SCOPE: SymbolScope = "BUILTIN";
pub const FREE_SCOPE: SymbolScope = "FREE";
pub const FUNCTION_SCOPE: SymbolScope = "FUNCTION";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Symbol {
    pub name: String,
    pub scope: SymbolScope,
    pub index: usize,
}

impl Symbol {
    fn new(name: &str, scope: SymbolScope, index: usize) -> Self {
        Self {
            name: name.to_string(),
            scope,
            index,
        }
    }
}

/// A Wrapper around `SymbolTableDefinition` to give immutable references access to mutable symbol table methods
#[derive(Clone)]
pub struct SymbolTable {
    pub table: RefCell<SymbolTableDefinition>,
    pub outer: Option<Rc<SymbolTable>>,
    pub free_symbols: RefCell<Vec<Symbol>>,
}

impl SymbolTable {
    /// Creates a new symbol table and inserts the builtins
    pub fn new() -> Self {
        let s = SymbolTable {
            table: RefCell::new(SymbolTableDefinition::default()),
            outer: None,
            free_symbols: RefCell::new(vec![]),
        };

        for (i, v) in BUILTIN_FUNCTIONS {
            s.define_builtin(*i, v);
        }

        s
    }

    /// Creates a new symbol table with the given outer table as its attached outer table
    pub fn new_enclosed(outer: Rc<SymbolTable>) -> Self {
        let mut s = Self::new();
        s.outer = Some(outer);
        s
    }

    /// A wrapper around the `SymbolTableDefinition`'s `define` method
    pub fn define(&self, name: &str) -> Symbol {
        self.table.borrow_mut().define(name, self.outer.is_some())
    }

    /// Defines builtin functions in the BUILTIN_SCOPE
    pub fn define_builtin(&self, index: usize, name: &str) -> Symbol {
        let symbol = Symbol::new(name, BUILTIN_SCOPE, index);
        self.table
            .borrow_mut()
            .store
            .insert(name.to_string(), symbol.clone());
        symbol
    }

    /// Defines function names to resolve recursive functions properly
    pub fn define_function_name(&self, name: &str) -> Symbol {
        let symbol = Symbol::new(name, FUNCTION_SCOPE, 0);
        self.table
            .borrow_mut()
            .store
            .insert(name.to_string(), symbol.clone());
        symbol
    }

    /// Returns the symbol associated with the given name by recursively checking all the scopes
    ///
    /// It will also set the free variables, if found.
    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        let mut obj = self.table.borrow().store.get(name).cloned();

        if obj.is_none() && self.outer.is_some() {
            obj = self.outer.as_ref().unwrap().resolve(name);
            if obj.is_none() {
                return obj;
            }

            let scope = obj.as_ref().unwrap().scope;
            if scope == GLOBAL_SCOPE || scope == BUILTIN_SCOPE {
                return obj;
            }

            let free = self.define_free(obj.unwrap());
            return Some(free);
        }

        obj
    }

    /// Defines a free variable in the symbol-table's free variable holder
    pub fn define_free(&self, original: Symbol) -> Symbol {
        let symbol_name = original.name.clone();
        self.free_symbols.borrow_mut().push(original);

        let symbol = Symbol::new(
            &symbol_name,
            FREE_SCOPE,
            self.free_symbols.borrow().len() - 1,
        );

        self.table
            .borrow_mut()
            .store
            .insert(symbol_name, symbol.clone());

        symbol
    }
}

#[derive(Clone, Default)]
pub struct SymbolTableDefinition {
    store: HashMap<String, Symbol>,
    num_definitions: usize,
}

impl SymbolTableDefinition {
    /// Create and store a new `Symbol` definition
    fn define(&mut self, name: &str, outer_exists: bool) -> Symbol {
        let mut symbol = Symbol::new(name, GLOBAL_SCOPE, self.num_definitions);
        if outer_exists {
            symbol.scope = LOCAL_SCOPE
        }

        self.store.insert(name.to_string(), symbol.clone());
        self.num_definitions += 1;
        symbol
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Symbol, SymbolTable, BUILTIN_SCOPE, FREE_SCOPE, FUNCTION_SCOPE, GLOBAL_SCOPE, LOCAL_SCOPE,
    };
    use std::{collections::HashMap, rc::Rc};

    #[test]
    fn test_define() {
        let mut expected = HashMap::new();
        expected.insert("a".to_string(), Symbol::new("a", GLOBAL_SCOPE, 0));
        expected.insert("b".to_string(), Symbol::new("b", GLOBAL_SCOPE, 1));
        expected.insert("c".to_string(), Symbol::new("c", LOCAL_SCOPE, 0));
        expected.insert("d".to_string(), Symbol::new("d", LOCAL_SCOPE, 1));
        expected.insert("e".to_string(), Symbol::new("e", LOCAL_SCOPE, 0));
        expected.insert("f".to_string(), Symbol::new("f", LOCAL_SCOPE, 1));

        let global = SymbolTable::new();
        assert_eq!(global.define("a"), *expected.get("a").unwrap());
        assert_eq!(global.define("b"), *expected.get("b").unwrap());

        let first_local = SymbolTable::new_enclosed(Rc::new(global));
        assert_eq!(first_local.define("c"), *expected.get("c").unwrap());
        assert_eq!(first_local.define("d"), *expected.get("d").unwrap());

        let second_local = SymbolTable::new_enclosed(Rc::new(first_local));
        assert_eq!(second_local.define("e"), *expected.get("e").unwrap());
        assert_eq!(second_local.define("f"), *expected.get("f").unwrap());
    }

    #[test]
    fn test_resolve() {
        let global = SymbolTable::new();
        global.define("a");
        global.define("b");

        let expected = vec![
            Symbol::new("a", GLOBAL_SCOPE, 0),
            Symbol::new("b", GLOBAL_SCOPE, 1),
        ];

        for sym in expected {
            let resolved = global.resolve(&sym.name);
            assert!(resolved.is_some());
            assert_eq!(sym, resolved.unwrap());
        }

        assert_eq!(None, global.resolve("x"));
    }

    #[test]
    fn test_resolve_local() {
        let global = SymbolTable::new();
        global.define("a");
        global.define("b");

        let local = SymbolTable::new_enclosed(Rc::new(global));
        local.define("c");
        local.define("d");

        let expected = vec![
            Symbol::new("a", GLOBAL_SCOPE, 0),
            Symbol::new("b", GLOBAL_SCOPE, 1),
            Symbol::new("c", LOCAL_SCOPE, 0),
            Symbol::new("d", LOCAL_SCOPE, 1),
        ];

        for sym in expected {
            let resolved = local.resolve(&sym.name);
            assert!(resolved.is_some());
            assert_eq!(sym, resolved.unwrap());
        }
    }

    #[test]
    fn test_resolve_nested_and_local() {
        let global = SymbolTable::new();
        global.define("a");
        global.define("b");

        let first_local = SymbolTable::new_enclosed(Rc::new(global));
        first_local.define("c");
        first_local.define("d");
        let first_local_ref = Rc::new(first_local);

        let second_local = SymbolTable::new_enclosed(first_local_ref.clone());
        second_local.define("e");
        second_local.define("f");
        let second_local_ref = Rc::new(second_local);

        let test_cases = vec![(
            first_local_ref,
            vec![
                Symbol::new("a", GLOBAL_SCOPE, 0),
                Symbol::new("b", GLOBAL_SCOPE, 1),
                Symbol::new("c", LOCAL_SCOPE, 0),
                Symbol::new("d", LOCAL_SCOPE, 1),
            ],
            second_local_ref,
            vec![
                Symbol::new("a", GLOBAL_SCOPE, 0),
                Symbol::new("b", GLOBAL_SCOPE, 1),
                Symbol::new("c", LOCAL_SCOPE, 0),
                Symbol::new("e", LOCAL_SCOPE, 0),
                Symbol::new("f", LOCAL_SCOPE, 1),
            ],
        )];

        for tc in test_cases {
            for sym in tc.1 {
                let resolved = tc.0.resolve(&sym.name);
                assert!(resolved.is_some());
                assert_eq!(sym, resolved.unwrap());
            }
        }
    }

    #[test]
    fn test_define_resolve_builtins() {
        let global = SymbolTable::new();
        let global_ref = Rc::new(global);

        let first_local = SymbolTable::new_enclosed(global_ref.clone());
        let first_local_ref = Rc::new(first_local);

        let second_local = SymbolTable::new_enclosed(first_local_ref.clone());
        let second_local_ref = Rc::new(second_local);

        let expected = vec![
            Symbol::new("a", BUILTIN_SCOPE, 0),
            Symbol::new("c", BUILTIN_SCOPE, 1),
            Symbol::new("e", BUILTIN_SCOPE, 2),
            Symbol::new("f", BUILTIN_SCOPE, 3),
        ];

        for (i, v) in expected.iter().enumerate() {
            global_ref.define_builtin(i, &v.name);
        }

        for table in [global_ref, first_local_ref, second_local_ref] {
            for sym in &expected {
                let result = table.resolve(&sym.name);
                assert!(result.is_some());
                assert_eq!(*sym, result.unwrap());
            }
        }
    }

    #[test]
    fn test_resolve_free() {
        let global = SymbolTable::new();
        global.define("a");
        global.define("b");
        let global_ref = Rc::new(global);

        let first_local = SymbolTable::new_enclosed(global_ref.clone());
        first_local.define("c");
        first_local.define("d");
        let first_local_ref = Rc::new(first_local);

        let second_local = SymbolTable::new_enclosed(first_local_ref.clone());
        second_local.define("e");
        second_local.define("f");
        let second_local_ref = Rc::new(second_local);

        // (table, expected_symbols, expected_free_symbols)
        let test_cases = vec![
            (
                first_local_ref,
                vec![
                    Symbol::new("a", GLOBAL_SCOPE, 0),
                    Symbol::new("b", GLOBAL_SCOPE, 1),
                    Symbol::new("c", LOCAL_SCOPE, 0),
                    Symbol::new("d", LOCAL_SCOPE, 1),
                ],
                vec![],
            ),
            (
                second_local_ref,
                vec![
                    Symbol::new("a", GLOBAL_SCOPE, 0),
                    Symbol::new("b", GLOBAL_SCOPE, 1),
                    Symbol::new("c", FREE_SCOPE, 0),
                    Symbol::new("d", FREE_SCOPE, 1),
                    Symbol::new("e", LOCAL_SCOPE, 0),
                    Symbol::new("f", LOCAL_SCOPE, 1),
                ],
                vec![
                    Symbol::new("c", LOCAL_SCOPE, 0),
                    Symbol::new("d", LOCAL_SCOPE, 1),
                ],
            ),
        ];

        for tc in test_cases {
            for sym in tc.1 {
                let result = tc.0.resolve(&sym.name);
                assert!(result.is_some());
                assert_eq!(sym, result.unwrap());
            }

            assert_eq!(tc.0.free_symbols.borrow().len(), tc.2.len());

            for (i, sym) in tc.2.iter().enumerate() {
                let borrow = tc.0.free_symbols.borrow();
                let result = borrow.get(i);
                assert!(result.is_some());
                assert_eq!(sym, result.unwrap());
            }
        }
    }

    #[test]
    fn test_resolve_unresolvable_free() {
        let global = SymbolTable::new();
        global.define("a");
        let global_ref = Rc::new(global);

        let first_local = SymbolTable::new_enclosed(global_ref.clone());
        first_local.define("c");
        let first_local_ref = Rc::new(first_local);

        let second_local = SymbolTable::new_enclosed(first_local_ref.clone());
        second_local.define("e");
        second_local.define("f");

        // (table, expected_symbols, expected_free_symbols)
        let expected = vec![
            Symbol::new("a", GLOBAL_SCOPE, 0),
            Symbol::new("c", FREE_SCOPE, 0),
            Symbol::new("e", LOCAL_SCOPE, 0),
            Symbol::new("f", LOCAL_SCOPE, 1),
        ];

        for sym in expected {
            let result = second_local.resolve(&sym.name);
            assert!(result.is_some());
            assert_eq!(sym, result.unwrap());
        }

        for name in ["b", "d"] {
            let result = second_local.resolve(name);
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_define_and_resolve_function_name() {
        let global = SymbolTable::new();
        global.define_function_name("a");

        let expected = Symbol::new("a", FUNCTION_SCOPE, 0);
        let result = global.resolve(&expected.name);
        assert!(result.is_some());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_shadowing_function_name() {
        let global = SymbolTable::new();
        global.define_function_name("a");
        global.define("a");

        let expected = Symbol::new("a", GLOBAL_SCOPE, 0);
        let result = global.resolve(&expected.name);
        assert!(result.is_some());
        assert_eq!(expected, result.unwrap());
    }
}
