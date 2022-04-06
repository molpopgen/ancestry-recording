use crate::individual::Individual;
use crate::LargeSignedInteger;
use std::cmp::Ordering;

/// A genomic segment, `[left, right)` inherited
/// by a `child`.
///
/// # Note
///
/// The only reason that the child is Option
/// is b/c of how we generate a Sentinel value
/// on the segment queue.  This causes serious API
/// clutter and we should fix this
#[derive(Clone, Eq, PartialEq)]
pub struct Segment {
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
    pub child: Option<Individual>,
}

impl Segment {
    pub fn new(
        left: LargeSignedInteger,
        right: LargeSignedInteger,
        child: Option<Individual>,
    ) -> Self {
        assert!(left < right, "{} {}", left, right);
        Self { left, right, child }
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> Ordering {
        self.left.cmp(&other.left)
    }
}

impl PartialOrd for Segment {
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
            Segment::new(3, 4, None),
            Segment::new(2, 3, None),
            Segment::new(1, 2, None),
        ];
        v.sort();
        assert!(v.windows(2).all(|w| w[0].left < w[1].left));
    }
}
