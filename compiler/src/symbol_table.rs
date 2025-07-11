use std::collections::HashMap;

use crate::writer::Segment;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    Static,
    Field,
    Arg,
    Var,
    None,
}

impl Kind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "static" => Kind::Static,
            "field" => Kind::Field,
            "arg" => Kind::Arg,
            "var" => Kind::Var,
            _ => Kind::None,
        }
    }
}

impl From<Kind> for Segment {
    fn from(kind: Kind) -> Segment {
        match kind {
            Kind::Static => Segment::Static,
            Kind::Field => Segment::This,
            Kind::Arg => Segment::Argument,
            Kind::Var => Segment::Local,
            Kind::None => panic!("Invalid kind: None"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub type_: String,
    pub kind: Kind,
    pub index: usize,
}

pub struct SymbolTable {
    table: HashMap<String, Symbol>,
    kind_counters: HashMap<Kind, usize>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut kind_counters = HashMap::new();
        kind_counters.insert(Kind::Static, 0);
        kind_counters.insert(Kind::Field, 0);
        kind_counters.insert(Kind::Arg, 0);
        kind_counters.insert(Kind::Var, 0);
        SymbolTable {
            table: HashMap::new(),
            kind_counters,
        }
    }

    pub fn reset(&mut self) {
        self.table.clear();
        for counter in self.kind_counters.values_mut() {
            *counter = 0;
        }
    }

    pub fn define(&mut self, name: String, type_: String, kind: Kind) {
        if kind == Kind::None {
            return;
        }
        let index = self.kind_counters[&kind];
        self.table.insert(
            name.clone(),
            Symbol {
                name,
                type_,
                kind: kind.clone(),
                index,
            },
        );
        self.kind_counters.insert(kind, index + 1);
    }

    pub fn var_count(&self, kind: Kind) -> usize {
        *self.kind_counters.get(&kind).unwrap_or(&0)
    }

    pub fn get(&self, name: &str) -> Option<Symbol> {
        self.table.get(name).cloned()
    }
}
