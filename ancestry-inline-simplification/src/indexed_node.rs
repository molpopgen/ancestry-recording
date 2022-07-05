use crate::LargeSignedInteger;
use crate::NodeFlags;
use crate::Segment;
use hashbrown::{HashMap, HashSet};

pub struct AncestrySegment {
    pub segment: Segment,
    pub child: usize,
}

pub type ChildMap = HashMap<usize, Vec<Segment>>;
pub type ParentSet = HashSet<usize>;

pub struct Node {
    /// Index of this node in the container
    pub index: usize,
    pub birth_time: LargeSignedInteger,
    pub flags: NodeFlags,
    pub parents: ParentSet,
    pub ancestry: Vec<AncestrySegment>,
    pub children: ChildMap,
}
