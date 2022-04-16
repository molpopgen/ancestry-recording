use crate::LargeSignedInteger;
use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct HalfOpenInterval {
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
}

impl HalfOpenInterval {
    pub fn new(left: LargeSignedInteger, right: LargeSignedInteger) -> Self {
        Self { left, right }
    }
}

// NOTE: FIXME: a Trait called Interval would abstract out
// a bunch of stuff

impl Ord for HalfOpenInterval {
    fn cmp(&self, other: &Self) -> Ordering {
        self.left.cmp(&other.left)
    }
}

impl PartialOrd for HalfOpenInterval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
