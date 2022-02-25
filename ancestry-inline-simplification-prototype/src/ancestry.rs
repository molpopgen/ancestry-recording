use crate::{LargeSignedInteger, Segment, SignedInteger};
use std::{collections::{HashMap, HashSet}, thread::panicking};

#[derive(Clone, Copy)]
pub enum NodeStatus {
    ALIVE,
    DEAD,
}

pub type ChildMap = HashMap<SignedInteger, Vec<Segment>>;

pub struct Ancestry {
    next_node_id: SignedInteger,
    node_to_index: HashMap<SignedInteger, usize>,
    pub status: Vec<NodeStatus>,
    pub birth_time: Vec<SignedInteger>,
    pub ancestry: Vec<Vec<Segment>>,
    pub children: Vec<ChildMap>,
    // TODO: replace hashing function,
    // as the default is slower than we
    // need b/c "crypto strength".
    pub parents: Vec<HashSet<SignedInteger>>,
}

impl Ancestry {
    pub fn new(num_nodes: SignedInteger) -> Self {
        assert!(num_nodes > 0);

        let mut node_to_index = HashMap::<SignedInteger, usize>::default();

        for i in 0..num_nodes {
            node_to_index.insert(i, i as usize);
        }

        Self {
            next_node_id: num_nodes,
            node_to_index,
            status: vec![NodeStatus::ALIVE; num_nodes as usize],
            birth_time: vec![0; num_nodes as usize],
            ancestry: vec![vec![]; num_nodes as usize],
            children: vec![ChildMap::default(); num_nodes as usize],
            parents: vec![HashSet::<SignedInteger>::default(); num_nodes as usize],
        }
    }

    pub fn add_child_segment(&mut self, parent: SignedInteger, child: SignedInteger, left: LargeSignedInteger, right: LargeSignedInteger){
        if let Some(p) = self.node_to_index.get_mut(&parent) {
            let seg = Segment::new(-1, left, right);
            if let Some(c) = self.children[*p].get_mut(&child) {
                // TODO: doc why we use a "null" node ID here...
                c.push(seg);
            }
            else {
                self.children[*p].insert(child, vec![seg]);
            }
        } else {
            panic!("parent {} does not exist", parent);
        }
    }

    /// Returns the "ID" of a new node
    pub fn add_node(&mut self) -> SignedInteger {
        if let Some(_) = self.node_to_index.get(&self.next_node_id) {
            panic!("We've done something quite wrong...");
        }
        let len = self.len();
        if len >= SignedInteger::MAX as usize {
            panic!("we have run out of SignedInteger!");
        }
        self.node_to_index.insert(self.next_node_id, len);
        let rv = self.next_node_id;
        self.next_node_id+=1;
        rv
    }

    // "private" fns (for now at least...)
    fn len(&self) -> usize {
        assert_eq!(self.status.len(), self.ancestry.len());
        assert_eq!(self.status.len(), self.birth_time.len());
        assert_eq!(self.status.len(), self.children.len());
        assert_eq!(self.status.len(), self.parents.len());

        self.status.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ancestry_new() {
        let a = Ancestry::new(10);
        assert_eq!(a.next_node_id, 10);
        assert_eq!(a.len(), 10);

        for i in 0..10 {
            assert_eq!(a.node_to_index.get(&i).unwrap(), &(i as usize));
        }
    }
}
