use crate::{segment::Segment, LargeSignedInteger, SignedInteger};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::{cell::RefCell, ops::Deref};

// Use this over std::collections b/c the hashing
// fn is much faster. (We aren't doing cryptography.)
// TODO: See the O'Reilly book for which crate
// they recommend here.
use hashbrown::{HashMap, HashSet};

/// Pointer to an individual.
///
/// Derefs to Rc<RefCell<Individual>>.
/// Required so that we can hash Rc instances.
/// Hashing is via the underlying [Individual](Individual)'s
/// [index](Individual::index) value.
#[derive(Clone)]
pub struct IndividualPointer(Rc<RefCell<Individual>>);

pub type ChildMap = HashMap<IndividualPointer, Vec<Segment>>;
pub type ParentSet = HashSet<IndividualPointer>;

#[derive(Clone)] // NOTE: this does not have to be Clone b/c we work via pointers
pub struct Individual {
    pub index: SignedInteger,
    pub birth_time: LargeSignedInteger,
    pub alive: bool,
    pub parents: ParentSet,
    pub ancestry: Vec<Segment>,
    pub children: ChildMap,
}

impl Deref for IndividualPointer {
    type Target = Rc<RefCell<Individual>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for IndividualPointer {
    fn eq(&self, other: &Self) -> bool {
        self.borrow().index == other.borrow().index
    }
}

impl Eq for IndividualPointer {}

impl Hash for IndividualPointer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.borrow().index.hash(state);
    }
}

impl IndividualPointer {
    pub fn new(index: SignedInteger, birth_time: LargeSignedInteger) -> Self {
        Self {
            0: Rc::new(RefCell::<Individual>::new(Individual::new(
                index, birth_time,
            ))),
        }
    }

    pub fn add_parent(&mut self, parent: IndividualPointer) {
        assert!(self.borrow_mut().birth_time > parent.borrow().birth_time);
        self.borrow_mut().parents.insert(parent);
    }

    pub fn add_child_segment(
        &mut self,
        left: LargeSignedInteger,
        right: LargeSignedInteger,
        child: IndividualPointer,
    ) {
        assert!(child.borrow().birth_time > self.borrow().birth_time);
        let mut b = self.borrow_mut();
        let seg = Segment::new(left, right, None);
        if let Some(v) = b.children.get_mut(&child) {
            v.push(seg);
        } else {
            b.children.insert(child, vec![seg]);
        }
    }
}

impl Individual {
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

// This module is for experimenting with the Rc/RefCell pattern.
#[cfg(test)]
mod practice_tests {
    use super::*;

    fn remove_parent(parent: IndividualPointer, child: IndividualPointer) {
        child.borrow_mut().parents.remove(&parent);
    }

    // Better -- does not increase ref counts just for fn call.
    fn remove_parent_via_ref(parent: &IndividualPointer, child: &IndividualPointer) {
        child.borrow_mut().parents.remove(&parent);
    }

    #[test]
    fn test_interior_mutability() {
        let mut pop: Vec<IndividualPointer> = vec![];

        pop.push(IndividualPointer::new(0, 0));
        pop.push(IndividualPointer::new(1, 1));

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
        let mut pop: Vec<IndividualPointer> = vec![];

        pop.push(IndividualPointer::new(0, 0));
        pop.push(IndividualPointer::new(1, 1));

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
