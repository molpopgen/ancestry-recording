use crate::{
    indexed_node::NodeTable, HalfOpenInterval, LargeSignedInteger, Segment, SignedInteger,
};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AncestryIntersection {
    pub ancestry_segment: AncestrySegment,
    pub mapped_node: usize,
}

impl AncestryIntersection {
    pub fn new(
        left: LargeSignedInteger,
        right: LargeSignedInteger,
        child: usize,
        mapped_node: usize,
    ) -> Self {
        Self {
            ancestry_segment: AncestrySegment::new(left, right, child),
            mapped_node,
        }
    }
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
impl_half_open_interval!(AncestryIntersection, ancestry_segment);

// impl_ord_partial_ord_for_half_open_interval!(Segment);
impl_ord_partial_ord_for_half_open_interval!(AncestrySegment);
impl_ord_partial_ord_for_half_open_interval!(AncestryIntersection);

///
/// # Panics
///
/// During iteration, unexpected internal errors will cause panic.
/// Such cases are definitely bugs that should be reported.
pub(crate) struct AncestryOverlapper {
    intersections: Vec<AncestryIntersection>,
    overlaps: Rc<RefCell<Vec<AncestryIntersection>>>, // Prevents copying the segments over and over
    j: usize,
    n: usize,
    right: LargeSignedInteger,
}

impl AncestryOverlapper {
    pub(crate) fn new(intersections: Vec<AncestryIntersection>) -> Self {
        let mut intersections = intersections;
        let n = intersections.len();
        let overlaps = vec![];

        intersections.sort();
        let sorted = intersections.windows(2).all(|w| {
            w[0].left() <= w[1].left() && w[0].left() < w[0].right() && w[1].left() < w[1].right()
        });
        assert!(sorted);
        // Sentinel -- FIXME: get rid of the need for the dummy Nodes
        intersections.push(AncestryIntersection::new(
            LargeSignedInteger::MAX - 1,
            LargeSignedInteger::MAX,
            // NOTE: dummy node here to avoid using Option globally for
            // child field of Overlap
            usize::MAX,
            usize::MAX,
        ));
        let right = intersections[0].left();
        Self {
            intersections,
            overlaps: Rc::new(RefCell::new(overlaps)),
            j: 0,
            n,
            right,
        }
    }

    fn min_right_in_overlaps(&self) -> Option<LargeSignedInteger> {
        if !self.overlaps.borrow().is_empty() {
            Some(
                self.overlaps
                    .borrow()
                    .iter()
                    .fold(LargeSignedInteger::MAX, |a, b| std::cmp::min(a, b.right())),
            )
        } else {
            None
        }
    }
}

impl Iterator for AncestryOverlapper {
    type Item = (
        LargeSignedInteger,
        LargeSignedInteger,
        Rc<RefCell<Vec<AncestryIntersection>>>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.j < self.n {
            let mut left = self.right;
            self.overlaps.borrow_mut().retain(|x| x.right() > left);
            if self.overlaps.borrow().is_empty() {
                left = self.intersections[self.j].left();
            }
            while self.j < self.n && self.intersections[self.j].left() == left {
                self.overlaps
                    .borrow_mut()
                    .push(self.intersections[self.j].clone());
                self.j += 1;
            }
            self.j -= 1;
            self.right = self.min_right_in_overlaps().unwrap();
            self.right = std::cmp::min(self.right, self.intersections[self.j + 1].left());
            self.j += 1;
            assert!(
                self.right > left,
                "ancestry overlapper failure: left = {}, right = {}, overlaps = {:?}, intersections = {:?}",
                left,
                self.right,
                self.overlaps
                    .borrow()
                    .iter()
                    .map(|a| (a.left(), a.right()))
                    .collect::<Vec<(i64, i64)>>(),
                self.intersections.iter().map(|a| (a.left(), a.right())).collect::<Vec<(i64, i64)>>()
            );
            return Some((left, self.right, self.overlaps.clone()));
        }

        if !self.overlaps.borrow().is_empty() {
            let left = self.right;
            self.overlaps.borrow_mut().retain(|x| x.right() > left);
            if !self.overlaps.borrow().is_empty() {
                self.right = self.min_right_in_overlaps().unwrap();
                return Some((left, self.right, self.overlaps.clone()));
            }
        }

        None

        // TODO: see of this code also works.  It is a cleaner way to do, I think.
        //if !self.segments.is_empty() {
        //    let mut left = self.right;
        //    self.overlaps.borrow_mut().retain(|x| x.right > left);
        //    if self.overlaps.borrow().is_empty() {
        //        left = self.segments.last().unwrap().left;
        //    }
        //    while !self.segments.is_empty() && self.segments.last().unwrap().left == left {
        //        let x = self.segments.pop().unwrap();
        //        self.overlaps.borrow_mut().push(x);
        //    }
        //    self.right = self
        //        .overlaps
        //        .borrow()
        //        .iter()
        //        .fold(LargeSignedInteger::MAX, |a, b| std::cmp::min(a, b.right));
        //    if let Some(seg) = self.segments.last() {
        //        self.right = std::cmp::min(self.right, seg.right);
        //    }
        //}
    }
}

#[cfg(test)]
mod overlapper_tests {
    use super::*;
    use crate::AncestrySegment;

    struct FailingExamples {
        data: Vec<Vec<(i64, i64)>>,
    }

    impl FailingExamples {
        fn new() -> Self {
            let data = vec![vec![
                (0_i64, 69_i64),
                (0, 100),
                (60, 69),
                (69, 100),
                (69, 100),
            ]];
            Self { data }
        }

        fn convert_next(&mut self) -> Option<Vec<AncestryIntersection>> {
            match self.data.pop() {
                Some(pos) => Some(
                    pos.into_iter()
                        .map(|p| AncestryIntersection::new(p.0, p.1, 0, 0))
                        .collect::<Vec<AncestryIntersection>>(),
                ),
                None => None,
            }
        }
    }

    impl Iterator for FailingExamples {
        type Item = Vec<AncestryIntersection>;

        fn next(&mut self) -> Option<Self::Item> {
            self.convert_next()
        }
    }

    #[test]
    fn test_failing_examples_discovered_during_development() {
        let mut examples = FailingExamples::new();
        for a in examples {
            let overlapper = AncestryOverlapper::new(a);
            for (_i, (_left, _right, _overlaps)) in overlapper.enumerate() {}
        }
    }
}
