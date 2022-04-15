use crate::{individual::Individual, LargeSignedInteger, SignedInteger};
use std::cmp::Ordering;
use std::rc::Rc;
use std::{cell::RefCell, ops::Deref};

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct AncestryIntersection {
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
    pub child: Individual,
    pub mapped_individual: Individual,
}

impl AncestryIntersection {
    pub fn new(
        left: LargeSignedInteger,
        right: LargeSignedInteger,
        child: Individual,
        mapped_individual: Individual,
    ) -> Self {
        assert!(left < right, "{} {}", left, right);
        Self {
            left,
            right,
            child,
            mapped_individual,
        }
    }
}

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
        // Sentinel -- FIXME: get rid of the need for the dummy Individuals
        intersections.push(AncestryIntersection::new(
            LargeSignedInteger::MAX - 1,
            LargeSignedInteger::MAX,
            // NOTE: dummy individual here to avoid using Option globally for
            // child field of Overlap
            Individual::new(SignedInteger::MAX, LargeSignedInteger::MAX),
            Individual::new(SignedInteger::MAX, LargeSignedInteger::MAX),
        ));
        let sorted = intersections.windows(2).all(|w| w[0].left <= w[1].left);
        assert!(sorted);
        let right = intersections[0].left;
        Self {
            intersections,
            overlaps: Rc::new(RefCell::new(overlaps)),
            j: 0,
            n,
            right,
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
            self.overlaps.borrow_mut().retain(|x| x.right > left);
            if self.overlaps.borrow().is_empty() {
                left = self.intersections[self.j].left;
            }
            while self.j < self.n && self.intersections[self.j].left == left {
                self.overlaps
                    .borrow_mut()
                    .push(self.intersections[self.j].clone());
                self.j += 1;
            }
            self.j -= 1;
            self.right = self
                .overlaps
                .borrow()
                .iter()
                .fold(LargeSignedInteger::MAX, |a, b| std::cmp::min(a, b.right));
            self.right = std::cmp::min(self.right, self.intersections[self.j + 1].right);
            self.j += 1;
            return Some((left, self.right, self.overlaps.clone()));
        }

        if !self.overlaps.borrow().is_empty() {
            let left = self.right;
            self.overlaps.borrow_mut().retain(|x| x.right > left);
            if !self.overlaps.borrow().is_empty() {
                self.right = self
                    .overlaps
                    .borrow()
                    .iter()
                    .fold(LargeSignedInteger::MAX, |a, b| std::cmp::min(a, b.right));
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

// NOTE: FIXME: a Trait called Interval would abstract out 
// a bunch of stuff

impl Ord for AncestryIntersection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.left.cmp(&other.left)
    }
}

impl PartialOrd for AncestryIntersection {
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
            AncestryIntersection::new(3, 4, Individual::new(1, 1), Individual::new(1, 2)),
            AncestryIntersection::new(2, 3, Individual::new(1, 2), Individual::new(1, 2)),
            AncestryIntersection::new(1, 2, Individual::new(1, 3), Individual::new(1, 2)),
        ];
        v.sort();
        assert!(v.windows(2).all(|w| w[0].left < w[1].left));
    }
}

#[cfg(test)]
mod overlapper_tests {
    use super::*;
    use crate::segment::Segment;

    #[test]
    fn test_single_overlap() {
        let mut parent = Individual::new(0, 0);

        let child1 = Individual::new(1, 1);
        let child2 = Individual::new(2, 1);

        {
            child1
                .borrow_mut()
                .ancestry
                .push(Segment::new(0, 5, child1.clone()));
            child2
                .borrow_mut()
                .ancestry
                .push(Segment::new(1, 6, child2.clone()));
        }

        parent.add_child_segment(0, 5, child1.clone());
        parent.add_child_segment(1, 6, child2.clone());

        let overlapper = AncestryOverlapper::new(parent.intersecting_ancestry());

        let expected = vec![vec![0, 5], vec![1, 6]];

        for (i, (left, right, _overlaps)) in overlapper.enumerate() {
            assert_eq!(expected[i][0], left);
            assert_eq!(expected[i][1], right);
        }
    }
}
