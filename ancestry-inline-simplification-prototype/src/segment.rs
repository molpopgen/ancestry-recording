use crate::individual::Individual;
use crate::LargeSignedInteger;
use crate::SignedInteger;
use std::cmp::Ordering;

#[derive(Clone, Eq, PartialEq)]
pub struct Segment {
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
    pub node: Option<Individual>,
}

impl Segment {
    pub fn new(
        left: LargeSignedInteger,
        right: LargeSignedInteger,
        node: Option<Individual>,
    ) -> Self {
        assert!(left < right, "{} {}", left, right);
        Self { left, right, node }
    }
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
        Some(self.cmp(other))
    }
}
