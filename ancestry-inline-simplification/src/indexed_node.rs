use crate::HalfOpenInterval;
use crate::LargeSignedInteger;
use crate::NodeFlags;
use crate::Segment;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AncestrySegment {
    pub segment: Segment,
    pub child: usize,
}

impl AncestrySegment {
    pub fn new(left: LargeSignedInteger, right: LargeSignedInteger, child: usize) -> Self {
        Self {
            segment: Segment::new_unchecked(left, right),
            child,
        }
    }
}

macro_rules! impl_half_open_interval {
    ($type: ty, $field: ident) => {
        impl HalfOpenInterval for $type {
            fn left(&self) -> LargeSignedInteger {
                self.$field.left()
            }
            fn right(&self) -> LargeSignedInteger {
                self.$field.right()
            }
        }
    };
}

impl_half_open_interval!(AncestrySegment, segment);

pub type ChildMap = nohash_hasher::IntMap<usize, Vec<Segment>>;
pub type ParentSet = nohash_hasher::IntSet<usize>;

#[derive(Default)]
pub struct NodeTable {
    pub index: Vec<usize>, // Redundant?
    pub counts: Vec<u32>,
    pub birth_time: Vec<LargeSignedInteger>,
    pub flags: Vec<NodeFlags>,
    pub parents: Vec<ParentSet>,
    pub ancestry: Vec<Vec<AncestrySegment>>,
    pub children: Vec<ChildMap>,
    queue: Vec<usize>, // for recycling
}

impl NodeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn new_birth(
        &mut self,
        birth_time: LargeSignedInteger,
        genome_length: LargeSignedInteger,
        parents: ParentSet,
    ) -> Result<usize, usize> {
        for p in &parents {
            if *p >= self.counts.len() {
                return Err(*p);
            }
            if self.birth_time[*p] >= birth_time {
                return Err(*p);
            }
        }
        match self.queue.pop() {
            Some(index) => {
                self.counts[index] = 1;
                self.birth_time[index] = birth_time;
                self.flags[index] = NodeFlags::new_alive();
                self.parents[index] = parents;
                self.ancestry[index].clear();
                self.ancestry[index].push(AncestrySegment {
                    segment: Segment::new(0, genome_length).unwrap(),
                    child: index,
                });
                self.children[index].clear();
                Ok(index)
            }
            None => {
                self.index.push(self.index.len());
                self.counts.push(1);
                self.birth_time.push(birth_time);
                self.flags.push(NodeFlags::new_alive());
                self.parents.push(parents);
                self.ancestry.push(vec![AncestrySegment {
                    segment: Segment::new(0, genome_length).unwrap(),
                    child: self.index.len() - 1,
                }]);
                self.children.push(ChildMap::default());
                Ok(self.index.len() - 1)
            }
        }
    }
}
