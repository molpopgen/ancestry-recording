use crate::individual_heap::IndividualHeap;
use crate::{
    AncestryIntersection, AncestrySegment, HalfOpenInterval, LargeSignedInteger, NodeFlags,
    Segment, SignedInteger,
};
use crate::{AncestryOverlapper, InlineAncestryError};
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
/// Hashing and equality are implemented with respect to the
/// underlying *pointers* and not the *data*.
#[derive(Clone)]
pub struct Individual(Rc<RefCell<IndividualData>>);

pub type ChildMap = HashMap<Individual, Vec<Segment>>;
pub type ParentSet = HashSet<Individual>;

#[derive(Clone)] // NOTE: this does not have to be Clone b/c we work via pointers
pub struct IndividualData {
    pub index: SignedInteger, // TODO: remove this, as it is really only useful for debugging
    pub birth_time: LargeSignedInteger,
    pub flags: NodeFlags,
    pub parents: ParentSet,
    pub ancestry: Vec<AncestrySegment>,
    pub children: ChildMap,
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
        std::ptr::eq(self.as_ptr(), other.as_ptr())
    }
}

impl Eq for Individual {}

impl Hash for Individual {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state);
    }
}

impl Individual {
    pub fn new_alive(index: SignedInteger, birth_time: LargeSignedInteger) -> Self {
        Self(Rc::new(RefCell::<IndividualData>::new(
            IndividualData::new(index, birth_time),
        )))
    }

    pub fn is_alive(&self) -> bool {
        self.borrow().flags.is_alive()
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
        let interval = Segment::new_unchecked(left, right);
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
        if !details.contains_key(child) {
            details.insert(child.clone(), ChildInputDetails::new(0));
        }

        let interval = Segment::new_unchecked(left, right);
        let mut ind = self.borrow_mut();

        // Add child if it does not exist
        if !ind.children.contains_key(child) {
            ind.children.insert(child.clone(), vec![]);
        }

        if let Some(ref mut entry) = details.get_mut(child) {
            let child_ref = ind.children.get_mut(child).unwrap();
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

    pub fn propagate_upwards(&mut self) -> Result<(), InlineAncestryError> {
        let mut heap = IndividualHeap::new();
        heap.push(self.clone());
        while let Some(mut ind) = heap.pop() {
            let _ = ind.update_ancestry()?;
            ind.non_overlapping_segments()?;
            for parent in ind.borrow().parents.iter() {
                heap.push(parent.clone());
            }
        }
        Ok(())
    }

    fn update_ancestry(&mut self) -> Result<bool, InlineAncestryError> {
        let overlapper = AncestryOverlapper::new(self.intersecting_ancestry());

        let mut input_child_details: HashMap<Individual, ChildInputDetails> = HashMap::default();
        let mut ancestry_change_detected = false;
        let input_ancestry_len: usize;
        let self_alive = self.is_alive();

        // FIXME: the next block is untestable -- should be a separate fn.
        let mut input_unary_ancestry = vec![];
        let mut input_non_unary_ancestry = vec![];

        {
            let b = self.borrow();
            input_ancestry_len = b.ancestry.len();

            // FIXME: untestable and mixed in w/other functionality.
            for (c, segs) in &b.children {
                input_child_details.insert(c.clone(), ChildInputDetails::new(segs.len()));
            }
            for (i, a) in b.ancestry.iter().rev().enumerate() {
                if a.child == *self {
                    input_non_unary_ancestry.push(input_ancestry_len - i - 1);
                } else {
                    input_unary_ancestry.push(input_ancestry_len - i - 1);
                }
            }
        }

        let mut mapped_ind: Option<Individual> = None;

        for (left, right, overlaps) in overlapper {
            let num_overlaps = overlaps.borrow().len();
            if num_overlaps == 1 {
                let temp_mapped_ind = overlaps.borrow_mut()[0].ancestry_segment.child.clone();

                {
                    // If mapped_ind is not a child of self,
                    // ensure that self is not a parent of mapped_ind
                    let mut b = self.borrow_mut();
                    if b.children.get_mut(&temp_mapped_ind).is_none() {
                        temp_mapped_ind.borrow_mut().parents.remove(self);
                    }
                }

                if self_alive {
                    let mapped_ind_alive = temp_mapped_ind.is_alive();

                    if mapped_ind_alive {
                        mapped_ind = Some(temp_mapped_ind);
                        self.update_child_segments(
                            mapped_ind.as_ref().unwrap(),
                            left,
                            right,
                            &mut input_child_details,
                        );
                    } else {
                        mapped_ind = Some(overlaps.borrow_mut()[0].mapped_individual.clone());

                        mapped_ind.as_mut().unwrap().add_parent(self.clone());

                        self.update_child_segments(
                            mapped_ind.as_ref().unwrap(),
                            left,
                            right,
                            &mut input_child_details,
                        );
                    }
                }
            } else {
                // overlap (coalescence) => ancestry segment maps to self (parent).
                mapped_ind = Some(self.clone());
                for x in overlaps.borrow_mut().iter_mut() {
                    self.update_child_segments(
                        &x.mapped_individual,
                        left,
                        right,
                        &mut input_child_details,
                    );
                    x.mapped_individual.add_parent(self.clone());
                }
            }

            assert!(mapped_ind.is_some());

            // FIXME: untestable
            if !self_alive {
                let mut bs = self.borrow_mut();
                if !input_non_unary_ancestry.is_empty() {
                    let idx = input_non_unary_ancestry.pop().unwrap();
                    if left != bs.ancestry[idx].left()
                        || right != bs.ancestry[idx].right()
                        || *mapped_ind.as_ref().unwrap() != bs.ancestry[idx].child
                    {
                        ancestry_change_detected = true;
                    }
                    bs.ancestry[idx] =
                        AncestrySegment::new(left, right, mapped_ind.as_ref().unwrap().clone());
                } else {
                    ancestry_change_detected = true;
                    bs.ancestry.push(AncestrySegment::new(
                        left,
                        right,
                        mapped_ind.as_ref().unwrap().clone(),
                    ));
                }
            }
        }

        // FIXME: untestable
        if !self_alive {
            let mut bs = self.borrow_mut();
            for idx in input_non_unary_ancestry {
                ancestry_change_detected = true;
                bs.ancestry[idx].segment.left = LargeSignedInteger::MIN;
            }
            for idx in input_unary_ancestry {
                if !bs.ancestry[idx].child.is_alive()
                    || (!bs.ancestry[idx].child.borrow().children.is_empty()
                        && bs.ancestry[idx].child.borrow().children.contains_key(self))
                {
                    ancestry_change_detected = true;
                    bs.ancestry[idx].segment.left = LargeSignedInteger::MIN;
                }
            }
            bs.ancestry.retain(|x| x.left() != LargeSignedInteger::MIN);
            bs.ancestry.sort();
        }

        {
            // FIXME: untestable
            let mut bs = self.borrow_mut();

            for (c, s) in bs.children.iter_mut() {
                s.truncate(input_child_details.get(c).unwrap().output_number_segs);
                if s.is_empty() {
                    c.borrow_mut().parents.remove(self);
                }
            }
            bs.children.retain(|_, v| !v.is_empty());
        }

        Ok(ancestry_change_detected || self.borrow().ancestry.is_empty())
    }

    pub(crate) fn intersecting_ancestry(&self) -> Vec<AncestryIntersection> {
        let mut rv = vec![];

        for (child, segs) in self.borrow().children.iter() {
            for seg in segs.iter() {
                for x in child.borrow().ancestry.iter() {
                    if x.overlaps(seg) {
                        rv.push(AncestryIntersection::new(
                            std::cmp::max(x.left(), seg.left()),
                            std::cmp::min(x.right(), seg.right()),
                            child.clone(),
                            x.child.clone(),
                        ));
                    }
                }
            }
        }

        rv
    }

    fn non_overlapping_segments(&self) -> Result<(), InlineAncestryError> {
        let b = self.borrow();
        crate::util::non_overlapping_segments(&b.ancestry)?;
        for (_child, segments) in b.children.iter() {
            crate::util::non_overlapping_segments(&segments)?;
        }
        Ok(())
    }
}

impl IndividualData {
    pub fn new(index: SignedInteger, birth_time: LargeSignedInteger) -> Self {
        Self {
            index,
            birth_time,
            flags: NodeFlags::new_alive(),
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

        pop.push(Individual::new_alive(0, 0));
        pop.push(Individual::new_alive(1, 1));

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

        pop.push(Individual::new_alive(0, 0));
        pop.push(Individual::new_alive(1, 1));

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
