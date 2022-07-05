use crate::LargeSignedInteger;
use crate::NodeFlags;
use crate::Segment;

pub struct AncestrySegment {
    pub segment: Segment,
    pub child: usize,
}

pub type ChildMap = nohash_hasher::IntMap<usize, Vec<Segment>>;
pub type ParentSet = nohash_hasher::IntSet<usize>;

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
        unimplemented!("have to have a mapping onto Self here, dawg...");
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
