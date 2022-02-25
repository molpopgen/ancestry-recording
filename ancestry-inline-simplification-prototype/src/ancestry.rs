use crate::{LargeSignedInteger, Segment, SignedInteger};
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub enum NodeStatus {
    ALIVE,
    DEAD,
}

pub struct Ancestry {
    next_node_id: SignedInteger,
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

        Self {
            next_node_id: num_nodes,
            status: vec![NodeStatus::ALIVE; num_nodes as usize],
            ancestry: vec![vec![]; num_nodes as usize],
            children: vec![vec![]; num_nodes as usize],
            parents: vec![HashSet::<SignedInteger>::default(); num_nodes as usize],
        }
    }
}
