use crate::{NodeId, Position, SignedInteger, Time};

#[derive(Clone, Copy)]
pub struct Segment {
    pub descendant: NodeId,
    pub left: Position,
    pub right: Position,
}

pub struct AncestryRecord {
    pub node: NodeId,
    pub birth_time: Time,
    pub ancestry: Vec<Segment>,
    pub descendants: Vec<Segment>,
}

/// This is node table,
/// edge table, and ancestry
/// all rolled up into one.
pub struct Ancestry {
    pub genome_length: Position,
    pub ancestry: Vec<AncestryRecord>,
}

impl Segment {
    pub fn new(descendant: NodeId, left: Position, right: Position) -> Self {
        Self {
            descendant,
            left,
            right,
        }
    }
}

impl AncestryRecord {
    pub fn new(node: NodeId, birth_time: Time) -> Self {
        Self {
            node,
            birth_time,
            ancestry: vec![],
            descendants: vec![],
        }
    }

    pub fn new_from(
        node: NodeId,
        birth_time: Time,
        ancestry: Vec<Segment>,
        descendants: Vec<Segment>,
    ) -> Self {
        Self {
            node,
            birth_time,
            ancestry,
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

    pub(crate) fn get_mut(&mut self, node: NodeId) -> Option<&mut AncestryRecord> {
        self.ancestry.get_mut(node.value as usize)
    }

    /// Adding an "edge" during a sim
    pub fn record_transmission(
        &mut self,
        ancestor: NodeId,   // "parent" in tskit
        descendant: NodeId, // "child" in tskit
        left: Position,
        right: Position,
    ) {
        if let Some(record) = self.get_mut(ancestor) {
            record.descendants.push(Segment {
                descendant,
                left,
                right,
            });
        } else {
            panic!("{:?} has not been recorded as a node", ancestor);
        }
    }

    /// Adding a "node" during a sim
    pub fn record_node(&mut self, birth_time: Time) -> NodeId {
        assert!(self.ancestry.len() < SignedInteger::MAX as usize);
        let value = (self.ancestry.len() + 1) as SignedInteger;
        let rv = NodeId { value };
        let x = AncestryRecord::new(rv, birth_time);
        self.ancestry.push(x);
        rv
    }
}
