use crate::{LargeSignedInteger, SignedInteger};

#[derive(Clone, Copy)]
pub struct Segment {
    pub node: SignedInteger,
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
}

pub struct AncestryRecord {
    pub node: SignedInteger,
    pub birth_time: LargeSignedInteger,
    pub ancestry: Vec<Segment>,
    pub descendants: Vec<Segment>,
}

/// This is node table,
/// edge table, and ancestry
/// all rolled up into one.
pub struct Ancestry {
    pub genome_length: LargeSignedInteger,
    pub ancestry: Vec<AncestryRecord>,
}

impl Segment {
    pub fn new(node: SignedInteger, left: LargeSignedInteger, right: LargeSignedInteger) -> Self {
        assert!(left < right);
        assert!(node >= 0);
        Self { node, left, right }
    }
}

impl AncestryRecord {
    pub fn new(node: SignedInteger, birth_time: LargeSignedInteger) -> Self {
        Self {
            node,
            birth_time,
            ancestry: vec![],
            descendants: vec![],
        }
    }

    pub fn new_from(
        node: SignedInteger,
        birth_time: LargeSignedInteger,
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
    pub fn new(genome_length: LargeSignedInteger) -> Self {
        Self {
            genome_length,
            ancestry: vec![],
        }
    }

    pub(crate) fn get_mut(&mut self, node: SignedInteger) -> Option<&mut AncestryRecord> {
        self.ancestry.get_mut(node as usize)
    }

    /// Adding an "edge" during a sim
    pub fn record_transmission(
        &mut self,
        ancestor: SignedInteger,   // "parent" in tskit
        descendant: SignedInteger, // "child" in tskit
        left: LargeSignedInteger,
        right: LargeSignedInteger,
    ) {
        if let Some(record) = self.get_mut(ancestor) {
            record.descendants.push(Segment {
                node: descendant,
                left,
                right,
            });
        } else {
            panic!("{:?} has not been recorded as a node", ancestor);
        }
    }

    /// Adding a "node" during a sim
    pub fn record_node(&mut self, birth_time: LargeSignedInteger) -> SignedInteger {
        assert!(self.ancestry.len() < SignedInteger::MAX as usize);
        let value = (self.ancestry.len() + 1) as SignedInteger;
        let x = AncestryRecord::new(value, birth_time);
        self.ancestry.push(x);
        value
    }
}
