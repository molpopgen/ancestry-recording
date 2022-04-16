use crate::LargeSignedInteger;
use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Interval {
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
}

impl Interval {
    pub fn new(left: LargeSignedInteger, right: LargeSignedInteger) -> Self {
        Self { left, right }
    }
}

// NOTE: FIXME: a Trait called Interval would abstract out
// a bunch of stuff

impl Ord for Interval {
    fn cmp(&self, other: &Self) -> Ordering {
        self.left.cmp(&other.left)
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
