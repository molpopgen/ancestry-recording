use crate::{LargeSignedInteger, Segment, SignedInteger};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
pub enum NodeStatus {
    ALIVE,
    DEAD,
}

pub struct Ancestry {
    next_node_id: SignedInteger,
    node_to_index: HashMap<SignedInteger, usize>,
    pub status: Vec<NodeStatus>,
    pub ancestry: Vec<Vec<Segment>>,
    pub children: Vec<Vec<Segment>>,
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
            ancestry: vec![vec![]; num_nodes as usize],
            children: vec![vec![]; num_nodes as usize],
            parents: vec![HashSet::<SignedInteger>::default(); num_nodes as usize],
        }
    }
}
