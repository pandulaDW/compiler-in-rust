use std::collections::HashMap;

type SymbolScope = &'static str;

const GLOBAL_SCOPE: &str = "GLOBAL";

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

#[derive(Clone)]
pub struct SymbolTable {
    store: HashMap<String, Symbol>,
    num_definitions: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            num_definitions: 0,
        }
    }

    /// Create and store a new `Symbol` definition
    pub fn define(&mut self, name: &str) -> Symbol {
        let symbol = Symbol::new(name, GLOBAL_SCOPE, self.num_definitions);
        self.store.insert(name.to_string(), symbol.clone());
        self.num_definitions += 1;
        symbol
    }

    /// Resolves and return the symbol associated with the name
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.store.get(name)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Symbol, SymbolTable, GLOBAL_SCOPE};

    #[test]
    fn test_define() {
        let mut expected = HashMap::new();
        expected.insert("a".to_string(), Symbol::new("a", GLOBAL_SCOPE, 0));
        expected.insert("b".to_string(), Symbol::new("b", GLOBAL_SCOPE, 1));

        let mut global = SymbolTable::new();
        let a = global.define("a");
        assert_eq!(a, *expected.get("a").unwrap());

        let b = global.define("b");
        assert_eq!(b, *expected.get("b").unwrap());
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
    }
}
