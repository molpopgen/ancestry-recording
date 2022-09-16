use crate::node::Node;
use crate::InlineAncestryError;
use crate::LargeSignedInteger;
use std::cmp::Ordering;

pub(crate) trait HalfOpenInterval {
    fn left(&self) -> LargeSignedInteger;
    fn right(&self) -> LargeSignedInteger;
    fn overlaps<T: HalfOpenInterval>(&self, other: &T) -> bool {
        self.right() > other.left() && other.right() > self.left()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Segment {
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
}

impl Segment {
    pub fn new(
        left: LargeSignedInteger,
        right: LargeSignedInteger,
    ) -> Result<Segment, InlineAncestryError> {
        if left < 0 {
            Err(InlineAncestryError::InvalidPosition { p: left })
        } else if right < 0 {
            Err(InlineAncestryError::InvalidPosition { p: right })
        } else if right <= left {
            Err(InlineAncestryError::InvalidSegment { left, right })
        } else {
            Ok(Self::new_unchecked(left, right))
        }
    }

    pub(crate) fn new_unchecked(left: LargeSignedInteger, right: LargeSignedInteger) -> Self {
        debug_assert!(left < right, "{} {}", left, right);
        debug_assert!(left >= 0);
        Self { left, right }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AncestrySegment {
    pub segment: Segment,
    pub child: Node,
}

impl AncestrySegment {
    pub fn new(left: LargeSignedInteger, right: LargeSignedInteger, child: Node) -> Self {
        Self {
            segment: Segment::new_unchecked(left, right),
            child,
        }
    }
}

#[derive(Clone, Eq, Debug, PartialEq)]
pub(crate) struct AncestryIntersection {
    pub ancestry_segment: Segment,
    pub mapped_node: Node,
}

impl AncestryIntersection {
    pub fn new(left: LargeSignedInteger, right: LargeSignedInteger, mapped_node: Node) -> Self {
        Self {
            ancestry_segment: Segment::new(left, right).unwrap(),
            mapped_node,
        }
    }
}

impl HalfOpenInterval for Segment {
    fn left(&self) -> LargeSignedInteger {
        self.left
    }
    fn right(&self) -> LargeSignedInteger {
        self.right
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

macro_rules! impl_ord_partial_ord_for_half_open_interval {
    ($type: ty) => {
        impl Ord for $type {
            fn cmp(&self, other: &Self) -> Ordering {
                self.left().cmp(&other.left())
            }
        }

        impl PartialOrd for $type {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
    };
}

impl_half_open_interval!(AncestrySegment, segment);
impl_half_open_interval!(AncestryIntersection, ancestry_segment);

impl_ord_partial_ord_for_half_open_interval!(Segment);
impl_ord_partial_ord_for_half_open_interval!(AncestrySegment);
impl_ord_partial_ord_for_half_open_interval!(AncestryIntersection);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorting_ancestry_segment() {
        let mut v = vec![
            AncestrySegment::new(3, 4, Node::new_alive(1, 1)),
            AncestrySegment::new(2, 3, Node::new_alive(1, 2)),
            AncestrySegment::new(1, 2, Node::new_alive(1, 3)),
        ];
        v.sort();
        assert!(v.windows(2).all(|w| w[0].left() < w[1].left()));
    }

    #[test]
    fn test_sorting_ancestry_intersection() {
        let mut v = vec![
            AncestryIntersection::new(3, 4, Node::new_alive(1, 2)),
            AncestryIntersection::new(2, 3, Node::new_alive(1, 2)),
            AncestryIntersection::new(1, 2, Node::new_alive(1, 2)),
        ];
        v.sort();
        assert!(v.windows(2).all(|w| w[0].left() < w[1].left()));
    }
}
