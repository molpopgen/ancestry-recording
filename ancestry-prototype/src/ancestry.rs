use std::cmp::Ordering;
use std::vec;

use crate::{LargeSignedInteger, SignedInteger};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Segment {
    pub node: SignedInteger,
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
}

impl Ord for Segment {
    // Flipped to create min heaps
    fn cmp(&self, other: &Self) -> Ordering {
        other.left.cmp(&self.left)
    }
}

impl PartialOrd for Segment {
    // Flipped to create min heaps
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

pub struct Parent {
    pub node: SignedInteger,
    pub birth_time: LargeSignedInteger,
    pub descendants: Vec<Segment>,
}

pub struct AncestryRecord {
    pub node: SignedInteger,
    pub ancestry: Vec<Segment>,
}

/// This is node table,
/// edge table, and ancestry
/// all rolled up into one.
pub struct Ancestry {
    pub genome_length: LargeSignedInteger,
    pub edges: Vec<Parent>,
    pub ancestry: Vec<AncestryRecord>,
}

impl Parent {
    pub fn new(node: SignedInteger, birth_time: LargeSignedInteger) -> Self {
        Self {
            node,
            birth_time,
            descendants: vec![],
        }
    }
}

impl Segment {
    pub fn new(node: SignedInteger, left: LargeSignedInteger, right: LargeSignedInteger) -> Self {
        assert!(left < right, "{} {}", left, right);
        assert!(node >= 0);
        Self { node, left, right }
    }
}

impl AncestryRecord {
    pub fn new(node: SignedInteger) -> Self {
        Self {
            node,
            ancestry: vec![],
        }
    }

    pub fn new_from(node: SignedInteger, ancestry: Vec<Segment>) -> Self {
        Self { node, ancestry }
    }
}

impl Ancestry {
    pub fn new(genome_length: LargeSignedInteger) -> Self {
        Self {
            genome_length,
            edges: vec![],
            ancestry: vec![],
        }
    }

    pub(crate) fn get_edges_mut(&mut self, node: SignedInteger) -> Option<&mut Parent> {
        self.edges.get_mut(node as usize)
    }

    /// Adding an "edge" during a sim
    pub fn record_transmission(
        &mut self,
        ancestor: SignedInteger,   // "parent" in tskit
        descendant: SignedInteger, // "child" in tskit
        left: LargeSignedInteger,
        right: LargeSignedInteger,
    ) {
        if let Some(record) = self.get_edges_mut(ancestor) {
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
        let value = self.ancestry.len() as SignedInteger;
        let x = AncestryRecord::new(value);
        self.ancestry.push(x);
        self.edges.push(Parent::new(value, birth_time));
        value
    }

    pub fn validate_post_simplification(&self) -> () {
        assert_eq!(self.edges.len(), self.ancestry.len());
        for (i, j) in self.edges.iter().zip(self.ancestry.iter()) {
            assert_eq!(i.node, j.node);
            let sorted = i.descendants.windows(2).all(|w| w[0].left <= w[1].left);
            assert!(sorted);
            let sorted = j.ancestry.windows(2).all(|w| w[0].left <= w[1].left);
            assert!(sorted);
        }
        assert!(self
            .edges
            .windows(2)
            .all(|w| w[0].birth_time <= w[1].birth_time));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BinaryHeap;

    #[test]
    fn test_binary_heap() {
        let mut v = vec![];
        v.push(Segment::new(0, 9, 10));
        v.push(Segment::new(0, 11, 12));
        v.push(Segment::new(0, 7, 10));

        let mut heap = BinaryHeap::from(v);

        heap.push(Segment::new(1, 8, 9));

        let x = heap.pop().unwrap();
        assert_eq!(x.left, 7);
        let x = heap.pop().unwrap();
        assert_eq!(x.left, 8);
        let x = heap.pop().unwrap();
        assert_eq!(x.left, 9);
        let x = heap.pop().unwrap();
        assert_eq!(x.left, 11);
    }
}
