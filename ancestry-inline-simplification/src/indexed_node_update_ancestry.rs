use crate::{
    indexed_node::AncestrySegment, indexed_node::NodeTable, HalfOpenInterval, LargeSignedInteger,
    Segment, SignedInteger,
};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

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

#[inline(never)]
fn intersecting_ancestry(
    node_index: usize,
    ancestry: &[Vec<crate::indexed_node::AncestrySegment>],
    children: &[crate::indexed_node::ChildMap],
) -> Vec<AncestryIntersection> {
    let mut rv = vec![];

    for (child, segs) in &children[node_index] {
        assert!(!segs.is_empty());
        for seg in segs.iter() {
            for x in ancestry[*child].iter() {
                if x.overlaps(seg) {
                    rv.push(AncestryIntersection::new(
                        std::cmp::max(x.left(), seg.left()),
                        std::cmp::min(x.right(), seg.right()),
                        *child,
                        x.child,
                    ));
                }
            }
        }
    }

    rv
}

#[inline(never)]
fn make_overlapper(
    node_index: usize,
    ancestry: &[Vec<crate::indexed_node::AncestrySegment>],
    children: &[crate::indexed_node::ChildMap],
) -> AncestryOverlapper {
    let intersection = intersecting_ancestry(node_index, ancestry, children);
    AncestryOverlapper::new(intersection)
}

#[inline(never)]
fn update_child_segments(
    node_index: usize,
    child: usize,
    left: LargeSignedInteger,
    right: LargeSignedInteger,
    children: &mut [crate::indexed_node::ChildMap],
) {
    assert_ne!(node_index, child);
    match children[node_index].get_mut(&child) {
        Some(segs) => {
            let need_push = match segs.last_mut() {
                Some(seg) => {
                    if seg.right == left {
                        seg.right = right; // Squash child segs as we go.
                        false
                    } else {
                        true
                    }
                }
                None => true,
            };
            if need_push {
                let seg = Segment::new(left, right).unwrap();
                segs.push(seg);
            }
        }
        None => {
            let seg = Segment::new(left, right).unwrap();
            children[node_index].insert(child, vec![seg]);
        }
    }
}

#[inline(never)]
fn process_overlaps(
    input_ancestry_len: usize,
    node_index: usize,
    flags: &[crate::NodeFlags],
    output_ancestry_index: &mut usize,
    ancestry_change_detected: &mut bool,
    overlapper: &mut AncestryOverlapper,
    children: &mut [crate::indexed_node::ChildMap],
    ancestry: &mut [Vec<crate::indexed_node::AncestrySegment>],
) {
    for (left, right, overlaps) in overlapper {
        assert!(left < right);
        let mut mapped_node = node_index;
        let borrowed_overlaps = overlaps.borrow();

        if borrowed_overlaps.len() == 1 {
            mapped_node = borrowed_overlaps[0].mapped_node;
            if flags[node_index].is_alive() {
                update_child_segments(node_index, mapped_node, left, right, children);
            }
        } else {
            debug_assert_eq!(mapped_node, node_index);
            for overlap in borrowed_overlaps.iter() {
                update_child_segments(node_index, overlap.mapped_node, left, right, children);
            }
        }
        if !flags[node_index].is_alive() {
            if *output_ancestry_index < input_ancestry_len {
                // SAFETY: we just checked the bounds
                let input_ancestry_seg =
                    unsafe { ancestry[node_index].get_unchecked_mut(*output_ancestry_index) };
                if input_ancestry_seg.left() != left
                    || input_ancestry_seg.right() != right
                    || input_ancestry_seg.child != mapped_node
                {
                    input_ancestry_seg.segment.left = left;
                    input_ancestry_seg.segment.right = right;
                    input_ancestry_seg.child = mapped_node;
                    *ancestry_change_detected = true;
                }
            } else {
                let seg = AncestrySegment::new(left, right, mapped_node);
                ancestry[node_index].push(seg);
                *ancestry_change_detected = true;
            }
            *output_ancestry_index += 1;
        }
    }
}

pub(crate) fn update_ancestry(
    node_index: usize,
    flags: &[crate::NodeFlags],
    ancestry: &mut [Vec<crate::indexed_node::AncestrySegment>],
    parents: &mut [crate::indexed_node::ParentSet],
    children: &mut [crate::indexed_node::ChildMap],
) -> bool {
    let mut overlapper = make_overlapper(node_index, ancestry, children);
    let input_ancestry_len = ancestry[node_index].len();
    let mut output_ancestry_index: usize = 0;
    let mut ancestry_change_detected = false;

    // remove current node from parents set of children
    for child in children[node_index].keys() {
        assert!(parents[*child].contains(&node_index));
        parents[*child].remove(&node_index);
    }
    children[node_index].clear(); // It'd be nice to not do this.

    process_overlaps(
        input_ancestry_len,
        node_index,
        flags,
        &mut output_ancestry_index,
        &mut ancestry_change_detected,
        &mut overlapper,
        children,
        ancestry,
    );

    if !flags[node_index].is_alive() {
        // Remove trailing input ancestry
        if output_ancestry_index < input_ancestry_len {
            ancestry[node_index].truncate(output_ancestry_index);
            ancestry_change_detected = true;
        }
    }

    for child in children[node_index].keys() {
        parents[*child].insert(node_index);
    }

    ancestry_change_detected || ancestry[node_index].is_empty()
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
