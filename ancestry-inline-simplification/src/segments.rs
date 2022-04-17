use crate::individual::Individual;
use crate::LargeSignedInteger;
use std::cmp::Ordering;

pub(crate) trait HalfOpenInterval {
    fn left(&self) -> LargeSignedInteger;
    fn right(&self) -> LargeSignedInteger;
    fn overlaps<T: HalfOpenInterval>(&self, other: &T) -> bool {
        self.right() > other.left() && other.right() > self.left()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Segment {
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
}

impl Segment {
    pub fn new(left: LargeSignedInteger, right: LargeSignedInteger) -> Self {
        assert!(left < right, "{} {}", left, right);
        assert!(left >= 0);
        Self { left, right }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct AncestrySegment {
    pub segment: Segment,
    pub child: Individual,
}

impl AncestrySegment {
    pub fn new(left: LargeSignedInteger, right: LargeSignedInteger, child: Individual) -> Self {
        assert!(left < right, "{} {}", left, right);
        Self {
            segment: Segment::new(left, right),
            child,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct AncestryIntersection {
    pub ancestry_segment: AncestrySegment,
    pub mapped_individual: Individual,
}

impl AncestryIntersection {
    pub fn new(
        left: LargeSignedInteger,
        right: LargeSignedInteger,
        child: Individual,
        mapped_individual: Individual,
    ) -> Self {
        Self {
            ancestry_segment: AncestrySegment::new(left, right, child),
            mapped_individual,
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
       impl HalfOpenInterval for $ type {
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
