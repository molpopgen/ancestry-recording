use crate::{Node, Position};

pub struct Descendant {
    pub descendant: Node,
    pub left: Position,
    pub right: Position,
}

pub struct AncestryRecord {
    pub ancestors: Vec<Node>,
    pub descendants: Vec<Descendant>,
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
    pub fn new(ancestors: Vec<Node>, descendants: Vec<Descendant>) -> Self {
        Self {
            ancestors,
            descendants,
        }
    }
}

impl Default for AncestryRecord {
    fn default() -> Self {
        Self {
            ancestors: vec![],
            descendants: vec![],
        }
    }
}
