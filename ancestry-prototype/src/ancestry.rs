use crate::{NodeId, Position};

pub struct Descendant {
    pub descendant: NodeId,
    pub left: Position,
    pub right: Position,
}

pub struct AncestryRecord {
    node: NodeId,
    pub ancestors: Vec<NodeId>,
    pub descendants: Vec<Descendant>,
}

pub struct Ancestry {
    pub genome_length: Position,
    pub ancestry: Vec<AncestryRecord>,
}

impl Descendant {
    pub fn new(descendant: NodeId, left: Position, right: Position) -> Self {
        Self {
            descendant,
            left,
            right,
        }
    }
}

impl AncestryRecord {
    pub fn new(node: NodeId) -> Self {
        Self {
            node,
            ancestors: vec![],
            descendants: vec![],
        }
    }

    pub fn new_from(node: NodeId, ancestors: Vec<NodeId>, descendants: Vec<Descendant>) -> Self {
        Self {
            node,
            ancestors,
            descendants,
        }
    }
}

impl Ancestry {
    pub fn new(genome_length: Position) -> Self {
        Self {
            genome_length,
            ancestry: vec![],
        }
    }

    pub fn record_descendant(
        &mut self,
        ancestor: NodeId,   // "parent" in tskit
        descendant: NodeId, // "child" in tskit
        left: Position,
        right: Position,
    ) {
        if let Some(record) = self.ancestry.get_mut(&ancestor) {
            record.descendants.push(Descendant {
                descendant,
                left,
                right,
            });
        } else {
            let x = self.ancestry.insert(
                ancestor,
                AncestryRecord::new_from(
                    vec![ancestor],
                    vec![Descendant {
                        descendant,
                        left,
                        right,
                    }],
                ),
            );
            assert!(x.is_none());
        }
    }
}

// Implement traits here

// impl Default for AncestryRecord {
//     fn default() -> Self {
//         Self {
//             ancestors: vec![],
//             descendants: vec![],
//         }
//     }
// }

//impl Default for Ancestry {
//    fn default() -> Self {
//        Self {
//            ancestry: HashMap::new(),
//        }
//    }
//}
