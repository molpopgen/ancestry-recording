use crate::{interval::Interval, segment::Segment, LargeSignedInteger, SignedInteger};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::{cell::RefCell, ops::Deref};

// Use this over std::collections b/c the hashing
// fn is much faster. (We aren't doing cryptography.)
// TODO: See the O'Reilly book for which crate
// they recommend here.
use hashbrown::{HashMap, HashSet};

/// An individual is a pointer to [IndividualData](IndividualData).
///
/// Derefs to Rc<RefCell<Individual>>, giving interior mutability.
/// Required so that we can hash Rc instances.
/// Hashing is via the underlying [Individual](Individual)'s
/// [index](Individual::index) value.
#[derive(Clone)]
pub struct Individual(Rc<RefCell<IndividualData>>);

pub type ChildMap = HashMap<Individual, Vec<Interval>>;
pub type ParentSet = HashSet<Individual>;

#[derive(Clone)] // NOTE: this does not have to be Clone b/c we work via pointers
pub struct IndividualData {
    pub index: SignedInteger,
    pub birth_time: LargeSignedInteger,
    pub alive: bool,
    pub parents: ParentSet,
    pub ancestry: Vec<Segment>,
    pub children: ChildMap,
}

struct SegmentOverlapper {
    segments: Vec<Segment>,
    overlaps: Rc<RefCell<Vec<Segment>>>, // Prevents copying the segments over and over
    j: usize,
    n: usize,
    right: LargeSignedInteger,
}

struct ChildInputDetails {
    input_number_segs: usize,
    output_number_segs: usize,
}

impl Deref for Individual {
    type Target = Rc<RefCell<IndividualData>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for Individual {
    fn eq(&self, other: &Self) -> bool {
        self.borrow().index == other.borrow().index
    }
}

impl Eq for Individual {}

impl Hash for Individual {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.borrow().index.hash(state);
    }
}

impl Individual {
    pub fn new(index: SignedInteger, birth_time: LargeSignedInteger) -> Self {
        Self(Rc::new(RefCell::<IndividualData>::new(
            IndividualData::new(index, birth_time),
        )))
    }

    pub fn add_parent(&mut self, parent: Individual) {
        assert!(self.borrow_mut().birth_time > parent.borrow().birth_time);
        self.borrow_mut().parents.insert(parent);
    }

    pub fn add_child_segment(
        &mut self,
        left: LargeSignedInteger,
        right: LargeSignedInteger,
        child: Individual,
    ) {
        assert!(child.borrow().birth_time > self.borrow().birth_time);
        let mut b = self.borrow_mut();
        let interval = Interval::new(left, right);
        if let Some(v) = b.children.get_mut(&child) {
            v.push(interval);
        } else {
            b.children.insert(child, vec![interval]);
        }
    }

    fn update_child_segments(
        &mut self,
        child: &Individual,
        left: LargeSignedInteger,
        right: LargeSignedInteger,
        details: &mut HashMap<Self, ChildInputDetails>,
    ) {
        if !details.contains_key(&child) {
            details.insert(child.clone(), ChildInputDetails::new(0));
        }

        let interval = Interval::new(left, right);
        let mut ind = self.borrow_mut();

        // Add child if it does not exist
        if ind.children.get_mut(&child).is_none() {
            ind.children.insert(child.clone(), vec![]);
        }

        if let Some(ref mut entry) = details.get_mut(&child) {
            let child_ref = ind.children.get_mut(&child).unwrap();
            if entry.output_number_segs < entry.input_number_segs {
                child_ref[entry.output_number_segs] = interval;
            } else {
                child_ref.push(interval);
            }
            entry.output_number_segs += 1;
        } else {
            panic!("this cannot happen");
        }
    }

    // FIXME: this is where things are going wrong,
    // and may be the root cause of what we see in update_ancestry.
    pub fn propagate_upwards(&mut self) {
        let mut stack = vec![self.clone()];
        while !stack.is_empty() {
            let mut ind = stack.pop().unwrap();
            ind.update_ancestry();
            assert!(ind.non_overlapping_segments());
            for parent in ind.borrow().parents.iter() {
                stack.push(parent.clone());
            }
        }
    }

    fn update_ancestry(&mut self) {
        let overlapper = SegmentOverlapper::new(self.intersecting_ancestry());

        let mut input_child_details: HashMap<Individual, ChildInputDetails> = HashMap::default();
        let mut current_ancestry_seg = 0_usize;
        let input_ancestry_len: usize;
        let self_alive: bool;

        let mut input_unary_ancestry = vec![];
        let mut input_non_unary_ancestry = vec![];

        {
            let b = self.borrow();
            self_alive = b.alive;
            input_ancestry_len = b.ancestry.len();
            for (c, segs) in &b.children {
                input_child_details.insert(c.clone(), ChildInputDetails::new(segs.len()));
            }
            for (i, a) in b.ancestry.iter().rev().enumerate() {
                let ci = a.child.borrow().index;
                if ci == b.index {
                    input_non_unary_ancestry.push(input_ancestry_len - i - 1);
                } else {
                    input_unary_ancestry.push(input_ancestry_len - i - 1);
                }
            }
        }

        let mut mapped_ind: Individual;

        for (left, right, overlaps) in overlapper {
            let num_overlaps = overlaps.borrow().len();
            if num_overlaps == 1 {
                // unary edge transmission to child.
                //if let Some(ref mut child) = overlaps.borrow_mut()[0].child {
                //    mapped_ind = child.clone();
                //} else {
                //    panic!("cannot happen");
                //}

                let temp_mapped_ind = overlaps.borrow_mut()[0].child.clone();

                {
                    // If mapped_ind is not a child of self,
                    // ensure that self is not a parent of mapped_ind
                    let mut b = self.borrow_mut();
                    if b.children.get_mut(&temp_mapped_ind).is_none() {
                        temp_mapped_ind.borrow_mut().parents.remove(&self);
                    }
                }

                if self_alive {
                    let mapped_ind_alive = temp_mapped_ind.borrow().alive;

                    if mapped_ind_alive {
                        mapped_ind = temp_mapped_ind;
                        self.update_child_segments(
                            &mapped_ind,
                            left,
                            right,
                            &mut input_child_details,
                        );
                    } else {
                        // NOTE: this pattern of traversing the
                        // ancestry to hunt for unary segments
                        // really pisses off the borrow checker.
                        let mut found = false;
                        for a in &temp_mapped_ind.borrow().ancestry {
                            if a.right > left && right > a.left {
                                // NOTE: will panic! if child is None
                                mapped_ind = a.child.clone();

                                mapped_ind.borrow_mut().parents.insert(self.clone());

                                self.update_child_segments(
                                    &mapped_ind,
                                    std::cmp::max(left, a.left),
                                    std::cmp::min(right, a.right),
                                    &mut input_child_details,
                                );
                                found = true;
                                break;
                            }
                        }
                        assert!(found);
                    }
                }
            } else {
                // overlap (coalescence) => ancestry segment maps to self (parent).
                mapped_ind = self.clone();
                for x in overlaps.borrow_mut().iter_mut() {}
            }
        }
    }

    fn intersecting_ancestry(&self) -> Vec<Segment> {
        let mut rv = vec![];

        for (child, segs) in self.borrow().children.iter() {
            for seg in segs.iter() {
                for x in child.borrow().ancestry.iter() {
                    if x.right > seg.left && seg.right > x.left {
                        rv.push(Segment::new(
                            std::cmp::max(x.left, seg.left),
                            std::cmp::min(x.right, seg.right),
                            child.clone(),
                        ));
                    }
                }
            }
        }

        rv
    }

    fn non_overlapping_segments(&self) -> bool {
        let sorted = self
            .borrow()
            .ancestry
            .windows(2)
            .all(|w| w[0].left <= w[1].left);
        if !sorted {
            return false;
        }

        for (_child, segments) in self.borrow().children.iter() {
            let sorted = segments.windows(2).all(|w| w[0].left <= w[1].left);
            if !sorted {
                return false;
            }
        }

        true
    }
}

impl IndividualData {
    pub fn new(index: SignedInteger, birth_time: LargeSignedInteger) -> Self {
        Self {
            index,
            birth_time,
            alive: true,
            parents: ParentSet::default(),
            ancestry: vec![],
            children: ChildMap::default(),
        }
    }
}

impl ChildInputDetails {
    fn new(input_number_segs: usize) -> Self {
        Self {
            input_number_segs,
            output_number_segs: 0,
        }
    }
}

impl SegmentOverlapper {
    fn new(segments: Vec<Segment>) -> Self {
        let mut segments = segments;
        let n = segments.len();
        let overlaps = vec![];

        segments.sort();
        // Sentinel
        segments.push(Segment::new(
            LargeSignedInteger::MAX - 1,
            LargeSignedInteger::MAX,
            // NOTE: dummy individual here to avoid using Option globally for
            // child field of Segment
            Individual::new(SignedInteger::MAX, LargeSignedInteger::MAX),
        ));
        let sorted = segments.windows(2).all(|w| w[0].left <= w[1].left);
        assert!(sorted);
        let right = segments[0].left;
        Self {
            segments,
            overlaps: Rc::new(RefCell::new(overlaps)),
            j: 0,
            n,
            right,
        }
    }
}

impl Iterator for SegmentOverlapper {
    type Item = (
        LargeSignedInteger,
        LargeSignedInteger,
        Rc<RefCell<Vec<Segment>>>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.j < self.n {
            let mut left = self.right;
            self.overlaps.borrow_mut().retain(|x| x.right > left);
            if self.overlaps.borrow().is_empty() {
                left = self.segments[self.j].left;
            }
            while self.j < self.n && self.segments[self.j].left == left {
                self.overlaps
                    .borrow_mut()
                    .push(self.segments[self.j].clone());
                self.j += 1;
            }
            self.j -= 1;
            self.right = self
                .overlaps
                .borrow()
                .iter()
                .fold(LargeSignedInteger::MAX, |a, b| std::cmp::min(a, b.right));
            self.right = std::cmp::min(self.right, self.segments[self.j + 1].right);
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

// This module is for experimenting with the Rc/RefCell pattern.
#[cfg(test)]
mod practice_tests {
    use super::*;

    fn remove_parent(parent: Individual, child: Individual) {
        child.borrow_mut().parents.remove(&parent);
    }

    // Better -- does not increase ref counts just for fn call.
    fn remove_parent_via_ref(parent: &Individual, child: &Individual) {
        child.borrow_mut().parents.remove(&parent);
    }

    #[test]
    fn test_interior_mutability() {
        let mut pop: Vec<Individual> = vec![];

        pop.push(Individual::new(0, 0));
        pop.push(Individual::new(1, 1));

        {
            let c = pop[1].clone();
            pop[0].add_child_segment(0, 1, c);
        }

        {
            let p = pop[0].clone();
            pop[1].add_parent(p);
        }
        assert_eq!(Rc::strong_count(&pop[0]), 2);
        assert_eq!(Rc::strong_count(&pop[1]), 2);

        remove_parent(pop[0].clone(), pop[1].clone());
        assert_eq!(Rc::strong_count(&pop[0]), 1);
        assert_eq!(Rc::strong_count(&pop[1]), 2);
    }

    #[test]
    fn test_interior_mutability_via_ref() {
        let mut pop: Vec<Individual> = vec![];

        pop.push(Individual::new(0, 0));
        pop.push(Individual::new(1, 1));

        {
            let c = pop[1].clone();
            pop[0].add_child_segment(0, 1, c);
        }

        {
            let p = pop[0].clone();
            pop[1].add_parent(p);
        }
        assert_eq!(Rc::strong_count(&pop[0]), 2);
        assert_eq!(Rc::strong_count(&pop[1]), 2);

        remove_parent_via_ref(&pop[0], &pop[1]);
        assert_eq!(Rc::strong_count(&pop[0]), 1);
        assert_eq!(Rc::strong_count(&pop[1]), 2);
    }
}

#[cfg(test)]
mod overlapper_tests {
    use super::*;

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

        let overlapper = SegmentOverlapper::new(parent.intersecting_ancestry());

        let expected = vec![vec![0, 5], vec![1, 6]];

        for (i, (left, right, _overlaps)) in overlapper.enumerate() {
            assert_eq!(expected[i][0], left);
            assert_eq!(expected[i][1], right);
        }
    }
}
