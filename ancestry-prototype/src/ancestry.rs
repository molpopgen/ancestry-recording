use crate::{Node, Position};
use std::collections::HashMap;

pub struct Descendant {
    pub descendant: Node,
    pub left: Position,
    pub right: Position,
}

pub struct AncestryRecord {
    pub ancestors: Vec<Node>,
    pub descendants: Vec<Descendant>,
}

pub struct Ancestry {
    pub ancestry: HashMap<Node, AncestryRecord>,
}

impl Descendant {
    pub fn new(descendant: Node, left: Position, right: Position) -> Self {
        Self {
            descendant,
            left,
            right,
        }
    }
}

impl AncestryRecord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_from(ancestors: Vec<Node>, descendants: Vec<Descendant>) -> Self {
        Self {
            ancestors,
            descendants,
        }
    }
}

impl Ancestry {
    pub fn new() -> Self {
        Self::default()
    }
}

// Implement traits here

impl Default for AncestryRecord {
    fn default() -> Self {
        Self {
            ancestors: vec![],
            descendants: vec![],
        }
    }
}

impl Default for Ancestry {
    fn default() -> Self {
        Self {
            ancestry: HashMap::new(),
        }
    }
}
