use crate::individual::Individual;
use crate::LargeSignedInteger;
use std::cmp::Ordering;

/// A genomic segment, `[left, right)` inherited
/// by a `child`.
#[derive(Clone, Eq, PartialEq)]
pub struct AncestrySegment {
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
    pub child: Individual,
}

impl AncestrySegment {
    pub fn new(left: LargeSignedInteger, right: LargeSignedInteger, child: Individual) -> Self {
        assert!(left < right, "{} {}", left, right);
        Self { left, right, child }
    }
}

// NOTE: FIXME: a Trait called Interval would abstract out
// a bunch of stuff

impl Ord for AncestrySegment {
    fn cmp(&self, other: &Self) -> Ordering {
        self.left.cmp(&other.left)
    }
}

impl PartialOrd for AncestrySegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorting() {
        let mut v = vec![
            AncestrySegment::new(3, 4, Individual::new(1, 1)),
            AncestrySegment::new(2, 3, Individual::new(1, 2)),
            AncestrySegment::new(1, 2, Individual::new(1, 3)),
        ];
        v.sort();
        assert!(v.windows(2).all(|w| w[0].left < w[1].left));
    }
}
