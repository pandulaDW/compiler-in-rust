use std::{collections::HashMap, rc::Rc};

type SymbolScope = &'static str;

const GLOBAL_SCOPE: &str = "GLOBAL";
const LOCAL_SCOPE: &str = "LOCAL";

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

#[derive(Clone, Default)]
pub struct SymbolTable {
    store: HashMap<String, Symbol>,
    num_definitions: usize,
    outer: Option<Rc<SymbolTable>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_enclosed(outer: Rc<SymbolTable>) -> Self {
        let mut s = Self::new();
        s.outer = Some(outer);
        s
    }

    /// Create and store a new `Symbol` definition
    pub fn define(&mut self, name: &str) -> Symbol {
        let mut symbol = Symbol::new(name, GLOBAL_SCOPE, self.num_definitions);
        if self.outer.is_some() {
            symbol.scope = LOCAL_SCOPE
        }

        self.store.insert(name.to_string(), symbol.clone());
        self.num_definitions += 1;
        symbol
    }

    /// Resolves and return the symbol associated with the name
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        let mut result = self.store.get(name);
        if result.is_none() && self.outer.is_some() {
            result = self.outer.as_ref().unwrap().resolve(name);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{Symbol, SymbolTable, GLOBAL_SCOPE, LOCAL_SCOPE};
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

        let mut global = SymbolTable::new();
        assert_eq!(global.define("a"), *expected.get("a").unwrap());
        assert_eq!(global.define("b"), *expected.get("b").unwrap());

        let mut first_local = SymbolTable::new_enclosed(Rc::new(global));
        assert_eq!(first_local.define("c"), *expected.get("c").unwrap());
        assert_eq!(first_local.define("d"), *expected.get("d").unwrap());

        let mut second_local = SymbolTable::new_enclosed(Rc::new(first_local));
        assert_eq!(second_local.define("e"), *expected.get("e").unwrap());
        assert_eq!(second_local.define("f"), *expected.get("f").unwrap());
    }

    #[test]
    fn test_resolve() {
        let mut global = SymbolTable::new();
        global.define("a");
        global.define("b");

        let expected = vec![
            Symbol::new("a", GLOBAL_SCOPE, 0),
            Symbol::new("b", GLOBAL_SCOPE, 1),
        ];

        for sym in expected.iter() {
            let resolved = global.resolve(&sym.name);
            assert!(resolved.is_some());
            assert_eq!(sym, resolved.unwrap());
        }

        assert_eq!(None, global.resolve("x"));
    }

    #[test]
    fn test_resolve_local() {
        let mut global = SymbolTable::new();
        global.define("a");
        global.define("b");

        let mut local = SymbolTable::new_enclosed(Rc::new(global));
        local.define("c");
        local.define("d");

        let expected = vec![
            Symbol::new("a", GLOBAL_SCOPE, 0),
            Symbol::new("b", GLOBAL_SCOPE, 1),
            Symbol::new("c", LOCAL_SCOPE, 0),
            Symbol::new("d", LOCAL_SCOPE, 1),
        ];

        for sym in expected.iter() {
            let resolved = local.resolve(&sym.name);
            assert!(resolved.is_some());
            assert_eq!(sym, resolved.unwrap());
        }
    }

    #[test]
    fn test_resolve_nested_and_local() {
        let mut global = SymbolTable::new();
        global.define("a");
        global.define("b");

        let mut first_local = SymbolTable::new_enclosed(Rc::new(global));
        first_local.define("c");
        first_local.define("d");
        let first_local_ref = Rc::new(first_local);

        let mut second_local = SymbolTable::new_enclosed(first_local_ref.clone());
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
                assert_eq!(&sym, resolved.unwrap());
            }
        }
    }
}
