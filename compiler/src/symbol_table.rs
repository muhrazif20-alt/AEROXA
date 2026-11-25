use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub data_type: String,
    pub mutable: bool,
}

#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn declare(&mut self, symbol: Symbol) -> bool {
        let current = self.scopes.last_mut().unwrap();

        if current.contains_key(&symbol.name) {
            return false;
        }

        current.insert(symbol.name.clone(), symbol);
        true
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
    }

    pub fn lookup_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last()?.get(name)
    }

    pub fn lookup_outer_scope(&self, name: &str) -> Option<&Symbol> {
        if self.scopes.len() <= 1 {
            return None;
        }

        self.scopes[..self.scopes.len() - 1]
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
    }
}