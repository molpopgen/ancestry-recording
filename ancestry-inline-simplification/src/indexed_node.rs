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

#[derive(Default)]
pub struct Node {
    /// Index of this node in the container
    pub index: usize,
    pub birth_time: LargeSignedInteger,
    pub flags: NodeFlags,
    pub parents: ParentSet,
    pub ancestry: Vec<AncestrySegment>,
    pub children: ChildMap,
}

impl Node {
    pub(crate) fn new_birth(
        index: usize,
        birth_time: LargeSignedInteger,
        parents: ParentSet,
    ) -> Self {
        Self {
            index,
            birth_time,
            flags: NodeFlags::new_alive(),
            parents,
            ancestry: vec![], // FIXME: should be a mapping to self, which needs genome length!
            children: ChildMap::default(),
        }
    }

    pub(crate) fn recycle(&mut self, birth_time: LargeSignedInteger, parents: ParentSet) {
        unimplemented!("recycle not implemented");
    }
}
