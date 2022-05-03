use crate::individual_heap::IndividualHeap;
use crate::InlineAncestryError;
use crate::{AncestrySegment, LargeSignedInteger, NodeFlags, Segment, SignedInteger};
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
            IndividualData::new_alive(index, birth_time),
        )))
    }

    /// # Panics
    ///
    /// If `genome_length` < 1
    pub fn new_alive_with_ancestry_mapping_to_self(
        index: SignedInteger,
        birth_time: LargeSignedInteger,
        genome_length: LargeSignedInteger,
    ) -> Self {
        let rv = Self::new_alive(index, birth_time);
        rv.borrow_mut()
            .ancestry
            .push(AncestrySegment::new(0, genome_length, rv.clone()));
        rv
    }

    pub fn is_alive(&self) -> bool {
        self.borrow().flags.is_alive()
    }

    // FIXME: this is not a great fn to error from.
    // We should instead be checking that the right thing
    // happens at birth and then, during simplification,
    // rely on assert to find unexpected errors.
    pub fn add_parent(&mut self, parent: Individual) -> Result<(), InlineAncestryError> {
        let mut sb = self.borrow_mut();
        if sb.birth_time > parent.borrow().birth_time {
            sb.parents.insert(parent);
            Ok(())
        } else {
            Err(InlineAncestryError::InvalidBirthTimeOrder {
                parent: parent.borrow().birth_time,
                child: sb.birth_time,
            })
        }
    }

    pub fn add_child_segment(
        &mut self,
        left: LargeSignedInteger,
        right: LargeSignedInteger,
        child: Individual,
    ) -> Result<(), InlineAncestryError> {
        assert!(child.borrow().birth_time > self.borrow().birth_time);
        let mut b = self.borrow_mut();
        let interval = Segment::new(left, right)?;
        if let Some(v) = b.children.get_mut(&child) {
            v.push(interval);
        } else {
            b.children.insert(child, vec![interval]);
        }
        Ok(())
    }

    pub fn propagate_upwards(&mut self) -> Result<(), InlineAncestryError> {
        let mut heap = IndividualHeap::new();
        heap.push(self.clone());
        while let Some(mut ind) = heap.pop() {
            let changed = ind.update_ancestry()?;
            ind.non_overlapping_segments()?;
            // TODO: there is another flag needed here --
            // we don't need to do this for all alive individuals.
            if changed || ind.is_alive() {
                for parent in ind.borrow().parents.iter() {
                    heap.push(parent.clone());
                }
            }
        }
        Ok(())
    }

    #[inline(never)]
    // TODO: to dig more into the performance issues,
    // we need to move this into a separate module and
    // break it up into multiple functions, each of which is not inlined.
    fn update_ancestry(&mut self) -> Result<bool, InlineAncestryError> {
        let rv = crate::update_ancestry::update_ancestry(self);
        Ok(rv)
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
    pub fn new_alive(index: SignedInteger, birth_time: LargeSignedInteger) -> Self {
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
            pop[0].add_child_segment(0, 1, c).unwrap();
        }

        {
            let p = pop[0].clone();
            pop[1].add_parent(p).unwrap();
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
            pop[0].add_child_segment(0, 1, c).unwrap();
        }

        {
            let p = pop[0].clone();
            pop[1].add_parent(p).unwrap();
        }
        assert_eq!(Rc::strong_count(&pop[0]), 2);
        assert_eq!(Rc::strong_count(&pop[1]), 2);

        remove_parent_via_ref(&pop[0], &pop[1]);
        assert_eq!(Rc::strong_count(&pop[0]), 1);
        assert_eq!(Rc::strong_count(&pop[1]), 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alive_individual_has_ancestry_to_self() {
        let ind = Individual::new_alive_with_ancestry_mapping_to_self(0, 0, 10);
        assert_eq!(ind.borrow().ancestry.len(), 1);
        assert!(ind.borrow().ancestry[0].child == ind);
    }

    #[test]
    fn test_equality() {
        let ind = Individual::new_alive(0, 1);
        let clone = ind.clone();
        assert!(ind == clone);
    }

    #[test]
    fn test_equality_after_interior_mutation() {
        let ind = Individual::new_alive(0, 1);
        let clone = ind.clone();

        assert!(ind.borrow().ancestry.is_empty());

        let another_ind = Individual::new_alive(0, 1);
        ind.borrow_mut()
            .ancestry
            .push(AncestrySegment::new(0, 1, another_ind.clone()));
        assert!(!ind.borrow().ancestry.is_empty());
        assert!(!clone.borrow().ancestry.is_empty());
        assert!(ind == clone);

        assert!(ind.is_alive());
        ind.borrow_mut().flags.remove(NodeFlags::IS_ALIVE);
        assert!(!ind.is_alive());
        assert!(!clone.is_alive());
    }

    #[test]
    fn test_inequality() {
        let ind = Individual::new_alive(0, 1);
        let another_ind = Individual::new_alive(0, 1);
        assert!(ind != another_ind);
    }

    #[test]
    fn test_hashing() {
        let mut hash = hashbrown::HashSet::new();
        let ind = Individual::new_alive(0, 1);
        let clone = ind.clone();
        let another_ind = Individual::new_alive(0, 1);

        hash.insert(ind.clone());
        assert!(hash.contains(&ind));
        assert!(hash.contains(&clone));
        assert!(!hash.contains(&another_ind));

        ind.borrow_mut()
            .ancestry
            .push(AncestrySegment::new(0, 1, another_ind.clone()));
        assert!(hash.contains(&ind));
        assert!(hash.contains(&clone));
        assert!(!hash.contains(&another_ind));
    }
}
